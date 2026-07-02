use super::*;

pub(super) fn parse_login_command(args: &[String]) -> Command {
    let mut root = None;
    let mut headless = false;
    let mut no_poll = false;
    let mut index = 0_usize;

    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                let Some(value) = args.get(index + 1) else {
                    return command_usage_error(
                        CommandName::Login,
                        "usage_error",
                        "bowline login --root requires a path".to_string(),
                        vec![SafeAction {
                            label: "Log in and prepare ~/Code".to_string(),
                            command: Some("bowline login".to_string()),
                        }],
                    );
                };
                root = Some(value.to_string());
                index += 2;
            }
            "--headless" => {
                headless = true;
                index += 1;
            }
            "--no-poll" => {
                no_poll = true;
                index += 1;
            }
            flag if flag.starts_with("--") => {
                return command_usage_error(
                    CommandName::Login,
                    "usage_error",
                    format!("unknown bowline login option `{flag}`"),
                    vec![SafeAction {
                        label: "Start login".to_string(),
                        command: Some("bowline login".to_string()),
                    }],
                );
            }
            value => {
                return command_usage_error(
                    CommandName::Login,
                    "usage_error",
                    format!("unexpected bowline login argument `{value}`"),
                    vec![SafeAction {
                        label: "Start login".to_string(),
                        command: Some("bowline login".to_string()),
                    }],
                );
            }
        }
    }
    if root.is_none() {
        return command_usage_error(
            CommandName::Login,
            "usage_error",
            "bowline login requires --root <path>".to_string(),
            vec![SafeAction {
                label: "Log in and prepare ~/Code".to_string(),
                command: Some("bowline login --root ~/Code".to_string()),
            }],
        );
    }

    Command::Login(login::LoginArgs {
        root,
        no_poll,
        headless,
    })
}

pub(super) fn parse_approve_command(args: &[String]) -> Command {
    let mut selection = ParsedSelection::default();
    let mut selector = None;
    let mut yes = false;
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Approve, "approve", "--root");
                };
                selection.root = Some(value.clone());
                index += 2;
            }
            "--project" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Approve, "approve", "--project");
                };
                selection.project = Some(value.clone());
                index += 2;
            }
            "--request" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Approve, "approve", "--request");
                };
                if selector.is_some() {
                    return trust_selector_error(CommandName::Approve, "approve");
                }
                selector = Some(TrustRequestSelector::Request(value.clone()));
                index += 2;
            }
            "--code" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Approve, "approve", "--code");
                };
                if selector.is_some() {
                    return trust_selector_error(CommandName::Approve, "approve");
                }
                selector = Some(TrustRequestSelector::Code(value.clone()));
                index += 2;
            }
            "--yes" => {
                yes = true;
                index += 1;
            }
            flag if flag.starts_with("--") => {
                return unknown_option(CommandName::Approve, "approve", flag);
            }
            value => {
                return unexpected_argument(CommandName::Approve, "approve", value);
            }
        }
    }
    let Some(selector) = selector else {
        return trust_selector_error(CommandName::Approve, "approve");
    };
    let Some(selection) = selection.finish(CommandName::Approve, "approve") else {
        return missing_root(CommandName::Approve, "approve");
    };
    Command::Approve(ApproveArgs {
        selection,
        selector,
        yes,
    })
}

