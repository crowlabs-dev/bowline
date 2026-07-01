use std::path::PathBuf;

use bowline_core::commands::{
    CommandName, WorkCleanupCommandOutput, WorkDiffCommandOutput, WorkLifecycleCommandOutput,
    WorkListCommandOutput, WorkonCommandOutput,
};
use bowline_core::ids::DeviceId;
use bowline_local::work_views::{
    WorkCleanupOptions, WorkListOptions, WorkSelectorOptions, WorkViewError, WorkonOptions,
    accept_work_view, cleanup_work_views, create_work_view, diff_work_view, discard_work_view,
    list_work_views, restore_work_view,
};

use crate::surface::style::{self, Presentation, Role};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkonArgs {
    pub project_path: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkListArgs {
    pub include_hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkSelectorArgs {
    pub selector: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkCleanupArgs {
    pub apply: bool,
}

pub fn run_workon(
    args: WorkonArgs,
    db_path: Option<PathBuf>,
    owner_device_id: DeviceId,
    generated_at: String,
) -> Result<WorkonCommandOutput, WorkViewError> {
    create_work_view(WorkonOptions {
        db_path,
        project_path: args.project_path,
        name: args.name,
        owner_device_id: Some(owner_device_id),
        generated_at,
    })
}

pub fn run_list(
    args: WorkListArgs,
    db_path: Option<PathBuf>,
    current_device_id: DeviceId,
    generated_at: String,
) -> Result<WorkListCommandOutput, WorkViewError> {
    list_work_views(WorkListOptions {
        db_path,
        include_hidden: args.include_hidden,
        current_device_id: Some(current_device_id),
        generated_at,
    })
}

pub fn run_diff(
    args: WorkSelectorArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<WorkDiffCommandOutput, WorkViewError> {
    diff_work_view(WorkSelectorOptions {
        db_path,
        selector: args.selector,
        generated_at,
    })
}

pub fn run_lifecycle(
    command: CommandName,
    args: WorkSelectorArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<WorkLifecycleCommandOutput, WorkViewError> {
    let options = WorkSelectorOptions {
        db_path,
        selector: args.selector,
        generated_at,
    };
    match command {
        CommandName::Accept => accept_work_view(options),
        CommandName::Discard => discard_work_view(options),
        CommandName::Restore => restore_work_view(options),
        _ => unreachable!("unsupported work lifecycle command"),
    }
}

pub fn run_cleanup(
    args: WorkCleanupArgs,
    db_path: Option<PathBuf>,
    generated_at: String,
) -> Result<WorkCleanupCommandOutput, WorkViewError> {
    cleanup_work_views(WorkCleanupOptions {
        db_path,
        apply: args.apply,
        generated_at,
    })
}

pub fn render_workon_human(output: &WorkonCommandOutput) -> String {
    let pres = Presentation::detect(false);
    format!(
        "{}  {}\n{}  {}\n{}  {}\n\n",
        style::section("Work view", &pres),
        style::paint(&output.work_view.name, Role::Strong, &pres),
        style::section("Path", &pres),
        output.work_view.visible_path,
        style::section("State", &pres),
        style::paint("active", Role::Ready, &pres),
    )
}

pub fn render_list_human(output: &WorkListCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = vec![format!(
        "{}  {}",
        style::section("Work views", &pres),
        output.work_views.len()
    )];
    lines.extend(output.work_views.iter().map(|view| {
        format!(
            "  {}  {}  {}",
            style::paint(&view.name, Role::Strong, &pres),
            style::paint(&view.visible_path, Role::Label, &pres),
            style::paint(&style::kebab(&view.lifecycle), Role::Label, &pres),
        )
    }));
    lines.push(String::new());
    lines.join("\n")
}

pub fn render_diff_human(output: &WorkDiffCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = vec![format!(
        "{}  {}",
        style::section("Work view", &pres),
        style::paint(&output.work_view.name, Role::Strong, &pres)
    )];
    if output.changes.is_empty() {
        lines.push(format!(
            "  {}",
            style::paint("No local changes recorded.", Role::Label, &pres)
        ));
    } else {
        lines.extend(output.changes.iter().map(|change| {
            let redacted = if change.contains_secrets {
                style::paint("  (redacted)", Role::Label, &pres)
            } else {
                String::new()
            };
            format!(
                "  {} {}{redacted}",
                style::paint(&style::kebab(&change.kind), Role::Label, &pres),
                change.path,
            )
        }));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub fn render_lifecycle_human(output: &WorkLifecycleCommandOutput) -> String {
    let pres = Presentation::detect(false);
    format!(
        "{}  {}\n{}  {}\n\n",
        style::section("Work view", &pres),
        style::paint(&output.work_view.name, Role::Strong, &pres),
        style::section("State", &pres),
        style::kebab(&output.work_view.lifecycle),
    )
}

pub fn render_cleanup_human(output: &WorkCleanupCommandOutput) -> String {
    let pres = Presentation::detect(false);
    let mut lines = vec![format!(
        "{}  {}",
        style::section("Cleanup candidates", &pres),
        output.previewed_paths.len()
    )];
    if output.deleted_paths.is_empty() {
        lines.extend(
            output
                .previewed_paths
                .iter()
                .map(|path| format!("  {}", style::paint(path, Role::Label, &pres))),
        );
    } else {
        lines.extend(
            output
                .deleted_paths
                .iter()
                .map(|path| format!("  {} {path}", style::paint("deleted", Role::Limited, &pres))),
        );
    }
    lines.push(String::new());
    lines.join("\n")
}
