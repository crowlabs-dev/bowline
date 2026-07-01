use super::*;

use crate::surface::style::{self, Presentation, Role};

pub(super) fn render_search_human(output: &bowline_core::commands::SearchCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = vec![format!(
        "{}  {} for {}",
        style::section("Search", &pres),
        style::count_noun(output.results.len() as u64, "result", "results"),
        style::paint(&format!("`{}`", output.query), Role::Accent, &pres),
    )];
    for result in &output.results {
        let location = result
            .line_start
            .map(|line| format!(":{line}"))
            .unwrap_or_default();
        lines.push(format!(
            "  {}{}  {}",
            result.path,
            location,
            style::paint(&format!("score {:.1}", result.score), Role::Label, &pres),
        ));
        if let Some(snippet) = &result.snippet {
            lines.push(format!("    {}", style::paint(snippet, Role::Label, &pres)));
        }
    }
    if output.results.is_empty() {
        lines.push(format!(
            "  {}",
            style::paint("No indexed matches.", Role::Label, &pres)
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn render_symbols_human(output: &bowline_core::commands::SymbolCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = vec![format!(
        "{}  {} for {}",
        style::section("Symbols", &pres),
        style::count_noun(output.symbols.len() as u64, "result", "results"),
        style::paint(&format!("`{}`", output.query), Role::Accent, &pres),
    )];
    for symbol in &output.symbols {
        lines.push(format!(
            "  {}  {}  {}",
            style::paint(&symbol.name, Role::Strong, &pres),
            style::paint(
                &format!(
                    "{} · {}",
                    style::kebab(&symbol.kind),
                    style::kebab(&symbol.language)
                ),
                Role::Label,
                &pres,
            ),
            style::paint(
                &format!("{}:{}", symbol.path, symbol.line_start),
                Role::Label,
                &pres,
            ),
        ));
    }
    if output.symbols.is_empty() {
        lines.push(format!(
            "  {}",
            style::paint("No indexed symbols.", Role::Label, &pres)
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn render_login_human(output: &bowline_core::commands::LoginCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = Vec::new();
    let head = |state: &str, role: Role| {
        format!(
            "{}  {}",
            style::section("Login", &pres),
            style::paint(state, role, &pres)
        )
    };
    match output.account.status {
        bowline_core::devices::AccountLoginStatus::LoginPending => {
            lines.push(head("waiting for browser approval", Role::Preparing));
            if let Some(uri) = output
                .account
                .verification_uri_complete
                .as_ref()
                .or(output.account.verification_uri.as_ref())
            {
                lines.push(format!(
                    "{}  {}",
                    style::section("Open", &pres),
                    style::paint(uri, Role::Accent, &pres)
                ));
            }
            if let Some(code) = &output.account.user_code {
                lines.push(format!(
                    "{}  {}",
                    style::section("Code", &pres),
                    style::paint(code, Role::Strong, &pres)
                ));
            }
        }
        bowline_core::devices::AccountLoginStatus::AccountAuthenticated => {
            lines.push(head("authenticated", Role::Ready));
            if let Some(account_id) = &output.account.account_id {
                lines.push(format!(
                    "{}  {}",
                    style::section("Account", &pres),
                    account_id.as_str()
                ));
            }
        }
        bowline_core::devices::AccountLoginStatus::Expired => {
            lines.push(head("expired", Role::Attention));
        }
        bowline_core::devices::AccountLoginStatus::NotLoggedIn => {
            lines.push(head("not logged in", Role::Label));
        }
    }
    append_next_actions(&mut lines, &output.next_actions);
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn render_logout_human(output: &bowline_core::commands::LogoutCommandOutput) -> String {
    let mut lines = Vec::new();
    if output.signed_out {
        lines.push("Logout: signed out".to_string());
    } else {
        lines.push("Logout: already signed out".to_string());
    }
    append_next_actions(&mut lines, &output.next_actions);
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn render_init_human(output: &bowline_core::commands::InitCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = vec![
        format!(
            "{}  {}",
            style::section("Root", &pres),
            style::paint(&output.root, Role::Strong, &pres)
        ),
        format!(
            "{}  {}",
            style::section("State", &pres),
            style::paint("observed locally; sync not started", Role::Preparing, &pres)
        ),
        format!(
            "{}  {} · {} · {}",
            style::section("Observed", &pres),
            style::count_noun(output.scan_summary.repo_count, "repo", "repos"),
            style::count_noun(
                output.scan_summary.workspace_sync_path_count,
                "file",
                "files"
            ),
            style::count_noun(output.scan_summary.env_file_count, "env file", "env files"),
        ),
    ];
    if output.created_root {
        lines.push(format!(
            "  {}",
            style::paint("Created root directory.", Role::Label, &pres)
        ));
    }
    if !output.non_actions.is_empty() {
        lines.push(style::section("Did not", &pres));
        lines.extend(
            output
                .non_actions
                .iter()
                .map(|item| format!("  {}", style::paint(item, Role::Label, &pres))),
        );
    }
    append_next_actions(&mut lines, &output.next_actions);
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn render_devices_human(
    output: &bowline_core::commands::DevicesCommandOutput,
) -> String {
    let pres = Presentation::detect(false);
    let mut lines = Vec::new();
    match output.action {
        bowline_core::commands::DeviceCommandAction::List => {
            lines.push(format!(
                "{}  {} trusted · {} pending · {} revoked",
                style::section("Devices", &pres),
                output.devices.len(),
                output.pending_requests.len(),
                output.revoked_devices.len()
            ));
            lines.extend(output.devices.iter().map(|device| {
                let marker = if device.is_current_device {
                    style::paint("  (this device)", Role::Accent, &pres)
                } else {
                    String::new()
                };
                format!(
                    "  {}  {}{marker}",
                    device.name,
                    style::paint("trusted", Role::Ready, &pres)
                )
            }));
            lines.extend(output.pending_requests.iter().map(|request| {
                let state = match request.state {
                    bowline_core::devices::DeviceApprovalRequestState::Pending => {
                        "waiting for approval"
                    }
                    bowline_core::devices::DeviceApprovalRequestState::Approved => {
                        "approved, waiting for acceptance"
                    }
                    bowline_core::devices::DeviceApprovalRequestState::Denied => "denied",
                    bowline_core::devices::DeviceApprovalRequestState::Expired => "expired",
                };
                format!(
                    "  {}  {}  {}",
                    request.device_name,
                    style::paint(state, Role::Attention, &pres),
                    style::paint(
                        &format!(
                            "code {} ({})",
                            request.matching_code,
                            request.request_id.as_str()
                        ),
                        Role::Label,
                        &pres,
                    )
                )
            }));
        }
        bowline_core::commands::DeviceCommandAction::Request => {
            if let Some(request) = &output.created_request {
                lines.push(format!(
                    "{}  {}",
                    style::section("Device request", &pres),
                    request.request_id.as_str()
                ));
                lines.push(format!(
                    "{}  {}",
                    style::section("Code", &pres),
                    style::paint(&request.matching_code, Role::Strong, &pres)
                ));
                lines.push(style::paint(
                    "Waiting for approval on an existing trusted device.",
                    Role::Preparing,
                    &pres,
                ));
            } else {
                lines.push("Device request created.".to_string());
            }
        }
        bowline_core::commands::DeviceCommandAction::Approve => {
            lines.push(approval_line(
                "Approved",
                output.approved_device.as_ref(),
                &pres,
            ));
        }
        bowline_core::commands::DeviceCommandAction::Accept => {
            lines.push(approval_line(
                "Trusted",
                output.local_device.as_ref(),
                &pres,
            ));
        }
        bowline_core::commands::DeviceCommandAction::Deny => {
            lines.push("Device request denied.".to_string());
        }
        bowline_core::commands::DeviceCommandAction::Revoke => {
            if let Some(device) = &output.revoked_device {
                lines.push(format!(
                    "{}  {}",
                    style::section("Revoked", &pres),
                    style::paint(&device.name, Role::Limited, &pres)
                ));
            } else {
                lines.push("Device revoked.".to_string());
            }
        }
    }
    append_next_actions(&mut lines, &output.next_actions);
    lines.push(String::new());
    lines.join("\n")
}

fn approval_line(
    label: &str,
    device: Option<&bowline_core::devices::DeviceRecord>,
    pres: &Presentation,
) -> String {
    match device {
        Some(device) => format!(
            "{}  {}",
            style::section(label, pres),
            style::paint(&device.name, Role::Ready, pres)
        ),
        None => format!("{label}."),
    }
}

pub(super) fn render_recovery_human(output: &recovery::RecoveryRunOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = Vec::new();
    match output.output.action {
        bowline_core::commands::RecoveryCommandAction::Status => {
            let lifecycle = output.output.recovery_key.lifecycle;
            lines.push(format!(
                "{}  {}",
                style::section("Recovery Key", &pres),
                style::paint(
                    recovery_lifecycle_label(lifecycle),
                    recovery_lifecycle_role(lifecycle),
                    &pres,
                )
            ));
        }
        bowline_core::commands::RecoveryCommandAction::Create => {
            lines.push(style::paint("Recovery Key created.", Role::Ready, &pres));
            append_recovery_words(&mut lines, output.generated_words.as_deref(), &pres);
        }
        bowline_core::commands::RecoveryCommandAction::Verify => {
            lines.push(style::paint("Recovery Key verified.", Role::Ready, &pres));
        }
        bowline_core::commands::RecoveryCommandAction::Rotate => {
            lines.push(style::paint("Recovery Key rotated.", Role::Ready, &pres));
            append_recovery_words(&mut lines, output.generated_words.as_deref(), &pres);
        }
        bowline_core::commands::RecoveryCommandAction::Revoke => {
            lines.push(style::paint("Recovery Key revoked.", Role::Limited, &pres));
        }
        bowline_core::commands::RecoveryCommandAction::Use => {
            lines.push(style::paint("Recovery Key used.", Role::Ready, &pres));
        }
    }
    append_next_actions(&mut lines, &output.output.next_actions);
    lines.push(String::new());
    lines.join("\n")
}

fn append_recovery_words(lines: &mut Vec<String>, words: Option<&str>, pres: &Presentation) {
    if let Some(words) = words {
        lines.push(style::section("Words", pres));
        lines.push(words.to_string());
        lines.push(style::paint(
            "This is the only time bowline prints these words.",
            Role::Attention,
            pres,
        ));
    }
}

pub(super) fn render_bootstrap_ssh_human(
    output: &bowline_core::commands::BootstrapSshCommandOutput,
) -> String {
    let pres = Presentation::detect(false);
    let (trust_label, trust_role) = if output.trusted {
        ("trusted", Role::Ready)
    } else {
        ("not trusted", Role::Attention)
    };
    let mut lines = vec![
        format!(
            "{}  {}",
            style::section("Bootstrap SSH", &pres),
            style::paint(
                &format!("{}:{}", output.host, output.root),
                Role::Strong,
                &pres
            )
        ),
        format!(
            "{}  {}",
            style::section("Trust", &pres),
            style::paint(trust_label, trust_role, &pres)
        ),
    ];
    lines.extend(output.steps.iter().map(|step| {
        format!(
            "  {}  {}",
            style::paint(&step.name, Role::Label, &pres),
            step.summary
        )
    }));
    append_next_actions(&mut lines, &output.next_actions);
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn recovery_lifecycle_label(
    lifecycle: bowline_core::devices::RecoveryKeyLifecycle,
) -> &'static str {
    match lifecycle {
        bowline_core::devices::RecoveryKeyLifecycle::Missing => "missing",
        bowline_core::devices::RecoveryKeyLifecycle::GeneratedUnverified => "generated, unverified",
        bowline_core::devices::RecoveryKeyLifecycle::Active => "active",
        bowline_core::devices::RecoveryKeyLifecycle::Rotated => "rotated",
        bowline_core::devices::RecoveryKeyLifecycle::Revoked => "revoked",
    }
}

fn recovery_lifecycle_role(lifecycle: bowline_core::devices::RecoveryKeyLifecycle) -> Role {
    match lifecycle {
        bowline_core::devices::RecoveryKeyLifecycle::Active => Role::Ready,
        bowline_core::devices::RecoveryKeyLifecycle::GeneratedUnverified => Role::Preparing,
        bowline_core::devices::RecoveryKeyLifecycle::Missing => Role::Attention,
        bowline_core::devices::RecoveryKeyLifecycle::Rotated => Role::Ready,
        bowline_core::devices::RecoveryKeyLifecycle::Revoked => Role::Label,
    }
}

pub(super) fn append_next_actions(lines: &mut Vec<String>, next_actions: &[SafeAction]) {
    let pres = Presentation::detect(false);
    lines.extend(style::next_actions_block(next_actions, &pres));
}