pub(super) fn parse_deny_command(args: &[String]) -> Command {
    let mut selection = ParsedSelection::default();
    let mut selector = None;
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Deny, "deny", "--root");
                };
                selection.root = Some(value.clone());
                index += 2;
            }
            "--project" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Deny, "deny", "--project");
                };
                selection.project = Some(value.clone());
                index += 2;
            }
            "--request" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Deny, "deny", "--request");
                };
                if selector.is_some() {
                    return trust_selector_error(CommandName::Deny, "deny");
                }
                selector = Some(TrustRequestSelector::Request(value.clone()));
                index += 2;
            }
            "--code" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Deny, "deny", "--code");
                };
                if selector.is_some() {
                    return trust_selector_error(CommandName::Deny, "deny");
                }
                selector = Some(TrustRequestSelector::Code(value.clone()));
                index += 2;
            }
            flag if flag.starts_with("--") => {
                return unknown_option(CommandName::Deny, "deny", flag);
            }
            value => {
                return unexpected_argument(CommandName::Deny, "deny", value);
            }
        }
    }
    let Some(selector) = selector else {
        return trust_selector_error(CommandName::Deny, "deny");
    };
    let Some(selection) = selection.finish(CommandName::Deny, "deny") else {
        return missing_root(CommandName::Deny, "deny");
    };
    Command::Deny(ApproveArgs {
        selection,
        selector,
        yes: true,
    })
}

pub(super) fn parse_revoke_command(args: &[String]) -> Command {
    let mut selection = ParsedSelection::default();
    let mut device_id = None;
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Revoke, "revoke", "--root");
                };
                selection.root = Some(value.clone());
                index += 2;
            }
            "--project" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Revoke, "revoke", "--project");
                };
                selection.project = Some(value.clone());
                index += 2;
            }
            "--device" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Revoke, "revoke", "--device");
                };
                device_id = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--") => {
                return unknown_option(CommandName::Revoke, "revoke", flag);
            }
            value => {
                return unexpected_argument(CommandName::Revoke, "revoke", value);
            }
        }
    }
    let Some(device_id) = device_id else {
        return command_usage_error(
            CommandName::Revoke,
            "usage_error",
            "bowline revoke requires --device <id>".to_string(),
            trust_usage_actions("revoke"),
        );
    };
    let Some(selection) = selection.finish(CommandName::Revoke, "revoke") else {
        return missing_root(CommandName::Revoke, "revoke");
    };
    Command::Revoke(RevokeArgs {
        selection,
        device_id,
    })
}

pub(super) fn parse_init_command(args: &[String]) -> Command {
    let mut root = None;
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Init, "init", "--root");
                };
                root = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--") => {
                return unknown_option(CommandName::Init, "init", flag);
            }
            value => return unexpected_argument(CommandName::Init, "init", value),
        }
    }
    let Some(root) = root else {
        return missing_root(CommandName::Init, "init");
    };
    Command::Init(InitArgs { root })
}

pub(super) fn parse_prewarm_command(args: &[String]) -> Command {
    let mut approve_setup = false;
    let mut project_path = None;

    for arg in args {
        match arg.as_str() {
            "--approve-setup" => approve_setup = true,
            flag if flag.starts_with("--") => {
                return usage_error(
                    CommandName::Prewarm,
                    format!("unknown bowline prewarm option `{flag}`"),
                );
            }
            value if project_path.is_none() => project_path = Some(value.to_string()),
            _ => {
                return usage_error(
                    CommandName::Prewarm,
                    "bowline prewarm accepts exactly one path",
                );
            }
        }
    }

    match project_path {
        Some(project_path) => Command::Prewarm(PrewarmArgs {
            project_path,
            approve_setup,
        }),
        None => usage_error(
            CommandName::Prewarm,
            "bowline prewarm requires a project path",
        ),
    }
}

pub(super) fn parse_setup_command(args: &[String]) -> Command {
    let mut yes = false;
    let mut project_path = None;

    for arg in args {
        match arg.as_str() {
            "--yes" => yes = true,
            flag if flag.starts_with("--") => {
                return command_usage_error(
                    CommandName::Setup,
                    "usage_error",
                    format!("unknown bowline setup option `{flag}`"),
                    vec![SafeAction {
                        label: "Prepare the current project".to_string(),
                        command: Some("bowline setup".to_string()),
                    }],
                );
            }
            value if project_path.is_none() => project_path = Some(value.to_string()),
            value => {
                return command_usage_error(
                    CommandName::Setup,
                    "usage_error",
                    format!("unexpected bowline setup argument `{value}`"),
                    vec![SafeAction {
                        label: "Prepare the current project".to_string(),
                        command: Some("bowline setup".to_string()),
                    }],
                );
            }
        }
    }

    Command::Setup(SetupArgs { project_path, yes })
}

