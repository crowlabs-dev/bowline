use std::path::PathBuf;

use bowline_core::{
    commands::{
        AgentBudgetCommandOutput, AgentCliCapability, AgentCliName, AgentContextCommandOutput,
        AgentLeaseBase, AgentLeaseCreateCommandOutput, AgentPromptCommandOutput,
        AgentToolAuthority, AgentToolInvokeRequest, AgentToolName, AgentToolResult,
        AgentToolTransport, CONTRACT_VERSION,
    },
    ids::{DeviceId, LeaseId},
    status::SafeAction,
};
use bowline_local::agents::{
    AgentBudgetGrantOptions, AgentError, AgentLeaseCreateOptions, AgentLeaseSelectorOptions,
    agent_context, agent_prompt, create_agent_lease, grant_agent_hydration_budget,
    invoke_agent_tool_from_local_daemon,
};
use serde_json::Map;

use crate::io_helpers;
use crate::surface::style::{self, Presentation, Role};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLeaseCreateArgs {
    pub project_path: String,
    pub task: String,
    pub base: AgentLeaseBase,
    pub hydrate_budget_bytes: u64,
    pub work_view: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLeaseSelectorArgs {
    pub lease_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBudgetArgs {
    pub lease_id: String,
    pub add_bytes: u64,
}

pub fn parse_base(value: &str) -> Option<AgentLeaseBase> {
    match value {
        "latest-workspace" => Some(AgentLeaseBase::LatestWorkspace),
        "latest:main" => Some(AgentLeaseBase::LatestMain),
        _ => None,
    }
}

pub fn run_lease_create(
    args: AgentLeaseCreateArgs,
    db_path: Option<PathBuf>,
    device_id: DeviceId,
    generated_at: String,
) -> Result<AgentLeaseCreateCommandOutput, AgentError> {
    create_agent_lease(AgentLeaseCreateOptions {
        db_path,
        project_path: args.project_path,
        task: args.task,
        base: args.base,
        hydrate_budget_bytes: args.hydrate_budget_bytes,
        work_view: args.work_view,
        device_id,
        generated_at,
    })
}

pub fn run_context(
    args: AgentLeaseSelectorArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<AgentContextCommandOutput, AgentError> {
    let mut output = agent_context(AgentLeaseSelectorOptions {
        db_path,
        lease_id: LeaseId::new(args.lease_id),
        generated_at,
    })?;
    output.context.adapter_capabilities = crate::agent_adapters::detect_agent_cli_capabilities();
    add_agent_launch_actions(&mut output);
    Ok(output)
}

pub fn run_prompt(
    args: AgentLeaseSelectorArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<AgentPromptCommandOutput, AgentError> {
    let mut output = agent_prompt(AgentLeaseSelectorOptions {
        db_path,
        lease_id: LeaseId::new(args.lease_id),
        generated_at,
    })?;
    output.prompt.adapter_capabilities = crate::agent_adapters::detect_agent_cli_capabilities();
    Ok(output)
}

pub fn run_publish(
    args: AgentLeaseSelectorArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<AgentToolResult, AgentError> {
    invoke_agent_tool_from_local_daemon(
        db_path,
        tool_request(
            &args.lease_id,
            AgentToolName::PublishOverlayForReview,
            &generated_at,
        ),
        true,
        generated_at,
    )
}

pub fn run_complete(
    args: AgentLeaseSelectorArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<AgentToolResult, AgentError> {
    invoke_agent_tool_from_local_daemon(
        db_path,
        tool_request(&args.lease_id, AgentToolName::CompleteTask, &generated_at),
        true,
        generated_at,
    )
}

pub fn run_budget(
    args: AgentBudgetArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<AgentBudgetCommandOutput, AgentError> {
    grant_agent_hydration_budget(AgentBudgetGrantOptions {
        db_path,
        lease_id: LeaseId::new(args.lease_id),
        add_bytes: args.add_bytes,
        generated_at,
    })
}

pub fn render_lease_create_human(output: &AgentLeaseCreateCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let target_label = match output.lease.write_target_mode {
        bowline_core::commands::AgentWriteTargetMode::Direct => "Project",
        bowline_core::commands::AgentWriteTargetMode::WorkView => "Work view",
    };
    format!(
        "{}  {}\n{}  {}\n{}  {}\n\n",
        style::section("Agent lease", &pres),
        style::paint(output.lease.id.as_str(), Role::Strong, &pres),
        style::section(target_label, &pres),
        output.lease.write_target_path,
        style::section("State", &pres),
        style::paint("active", Role::Ready, &pres),
    )
}

pub fn render_context_human(output: &AgentContextCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let capabilities = output
        .context
        .capabilities
        .iter()
        .map(|capability| style::kebab(&capability.name))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{}  {}\n{}  {}\n{}  {}\n{}  {}\n\n",
        style::section("Agent lease", &pres),
        style::paint(output.context.lease.id.as_str(), Role::Strong, &pres),
        style::section("Target", &pres),
        output.context.lease.write_target_path,
        style::section("Readiness", &pres),
        style::kebab(&output.context.readiness.state),
        style::section("Capabilities", &pres),
        capabilities,
    )
}

pub fn render_prompt_human(output: &AgentPromptCommandOutput) -> String {
    format!("{}\n", output.prompt.text)
}

pub fn render_tool_human(output: &AgentToolResult) -> String {
    let pres = Presentation::detect(false);
    format!(
        "{}  {}\n{}  {}\n{}  {}\n\n",
        style::section("Agent tool", &pres),
        style::kebab(&output.tool),
        style::section("Outcome", &pres),
        style::kebab(&output.outcome),
        style::section("Summary", &pres),
        output.summary,
    )
}

pub fn render_budget_human(output: &AgentBudgetCommandOutput) -> String {
    let pres = Presentation::detect(false);
    format!(
        "{}  {}\n{}  {} bytes -> {} bytes\n{}  {} bytes\n\n",
        style::section("Agent lease", &pres),
        style::paint(output.lease.id.as_str(), Role::Strong, &pres),
        style::section("Hydration budget", &pres),
        output.previous_limit_bytes,
        output.budget.limit_bytes,
        style::section("Remaining", &pres),
        output.budget.remaining_bytes,
    )
}

fn add_agent_launch_actions(output: &mut AgentContextCommandOutput) {
    let Some(codex) = output
        .context
        .adapter_capabilities
        .iter()
        .find(|capability| capability.name == AgentCliName::Codex)
    else {
        return;
    };
    if !supports_codex_launch(codex) {
        return;
    }
    let bowline = std::env::current_exe()
        .ok()
        .map(|path| io_helpers::shell_word(&path.display().to_string()))
        .unwrap_or_else(|| "~/.local/bin/bowline".to_string());
    let command = format!(
        "export PATH=\"$HOME/.local/bin:$PATH\"; {} agent prompt --lease {} | codex exec --cd {} --sandbox workspace-write --add-dir ~/.local/share/bowline --add-dir ~/.local/state/bowline --add-dir ~/.local/state/bowline --add-dir \"$HOME/Library/Application Support/bowline\" --skip-git-repo-check -",
        bowline,
        io_helpers::shell_word(output.context.lease.id.as_str()),
        io_helpers::shell_word(&output.context.start_work.cwd),
    );
    if output
        .context
        .start_work
        .safe_next_actions
        .iter()
        .any(|action| action.command.as_deref() == Some(command.as_str()))
    {
        return;
    }
    output
        .context
        .start_work
        .safe_next_actions
        .push(SafeAction {
            label: "Launch Codex in this lease target".to_string(),
            command: Some(command),
        });
}

fn tool_request(lease_id: &str, tool: AgentToolName, generated_at: &str) -> AgentToolInvokeRequest {
    AgentToolInvokeRequest {
        message_type: "agent.tool.invoke".to_string(),
        protocol_version: CONTRACT_VERSION,
        request_id: format!(
            "req_{}_{}",
            tool_request_name(tool),
            generated_at
                .chars()
                .map(|character| if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                })
                .collect::<String>()
        ),
        lease_id: LeaseId::new(lease_id.to_string()),
        tool,
        authority: AgentToolAuthority {
            transport: AgentToolTransport::LocalDaemon,
            peer_credential_checked: false,
            nonce_presented: false,
        },
        arguments: Map::new(),
    }
}

fn tool_request_name(tool: AgentToolName) -> &'static str {
    match tool {
        AgentToolName::PublishOverlayForReview => "publish",
        AgentToolName::CompleteTask => "complete",
        _ => "tool",
    }
}

fn supports_codex_launch(capability: &AgentCliCapability) -> bool {
    capability.available
        && capability.supports_stdin_launch
        && capability.supports_cwd_selection
        && capability.supports_noninteractive_execution
}
