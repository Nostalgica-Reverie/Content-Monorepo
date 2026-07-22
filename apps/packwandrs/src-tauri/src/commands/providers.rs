use packwand_ops::Workspace;
use packwand_providers::{
    CurseForgeClient, ForgejoClient, GitHubClient, GitLabClient, ModrinthClient, ProviderKind,
    ProviderResolver, ResolveRequest, ResolvedProject, UreqTransport,
};
use tauri::{AppHandle, State};

use crate::commands::packs::pack_root;
use crate::error::{CommandResult, SerializableError};
use crate::events::emit_packs_changed;
use crate::state::AppState;

#[tauri::command]
pub async fn providers_resolve(
    provider: ProviderKind,
    request: ResolveRequest,
    token: Option<String>,
    instance: Option<String>,
) -> CommandResult<ResolvedProject> {
    tokio::task::spawn_blocking(move || resolve(provider, &request, token, instance))
        .await
        .map_err(|error| SerializableError::new("task", error.to_string()))?
}

#[tauri::command]
pub async fn providers_add(
    id: String,
    provider: ProviderKind,
    request: ResolveRequest,
    token: Option<String>,
    instance: Option<String>,
    replace: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<String> {
    let root = pack_root(&state.workspace()?, &id)?;
    let metadata_path = tokio::task::spawn_blocking(move || {
        let resolved = resolve(provider, &request, token, instance)?;
        let path = resolved.metadata_path();
        Workspace::open(root)?.add_resolved(resolved, replace)?;
        Ok::<_, SerializableError>(path)
    })
    .await
    .map_err(|error| SerializableError::new("task", error.to_string()))??;
    emit_packs_changed(&app)?;
    Ok(metadata_path)
}

fn resolve(
    provider: ProviderKind,
    request: &ResolveRequest,
    token: Option<String>,
    instance: Option<String>,
) -> CommandResult<ResolvedProject> {
    let token = token.unwrap_or_default();
    let transport = UreqTransport::new();
    Ok(match provider {
        ProviderKind::Modrinth => ModrinthClient::new(transport).resolve(request)?,
        ProviderKind::CurseForge => CurseForgeClient::new(transport, token).resolve(request)?,
        ProviderKind::GitHub => GitHubClient::new(transport, token).resolve(request)?,
        ProviderKind::Forgejo => match instance {
            Some(instance) => {
                ForgejoClient::for_instance(transport, instance, token).resolve(request)?
            }
            None => ForgejoClient::new(transport, token).resolve(request)?,
        },
        ProviderKind::GitLab => match instance {
            Some(instance) => {
                GitLabClient::for_instance(transport, instance, token).resolve(request)?
            }
            None => GitLabClient::new(transport, token).resolve(request)?,
        },
    })
}