pub(super) fn parse_status_command(args: &[String]) -> Command {
    let mut watch = false;
    let mut include_all = false;
    let mut selection = ParsedSelection::default();
    let mut index = 0_usize;

    while index < args.len() {
        match args[index].as_str() {
            "--watch" => {
                watch = true;
                index += 1;
            }
            "--all" => {
                include_all = true;
                index += 1;
            }
            "--root" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Status, "status", "--root");
                };
                selection.root = Some(value.clone());
                index += 2;
            }
            "--project" => {
                let Some(value) = args.get(index + 1) else {
                    return missing_value(CommandName::Status, "status", "--project");
                };
                selection.project = Some(value.clone());
                index += 2;
            }
            "--workspace" => return unknown_option(CommandName::Status, "status", "--workspace"),
            flag if flag.starts_with("--") => {
                return unknown_option(CommandName::Status, "status", flag);
            }
            value => return unexpected_argument(CommandName::Status, "status", value),
        }
    }
    let Some(selection) = selection.finish(CommandName::Status, "status") else {
        return missing_root(CommandName::Status, "status");
    };

    Command::Status(StatusArgs {
        selection,
        watch,
        include_all,
    })
}

pub(super) fn parse_actions_command(args: &[String]) -> Command {
    match parse_selection_only(CommandName::Actions, "actions", args) {
        Ok(selection) => Command::Actions(ActionsArgs { selection }),
        Err(error) => *error,
    }
}

pub(super) fn parse_tui_command(args: &[String]) -> Command {
    match parse_selection_only(CommandName::Tui, "tui", args) {
        Ok(selection) => Command::Tui(TuiArgs { selection }),
        Err(error) => *error,
    }
}

#[derive(Default)]
pub(super) struct ParsedSelection {
    pub(super) root: Option<String>,
    pub(super) project: Option<String>,
}

impl ParsedSelection {
    pub(super) fn finish(self, _command: CommandName, _name: &str) -> Option<WorkspaceSelection> {
        Some(WorkspaceSelection {
            root: self.root?,
            project: self.project,
        })
    }
}

pub(super) fn parse_selection_only(
    command: CommandName,
    name: &str,
    args: &[String],
) -> Result<WorkspaceSelection, Box<Command>> {
    let mut selection = ParsedSelection::default();
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(Box::new(missing_value(command, name, "--root")));
                };
                selection.root = Some(value.clone());
                index += 2;
            }
            "--project" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(Box::new(missing_value(command, name, "--project")));
                };
                selection.project = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--") => {
                return Err(Box::new(unknown_option(command, name, flag)));
            }
            value => return Err(Box::new(unexpected_argument(command, name, value))),
        }
    }
    selection
        .finish(command, name)
        .ok_or_else(|| Box::new(missing_root(command, name)))
}

pub(super) fn missing_root(command: CommandName, name: &str) -> Command {
    command_usage_error(
        command,
        "usage_error",
        format!("bowline {name} requires --root <path>"),
        trust_usage_actions(name),
    )
}

pub(super) fn missing_value(command: CommandName, name: &str, flag: &str) -> Command {
    command_usage_error(
        command,
        "usage_error",
        format!("bowline {name} {flag} requires a value"),
        trust_usage_actions(name),
    )
}

pub(super) fn unknown_option(command: CommandName, name: &str, flag: &str) -> Command {
    command_usage_error(
        command,
        "usage_error",
        format!("unknown bowline {name} option `{flag}`"),
        trust_usage_actions(name),
    )
}

pub(super) fn unexpected_argument(command: CommandName, name: &str, value: &str) -> Command {
    command_usage_error(
        command,
        "usage_error",
        format!("unexpected bowline {name} argument `{value}`"),
        trust_usage_actions(name),
    )
}

fn trust_selector_error(command: CommandName, name: &str) -> Command {
    command_usage_error(
        command,
        "usage_error",
        format!("bowline {name} requires exactly one of --request <id> or --code <matching-code>"),
        trust_usage_actions(name),
    )
}

