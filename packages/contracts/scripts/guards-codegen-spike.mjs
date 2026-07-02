#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";

const packageRoot = path.resolve(import.meta.dirname, "..");
const repoRoot = path.resolve(packageRoot, "../..");
const guardsPath = path.join(packageRoot, "src/guards.ts");
const tempTestPath = path.join(
  packageRoot,
  "src/__tests__/.guards-codegen-spike.generated.test.ts",
);
const tempOutputPath = path.join(
  packageRoot,
  "src/__tests__/.guards-codegen-spike.output.json",
);

function sourceFile(filePath) {
  return ts.createSourceFile(
    filePath,
    readFileSync(filePath, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
}

function exportedGuardInventory() {
  const source = sourceFile(guardsPath);
  const guards = [];
  for (const statement of source.statements) {
    if (
      !ts.isFunctionDeclaration(statement) ||
      !statement.name ||
      !statement.modifiers?.some(
        (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
      ) ||
      !/^is[A-Z]/u.test(statement.name.text)
    ) {
      continue;
    }

    const body = statement.body?.getText(source) ?? "";
    const deliberateReasons = [];
    if (
      /\bCONTRACT_VERSION\b/u.test(body) ||
      /\.[A-Za-z0-9_]+\s*[!=]==\s*["'`]/u.test(body) ||
      /\bvalue\s*[!=]==\s*["'`]/u.test(body)
    ) {
      deliberateReasons.push("literal pin");
    }
    if (
      /\.every\(/u.test(body) ||
      /\.some\(/u.test(body) ||
      /Object\.(entries|values)\(/u.test(body) ||
      /\.\w+\.length\s*[!=<>]=?/u.test(body)
    ) {
      deliberateReasons.push("collection/cross-field check");
    }
    if (/\?\s*[^:]+:/u.test(body) || /\?\?/u.test(body)) {
      deliberateReasons.push("optional fallback");
    }

    guards.push({
      name: statement.name.text,
      category: deliberateReasons.length > 0 ? "deliberate" : "mechanical",
      deliberateReasons,
    });
  }
  return guards;
}

function writeCoverageProbe() {
  mkdirSync(path.dirname(tempTestPath), { recursive: true });
  writeFileSync(
    tempTestPath,
    `import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import * as guards from "../guards";

type Fixture = { readonly name: string; readonly value: unknown };

function readJsonFixtures(dir: string): readonly Fixture[] {
  const fixtureDir = path.resolve(import.meta.dirname, "../../../../tests/contracts", dir);
  return readdirSync(fixtureDir)
    .filter((name) => name.endsWith(".json"))
    .sort()
    .map((name) => ({
      name: \`\${dir}/\${name}\`,
      value: JSON.parse(readFileSync(path.join(fixtureDir, name), "utf8")) as unknown,
    }));
}

function readStreamFixtures(): readonly Fixture[] {
  const filePath = path.resolve(
    import.meta.dirname,
    "../../../../tests/contracts/streams/status-watch.ndjson",
  );
  return readFileSync(filePath, "utf8")
    .trim()
    .split(/\\r?\\n/u)
    .map((line, index) => ({
      name: \`streams/status-watch.ndjson#\${index + 1}\`,
      value: JSON.parse(line) as unknown,
    }));
}

const fixtures = [
  ...readJsonFixtures("commands"),
  ...readJsonFixtures("status"),
  ...readJsonFixtures("events"),
  ...readJsonFixtures("snapshots"),
  ...readStreamFixtures(),
];

const guardEntries = Object.entries(guards)
  .filter(([name, guard]) => /^is[A-Z]/u.test(name) && typeof guard === "function")
  .sort(([left], [right]) => left.localeCompare(right));

describe("guards codegen spike fixture oracle coverage", () => {
  it("prints shared fixture coverage for exported guards", () => {
    const rows = guardEntries.map(([name, guard]) => {
      const hits = fixtures
        .filter((fixture) => (guard as (value: unknown) => boolean)(fixture.value))
        .map((fixture) => fixture.name);
      return { name, hits };
    });
    const covered = rows.filter((row) => row.hits.length > 0);
    const uncovered = rows.filter((row) => row.hits.length === 0);
    const summary = {
      fixtureCount: fixtures.length,
      totalGuards: rows.length,
      covered: covered.length,
      uncovered: uncovered.length,
      uncoveredPct: Number(((uncovered.length / rows.length) * 100).toFixed(1)),
      uncoveredGuards: uncovered.map((row) => row.name),
      coveredGuards: covered.map((row) => ({
        name: row.name,
        hits: row.hits,
      })),
    };
    writeFileSync(${JSON.stringify(tempOutputPath)}, JSON.stringify(summary));
    expect(summary.totalGuards).toBeGreaterThan(0);
  });
});
`,
  );
}

function runCoverageProbe() {
  writeCoverageProbe();
  try {
    const output = execFileSync(
      "pnpm",
      [
        "exec",
        "vitest",
        "run",
        "src/__tests__/.guards-codegen-spike.generated.test.ts",
      ],
      {
        cwd: packageRoot,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    const coverage = readFileSync(tempOutputPath, "utf8");
    return `${output}\nGUARDS_CODEGEN_SPIKE_COVERAGE ${coverage}`;
  } finally {
    rmSync(tempTestPath, { force: true });
    rmSync(tempOutputPath, { force: true });
  }
}

const inventory = exportedGuardInventory();
const mechanical = inventory.filter((guard) => guard.category === "mechanical");
const deliberate = inventory.filter((guard) => guard.category === "deliberate");

console.log(
  "GUARDS_CODEGEN_SPIKE_INVENTORY " +
    JSON.stringify({
      totalGuards: inventory.length,
      mechanical: mechanical.length,
      deliberate: deliberate.length,
      mechanicalGuards: mechanical.map((guard) => guard.name),
      deliberateGuards: deliberate.map((guard) => ({
        name: guard.name,
        reasons: guard.deliberateReasons,
      })),
    }),
);
console.log(runCoverageProbe());
console.log(`repoRoot=${repoRoot}`);
