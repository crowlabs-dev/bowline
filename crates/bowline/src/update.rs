use super::*;
use semver::Version;
use serde::Deserialize;
use std::fs;

const DEFAULT_INSTALL_HOST: &str = "https://install.bowline.sh";
const ENV_INSTALLER_URL: &str = "BOWLINE_UPDATE_INSTALLER_URL";
const ENV_MANIFEST_URL: &str = "BOWLINE_UPDATE_MANIFEST_URL";
const ENV_CACHE_PATH: &str = "BOWLINE_UPDATE_CACHE";
const ENV_DISABLE_UPDATE_CHECK: &str = "BOWLINE_UPDATE_DISABLE";
const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReleaseManifest {
    pub(super) version: String,
    #[serde(default)]
    pub(super) urgency: UpdateUrgency,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum UpdateUrgency {
    #[default]
    Normal,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UpdateCheck {
    pub(super) current_version: String,
    pub(super) latest_version: String,
    pub(super) update_available: bool,
    pub(super) urgency: UpdateUrgency,
}

pub(super) fn print_update(args: UpdateArgs, json: bool) -> ExitCode {
    let generated_at = generated_at();
    let check = match check_for_update_fresh(args.version.as_deref()) {
        Ok(check) => check,
        Err(error) => {
            print_runtime_error(CommandName::Update, generated_at, &error, json);
            return ExitCode::from(EXIT_RUNTIME);
        }
    };
    let output = update_output(&check, &generated_at, args.version.as_deref());

    if args.check {
        if json {
            print_json(&output);
        } else {
            print!("{}", render_update_human(&check));
        }
        return ExitCode::SUCCESS;
    }

    if !check.update_available && args.version.is_none() {
        if json {
            print_json(&output);
        } else {
            println!("Bowline is up to date ({CLI_VERSION}).");
        }
        return ExitCode::SUCCESS;
    }

    let installer = match download_installer(&installer_url()) {
        Ok(installer) => installer,
        Err(error) => {
            print_runtime_error(CommandName::Update, generated_at, &error, json);
            return ExitCode::from(EXIT_RUNTIME);
        }
    };
    let mut command = ProcessCommand::new("sh");
    command.arg(&installer);
    if let Some(version) = args.version.as_deref() {
        command.args(["--version", version]);
    }
    if json {
        match command.output() {
            Ok(result) if result.status.success() => {
                let _ = fs::remove_file(&installer);
                print_json(&output);
                ExitCode::SUCCESS
            }
            Ok(result) => {
                let _ = fs::remove_file(&installer);
                let message = String::from_utf8_lossy(&result.stderr);
                print_runtime_error(CommandName::Update, generated_at, message.trim(), true);
                ExitCode::from(EXIT_RUNTIME)
            }
            Err(error) => {
                let _ = fs::remove_file(&installer);
                print_runtime_error(CommandName::Update, generated_at, &error.to_string(), true);
                ExitCode::from(EXIT_RUNTIME)
            }
        }
    } else {
        println!("Updating Bowline from {}", installer_url());
        match command.status() {
            Ok(status) if status.success() => {
                let _ = fs::remove_file(&installer);
                ExitCode::SUCCESS
            }
            Ok(status) => {
                let _ = fs::remove_file(&installer);
                print_runtime_error(
                    CommandName::Update,
                    generated_at,
                    &format!("installer exited with status {status}"),
                    false,
                );
                ExitCode::from(EXIT_RUNTIME)
            }
            Err(error) => {
                let _ = fs::remove_file(&installer);
                print_runtime_error(CommandName::Update, generated_at, &error.to_string(), false);
                ExitCode::from(EXIT_RUNTIME)
            }
        }
    }
}

pub(super) fn check_for_update(
    version: Option<&str>,
    allow_network: bool,
) -> Result<UpdateCheck, String> {
    check_for_update_with_policy(version, allow_network, false)
}

fn check_for_update_fresh(version: Option<&str>) -> Result<UpdateCheck, String> {
    check_for_update_with_policy(version, true, true)
}

fn check_for_update_with_policy(
    version: Option<&str>,
    allow_network: bool,
    force_fetch: bool,
) -> Result<UpdateCheck, String> {
    let manifest = load_manifest(version, allow_network, force_fetch)?;
    Ok(UpdateCheck {
        current_version: CLI_VERSION.to_string(),
        latest_version: manifest.version.clone(),
        update_available: version_is_newer(&manifest.version, CLI_VERSION),
        urgency: manifest.urgency,
    })
}

pub(super) fn attach_update_status_if_available(
    output: &mut StatusCommandOutput,
    allow_network: bool,
) {
    if env::var(ENV_DISABLE_UPDATE_CHECK).ok().as_deref() == Some("1") {
        return;
    }
    let Ok(check) = check_for_update(None, allow_network) else {
        return;
    };
    if !check.update_available {
        return;
    }

    let required = check.urgency == UpdateUrgency::Required;
    if required {
        output.status.level = StatusLevel::Limited;
        output.status.attention_items.push(format!(
            "Bowline {} is required before this version continues.",
            check.latest_version
        ));
    }

    output.items.push(StatusItem {
        kind: StatusItemKind::Update,
        summary: if required {
            format!(
                "Required Bowline update available: {} -> {}.",
                check.current_version, check.latest_version
            )
        } else {
            format!(
                "Bowline update available: {} -> {}.",
                check.current_version, check.latest_version
            )
        },
        subject: Some(StatusSubject {
            kind: StatusSubjectKind::Component,
            id: format!("bowline-update-{}", check.latest_version),
            path: None,
        }),
        path: None,
        classification: None,
        mode: None,
        access: Vec::new(),
        event_id: None,
        event_name: None,
        device_id: None,
        lease_id: None,
        project_id: output.project_id.clone(),
        snapshot_id: None,
        policy_version: None,
        env_record_id: None,
    });
    output.next_actions.push(SafeAction {
        label: "Update Bowline".to_string(),
        command: Some("bowline update".to_string()),
    });
}

fn load_manifest(
    version: Option<&str>,
    allow_network: bool,
    force_fetch: bool,
) -> Result<ReleaseManifest, String> {
    let cache = cache_path(version);
    if allow_network && (force_fetch || should_fetch(&cache)) {
        match curl_text(&manifest_url(version), 2) {
            Ok(text) => {
                let manifest = parse_manifest(&text)?;
                if let Some(parent) = cache.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&cache, text);
                return Ok(manifest);
            }
            Err(error) if force_fetch => {
                return Err(format!("could not fetch release manifest: {error}"));
            }
            Err(_) => {}
        }
    }
    let text = fs::read_to_string(&cache)
        .map_err(|_| "could not fetch release manifest and no cached manifest is available")?;
    parse_manifest(&text)
}

fn parse_manifest(text: &str) -> Result<ReleaseManifest, String> {
    serde_json::from_str(text).map_err(|error| format!("invalid release manifest: {error}"))
}

fn should_fetch(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return true;
    };
    let Ok(modified) = metadata.modified() else {
        return true;
    };
    modified.elapsed().map_or(true, |age| age >= CACHE_TTL)
}