fn trust_usage_actions(name: &str) -> Vec<SafeAction> {
    vec![SafeAction {
        label: format!("Run {name} with an explicit root"),
        command: Some(format!("bowline {name} --root ~/Code")),
    }]
}

pub(super) fn parse_search_command(args: &[String]) -> Command {
    let mut values = Vec::new();
    let mut limit = DEFAULT_EXPLORATION_LIMIT;
    let mut cursor = None;
    let mut path_prefix = None;
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--limit" => {
                let Some(value) = args.get(index + 1) else {
                    return exploration_usage_error(
                        CommandName::Search,
                        "bowline search --limit requires a number",
                    );
                };
                let Some(parsed) = parse_exploration_limit(value) else {
                    return exploration_usage_error(
                        CommandName::Search,
                        "bowline search --limit must be between 1 and 100",
                    );
                };
                limit = parsed;
                index += 2;
            }
            "--cursor" => {
                let Some(value) = args.get(index + 1) else {
                    return exploration_usage_error(
                        CommandName::Search,
                        "bowline search --cursor requires a cursor",
                    );
                };
                let Some(parsed) = parse_exploration_cursor(value) else {
                    return exploration_usage_error(
                        CommandName::Search,
                        "bowline search --cursor must be opaque cursor format v1:<offset> with offset at most 10000",
                    );
                };
                cursor = Some(parsed);
                index += 2;
            }
            "--path-prefix" => {
                let Some(value) = args.get(index + 1) else {
                    return exploration_usage_error(
                        CommandName::Search,
                        "bowline search --path-prefix requires a prefix",
                    );
                };
                path_prefix = Some(value.to_string());
                index += 2;
            }
            flag if flag.starts_with("--") => {
                return command_usage_error(
                    CommandName::Search,
                    "usage_error",
                    format!("unknown bowline search option `{flag}`"),
                    vec![SafeAction {
                        label: "Search a project".to_string(),
                        command: Some("bowline search <query> [path]".to_string()),
                    }],
                );
            }
            value => {
                values.push(value.to_string());
                index += 1;
            }
        }
    }
    match values.as_slice() {
        [query] => Command::Search(SearchArgs {
            query: query.to_string(),
            path: None,
            limit,
            cursor,
            path_prefix,
        }),
        [query, path] => Command::Search(SearchArgs {
            query: query.to_string(),
            path: Some(path.to_string()),
            limit,
            cursor,
            path_prefix,
        }),
        [] => command_usage_error(
            CommandName::Search,
            "usage_error",
            "bowline search requires a query".to_string(),
            vec![SafeAction {
                label: "Search a project".to_string(),
                command: Some("bowline search <query> [path]".to_string()),
            }],
        ),
        _ => command_usage_error(
            CommandName::Search,
            "usage_error",
            "bowline search accepts <query> and an optional path".to_string(),
            vec![SafeAction {
                label: "Search a project".to_string(),
                command: Some("bowline search <query> [path]".to_string()),
            }],
        ),
    }
}

