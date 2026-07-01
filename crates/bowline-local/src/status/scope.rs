use super::*;

#[derive(Debug, Clone)]
pub(super) struct ResolvedScope {
    pub(super) workspace_id: Option<WorkspaceId>,
    pub(super) project_id: Option<ProjectId>,
    pub(super) project_path: Option<String>,
}

impl ResolvedScope {
    pub(super) fn event_query(&self, limit: u32) -> EventQuery {
        EventQuery {
            workspace_id: self.workspace_id.clone(),
            project_id: self.project_id.clone(),
            path_prefix: self.project_path.clone(),
            limit,
        }
    }
}

pub(super) fn resolve_scope(
    store: &MetadataStore,
    requested_path: Option<&str>,
    workspace_scope: bool,
) -> Result<ResolvedScope, LocalStatusError> {
    let workspace = workspace_for_requested_path(store, requested_path)?;
    let workspace_id = workspace.as_ref().map(|record| record.id.clone());
    let project = if workspace_scope {
        None
    } else if let (Some(workspace), Some(path)) = (workspace.as_ref(), requested_path) {
        store.project_by_path(&workspace.id, path)?
    } else {
        None
    };

    Ok(ResolvedScope {
        workspace_id,
        project_id: project.as_ref().map(|record| record.id.clone()),
        project_path: project.map(|record| record.path),
    })
}

pub(super) fn workspace_for_requested_path(
    store: &MetadataStore,
    requested_path: Option<&str>,
) -> Result<Option<WorkspaceRecord>, LocalStatusError> {
    let Some(path) = requested_path else {
        return store.current_workspace().map_err(Into::into);
    };
    let workspace = store.workspace_by_path(path)?;
    if workspace.is_some() || path == "~" || path.starts_with("~/") || Path::new(path).is_absolute()
    {
        return Ok(workspace);
    }
    store.current_workspace().map_err(Into::into)
}
