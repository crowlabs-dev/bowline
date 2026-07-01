use super::*;

pub(super) fn print_devices(args: devices::DevicesArgs, json: bool) -> ExitCode {
    let generated_at = generated_at();
    match devices::run(args, generated_at.clone()) {
        Ok(output) if json => {
            print_json(&output);
            ExitCode::SUCCESS
        }
        Ok(output) => {
            print!("{}", render_devices_human(&output));
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_runtime_error(CommandName::Devices, generated_at, &error, json);
            ExitCode::from(EXIT_RUNTIME)
        }
    }
}

pub(super) fn print_approve(args: ApproveArgs, json: bool) -> ExitCode {
    let generated_at = generated_at();
    let root = resolve_explicit_path(args.selection.root.clone());
    let workspace_id = match runtime::workspace_id_for_root(&root) {
        Ok(workspace_id) => workspace_id,
        Err(error) => {
            print_runtime_error(CommandName::Approve, generated_at, &error, json);
            return ExitCode::from(EXIT_RUNTIME);
        }
    };
    let request_id = match devices::request_id_for_selector(&workspace_id, &args.selector) {
        Ok(request_id) => request_id,
        Err(error) => {
            print_runtime_error(CommandName::Approve, generated_at, &error, json);
            return ExitCode::from(EXIT_RUNTIME);
        }
    };

    if !json && !args.yes && !confirm_return("Approve device request?") {
        return ExitCode::SUCCESS;
    }

    match devices::approve(workspace_id, request_id, generated_at.clone()) {
        Ok(mut output) if json => {
            output.command = CommandName::Approve;
            print_json(&output);
            ExitCode::SUCCESS
        }
        Ok(mut output) => {
            output.command = CommandName::Approve;
            print!("{}", render_devices_human(&output));
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_runtime_error(CommandName::Approve, generated_at, &error, json);
            ExitCode::from(EXIT_RUNTIME)
        }
    }
}

pub(super) fn print_deny(args: ApproveArgs, json: bool) -> ExitCode {
    let generated_at = generated_at();
    let root = resolve_explicit_path(args.selection.root);
    let workspace_id = match runtime::workspace_id_for_root(&root) {
        Ok(workspace_id) => workspace_id,
        Err(error) => {
            print_runtime_error(CommandName::Deny, generated_at, &error, json);
            return ExitCode::from(EXIT_RUNTIME);
        }
    };
    let request_id = match devices::request_id_for_selector(&workspace_id, &args.selector) {
        Ok(request_id) => request_id,
        Err(error) => {
            print_runtime_error(CommandName::Deny, generated_at, &error, json);
            return ExitCode::from(EXIT_RUNTIME);
        }
    };

    match devices::deny(workspace_id, request_id, generated_at.clone()) {
        Ok(mut output) if json => {
            output.command = CommandName::Deny;
            print_json(&output);
            ExitCode::SUCCESS
        }
        Ok(mut output) => {
            output.command = CommandName::Deny;
            print!("{}", render_devices_human(&output));
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_runtime_error(CommandName::Deny, generated_at, &error, json);
            ExitCode::from(EXIT_RUNTIME)
        }
    }
}

pub(super) fn print_revoke(args: RevokeArgs, json: bool) -> ExitCode {
    let generated_at = generated_at();
    let root = resolve_explicit_path(args.selection.root);
    let workspace_id = match runtime::workspace_id_for_root(&root) {
        Ok(workspace_id) => workspace_id,
        Err(error) => {
            print_runtime_error(CommandName::Revoke, generated_at, &error, json);
            return ExitCode::from(EXIT_RUNTIME);
        }
    };
    match devices::revoke(workspace_id, args.device_id, generated_at.clone()) {
        Ok(mut output) if json => {
            output.command = CommandName::Revoke;
            print_json(&output);
            ExitCode::SUCCESS
        }
        Ok(mut output) => {
            output.command = CommandName::Revoke;
            print!("{}", render_devices_human(&output));
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_runtime_error(CommandName::Revoke, generated_at, &error, json);
            ExitCode::from(EXIT_RUNTIME)
        }
    }
}

pub(super) fn print_recovery(args: recovery::RecoveryArgs, json: bool) -> ExitCode {
    let generated_at = generated_at();
    match recovery::run(args, generated_at.clone()) {
        Ok(output) if json => {
            print_json(&output.output);
            ExitCode::SUCCESS
        }
        Ok(output) => {
            print!("{}", render_recovery_human(&output));
            ExitCode::SUCCESS
        }
        Err(error) => {
            print_runtime_error(CommandName::Recover, generated_at, &error, json);
            ExitCode::from(EXIT_RUNTIME)
        }
    }
}

pub(super) fn print_resolve(args: resolve::ResolveArgs, json: bool, socket: &Path) -> ExitCode {
    let generated_at = generated_at();
    let use_tui = args.tui;
    let args = resolve::ResolveArgs {
        project_or_path: resolve_explicit_path(args.project_or_path),
        ..args
    };
    let output = resolve::run(args, generated_at);

    let command_failed = output.command_failed;
    if json {
        print_json(&output);
    } else if use_tui && io::stdin().is_terminal() && io::stdout().is_terminal() {
        let model = surface::tui::TuiModel::from_resolve(
            output.status.summary.clone(),
            surface::tui::TuiTone::from_status_label(output.status.level),
            output
                .available_actions
                .iter()
                .map(|action| surface::tui::TuiAction {
                    label: action.label.clone(),
                    command: action.command.clone(),
                    mutates: action
                        .command
                        .as_deref()
                        .map(|command| {
                            command.contains(" --accept ") || command.contains(" --reject ")
                        })
                        .unwrap_or(false),
                })
                .collect(),
            output
                .conflicts
                .iter()
                .map(|conflict| {
                    if conflict.contains_secrets {
                        format!(
                            "{}: secret-bearing conflict at {}",
                            conflict.id, conflict.bundle_path
                        )
                    } else {
                        format!("{}: {}", conflict.id, conflict.affected_files.join(", "))
                    }
                })
                .collect(),
        );
        match surface::tui::run_app(model) {
            Ok(Some(command)) => return run_confirmed_tui_command(&command, socket),
            Ok(None) => {}
            Err(error) => {
                print_runtime_error(
                    CommandName::Resolve,
                    output.generated_at.clone(),
                    &error.to_string(),
                    false,
                );
                return ExitCode::from(EXIT_RUNTIME);
            }
        }
    } else {
        let human = resolve::render_human(&output);
        print!("{human}");
    }

    if command_failed {
        return ExitCode::from(EXIT_RUNTIME);
    }

    ExitCode::SUCCESS
}