pub(super) fn parse_symbols_command(args: &[String]) -> Command {
    let mut values = Vec::new();
    let mut limit = DEFAULT_EXPLORATION_LIMIT;
    let mut cursor = None;
    let mut path_prefix = None;
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--limit" => {
                let Some(value) = args.get(index + 1) else {
                    return exploration_usage_error(
                        CommandName::Symbols,
                        "bowline symbols --limit requires a number",
                    );
                };
                let Some(parsed) = parse_exploration_limit(value) else {
                    return exploration_usage_error(
                        CommandName::Symbols,
                        "bowline symbols --limit must be between 1 and 100",
                    );
                };
                limit = parsed;
                index += 2;
            }
            "--cursor" => {
                let Some(value) = args.get(index + 1) else {
                    return exploration_usage_error(
                        CommandName::Symbols,
                        "bowline symbols --cursor requires a cursor",
                    );
                };
                let Some(parsed) = parse_exploration_cursor(value) else {
                    return exploration_usage_error(
                        CommandName::Symbols,
                        "bowline symbols --cursor must be opaque cursor format v1:<offset> with offset at most 10000",
                    );
                };
                cursor = Some(parsed);
                index += 2;
            }
            "--path-prefix" => {
                let Some(value) = args.get(index + 1) else {
                    return exploration_usage_error(
                        CommandName::Symbols,
                        "bowline symbols --path-prefix requires a prefix",
                    );
                };
                path_prefix = Some(value.to_string());
                index += 2;
            }
            flag if flag.starts_with("--") => {
                return command_usage_error(
                    CommandName::Symbols,
                    "usage_error",
                    format!("unknown bowline symbols option `{flag}`"),
                    vec![SafeAction {
                        label: "Look up symbols".to_string(),
                        command: Some("bowline symbols <name> [path]".to_string()),
                    }],
                );
            }
            value => {
                values.push(value.to_string());
                index += 1;
            }
        }
    }
    match values.as_slice() {
        [query] => Command::Symbols(SymbolsArgs {
            query: query.to_string(),
            path: None,
            limit,
            cursor,
            path_prefix,
        }),
        [query, path] => Command::Symbols(SymbolsArgs {
            query: query.to_string(),
            path: Some(path.to_string()),
            limit,
            cursor,
            path_prefix,
        }),
        [] => command_usage_error(
            CommandName::Symbols,
            "usage_error",
            "bowline symbols requires a name".to_string(),
            vec![SafeAction {
                label: "Look up symbols".to_string(),
                command: Some("bowline symbols <name> [path]".to_string()),
            }],
        ),
        _ => command_usage_error(
            CommandName::Symbols,
            "usage_error",
            "bowline symbols accepts <name> and an optional path".to_string(),
            vec![SafeAction {
                label: "Look up symbols".to_string(),
                command: Some("bowline symbols <name> [path]".to_string()),
            }],
        ),
    }
}

pub(super) fn parse_exploration_limit(value: &str) -> Option<usize> {
    let limit = value.parse::<usize>().ok()?;
    (1..=MAX_EXPLORATION_LIMIT)
        .contains(&limit)
        .then_some(limit)
}

pub(super) fn parse_exploration_cursor(value: &str) -> Option<usize> {
    let offset = value.strip_prefix("v1:")?.parse::<usize>().ok()?;
    (offset <= MAX_EXPLORATION_CURSOR_OFFSET).then_some(offset)
}

pub(super) fn exploration_usage_error(command: CommandName, message: &str) -> Command {
    command_usage_error(
        command,
        "usage_error",
        message.to_string(),
        vec![SafeAction {
            label: "Inspect command help".to_string(),
            command: Some(format!(
                "bowline help {} --json",
                match command {
                    CommandName::Search => "search",
                    CommandName::Symbols => "symbols",
                    _ => "help",
                }
            )),
        }],
    )
}

pub(super) fn parse_explain_command(args: &[String]) -> Command {
    match args {
        [] => command_usage_error(
            CommandName::Explain,
            "usage_error",
            "bowline explain requires a path".to_string(),
            vec![SafeAction {
                label: "Explain a path".to_string(),
                command: Some("bowline explain <path>".to_string()),
            }],
        ),
        [flag, ..] if flag.starts_with("--") => command_usage_error(
            CommandName::Explain,
            "usage_error",
            format!("unknown bowline explain option `{flag}`"),
            vec![SafeAction {
                label: "Explain a path".to_string(),
                command: Some("bowline explain <path>".to_string()),
            }],
        ),
        [path] => Command::Explain(ExplainArgs {
            path: path.to_string(),
        }),
        _ => command_usage_error(
            CommandName::Explain,
            "usage_error",
            "bowline explain accepts exactly one path".to_string(),
            vec![SafeAction {
                label: "Explain a path".to_string(),
                command: Some("bowline explain <path>".to_string()),
            }],
        ),
    }
}