fn curl_text(url: &str, timeout_secs: u64) -> Result<String, String> {
    let output = ProcessCommand::new("curl")
        .args([
            "-fsSL",
            "--retry",
            "1",
            "--max-time",
            &timeout_secs.to_string(),
            url,
        ])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    String::from_utf8(output.stdout).map_err(|error| error.to_string())
}

fn download_installer(url: &str) -> Result<PathBuf, String> {
    let path = env::temp_dir().join(format!(
        "bowline-install-{}-{}.sh",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    ));
    let path_arg = path.to_string_lossy().into_owned();
    let output = ProcessCommand::new("curl")
        .args([
            "-fsSL",
            "--retry",
            "1",
            "--max-time",
            "30",
            "-o",
            &path_arg,
            url,
        ])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(path)
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    let Ok(latest) = Version::parse(latest.trim_start_matches('v')) else {
        return false;
    };
    let Ok(current) = Version::parse(current.trim_start_matches('v')) else {
        return false;
    };
    latest > current
}

fn update_output(
    check: &UpdateCheck,
    generated_at: &str,
    requested_version: Option<&str>,
) -> UpdateCommandOutput {
    UpdateCommandOutput {
        contract_version: CONTRACT_VERSION,
        ok: true,
        command: CommandName::Update,
        generated_at: generated_at.to_string(),
        current_version: check.current_version.clone(),
        latest_version: check.latest_version.clone(),
        update_available: check.update_available,
        update_command: update_command(requested_version),
    }
}

fn render_update_human(check: &UpdateCheck) -> String {
    if check.update_available {
        format!(
            "Bowline update available: {} -> {}\nRun: bowline update\n",
            check.current_version, check.latest_version
        )
    } else {
        format!("Bowline is up to date ({})\n", check.current_version)
    }
}

fn update_command(version: Option<&str>) -> String {
    match version {
        Some(version) => format!("bowline update --version {version}"),
        None => "bowline update".to_string(),
    }
}

fn installer_url() -> String {
    env::var(ENV_INSTALLER_URL).unwrap_or_else(|_| format!("{DEFAULT_INSTALL_HOST}/install.sh"))
}

fn manifest_url(version: Option<&str>) -> String {
    if let Ok(url) = env::var(ENV_MANIFEST_URL) {
        return url;
    }
    match version {
        Some(version) if version.starts_with('v') => {
            format!("{DEFAULT_INSTALL_HOST}/releases/{version}/release-manifest.json")
        }
        Some(version) => {
            format!("{DEFAULT_INSTALL_HOST}/releases/v{version}/release-manifest.json")
        }
        None => format!("{DEFAULT_INSTALL_HOST}/release-manifest.json"),
    }
}

fn cache_path(version: Option<&str>) -> PathBuf {
    if let Ok(path) = env::var(ENV_CACHE_PATH) {
        return PathBuf::from(path);
    }
    let name = version
        .map(|version| format!("release-manifest-{version}.json"))
        .unwrap_or_else(|| "release-manifest.json".to_string());
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir())
        .join(".local/state/bowline")
        .join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_check_detects_newer_versions() {
        assert!(version_is_newer("0.1.1", "0.1.0"));
        assert!(version_is_newer("v1.0.0", "0.9.9"));
        assert!(version_is_newer("0.2.0", "0.2.0-beta.1"));
        assert!(!version_is_newer("0.1.0", "0.1.0"));
        assert!(!version_is_newer("0.0.9", "0.1.0"));
        assert!(!version_is_newer("0.2.0-beta.1", "0.2.0"));
    }

    #[test]
    fn parses_required_manifest() {
        let manifest = parse_manifest(r#"{"version":"9.0.0","urgency":"required"}"#).unwrap();

        assert_eq!(manifest.version, "9.0.0");
        assert_eq!(manifest.urgency, UpdateUrgency::Required);
    }
}
