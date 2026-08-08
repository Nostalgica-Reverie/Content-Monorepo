use packwand_ops::Workspace;
use packwand_providers::{
    BrowsePage, BrowseQuery, CurseForgeClient, ForgejoClient, GitHubClient, GitLabClient,
    ModrinthClient, ProviderBrowser, ProviderKind, ProviderResolver, ResolveRequest,
    ResolvedProject, UreqTransport, configured_api_key,
};

/// The CurseForge key to use for a request.
///
/// The frontend has no key of its own and passes `None`, which used to reach
/// CurseForge as an empty `X-API-Key` header and come back 403 — so every
/// CurseForge call from the GUI failed while the same call from the CLI
/// worked. `configured_api_key` is what the CLI has always used: the
/// `PACKWAND_CURSEFORGE_API_KEY` / `CURSEFORGE_API_KEY` / `CF_API_KEY`
/// environment variables, falling back to the bundled key.
fn curseforge_key(token: Option<String>) -> String {
    token
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(configured_api_key)
}
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
        ProviderKind::CurseForge => {
            CurseForgeClient::new(transport, curseforge_key(Some(token))).resolve(request)?
        }
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

/// Searches a provider's catalogue for the Browse page.
///
/// Only Modrinth and CurseForge are browsable: the repository providers
/// (Forgejo, GitHub, GitLab) have no catalogue to search — you reach a mod
/// there by knowing its repository — so asking them to browse is a request
/// error rather than an empty page.
#[tauri::command]
pub async fn providers_browse(
    provider: ProviderKind,
    query: BrowseQuery,
    token: Option<String>,
) -> CommandResult<BrowsePage> {
    tokio::task::spawn_blocking(move || {
        // Goes through `UreqTransport` like every other provider call, so the
        // per-host rate budget in `transport.rs` applies to search too. A
        // search box that bypassed it would be the fastest way to get the
        // whole app rate-limited.
        let transport = UreqTransport::new();
        match provider {
            ProviderKind::Modrinth => Ok(ModrinthClient::new(transport).search(&query)?),
            ProviderKind::CurseForge => {
                Ok(CurseForgeClient::new(transport, curseforge_key(token)).search(&query)?)
            }
            other => Err(SerializableError::new(
                "not_browsable",
                format!(
                    "{} has no catalogue to browse; add mods from it by repository instead",
                    other.name()
                ),
            )),
        }
    })
    .await
    .map_err(|error| SerializableError::new("task", error.to_string()))?
}

/// Hosts [`providers_open_page`] will hand to the system browser.
///
/// An allowlist rather than "open whatever you are given": the URLs reaching
/// this command come from a provider's API response, which is data from
/// outside the app. Opening an arbitrary URL — or an arbitrary *scheme* —
/// because a search result said so is how a listing turns into a way to launch
/// things on someone's machine.
const BROWSABLE_HOSTS: [&str; 5] = [
    "modrinth.com",
    "www.modrinth.com",
    "curseforge.com",
    "www.curseforge.com",
    "legacy.curseforge.com",
];

/// Opens a provider project page in the system browser.
///
/// The webview cannot navigate there itself: these sites refuse to be framed
/// (`X-Frame-Options: DENY` on Modrinth, `SAMEORIGIN` on CurseForge), and the
/// app ships no opener plugin, so an `<a target="_blank">` silently does
/// nothing. Handing the URL to the OS is the only route that works.
#[tauri::command]
pub async fn providers_open_page(url: String) -> CommandResult<()> {
    let parsed = url::Url::parse(&url)
        .map_err(|error| SerializableError::new("invalid_url", error.to_string()))?;
    if parsed.scheme() != "https" {
        return Err(SerializableError::new(
            "invalid_url",
            "only https project pages can be opened",
        ));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| SerializableError::new("invalid_url", "URL has no host"))?
        .to_ascii_lowercase();
    if !BROWSABLE_HOSTS.contains(&host.as_str()) {
        return Err(SerializableError::new(
            "invalid_url",
            format!("{host} is not a provider page host"),
        ));
    }
    tokio::task::spawn_blocking(move || open_in_browser(&url))
        .await
        .map_err(|error| SerializableError::new("task", error.to_string()))?
}

#[cfg(windows)]
fn open_in_browser(url: &str) -> CommandResult<()> {
    // `cmd /c start` rather than `ShellExecute`: no extra dependency, and the
    // empty title argument is required or `start` treats the URL as the title.
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
        .map_err(|error| SerializableError::new("open_failed", error.to_string()))
}

#[cfg(not(windows))]
fn open_in_browser(url: &str) -> CommandResult<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(opener)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| SerializableError::new("open_failed", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::BROWSABLE_HOSTS;

    /// A frontend that supplies no key must still reach CurseForge. This is
    /// the regression that made every CurseForge call from the GUI return 403
    /// while the identical call from the CLI succeeded.
    #[test]
    fn an_absent_or_blank_token_falls_back_to_the_configured_key() {
        let configured = packwand_providers::configured_api_key();
        assert!(!configured.is_empty(), "there is always a usable default");
        assert_eq!(super::curseforge_key(None), configured);
        assert_eq!(super::curseforge_key(Some(String::new())), configured);
        assert_eq!(super::curseforge_key(Some("   ".into())), configured);
    }

    #[test]
    fn an_explicit_token_wins_and_is_trimmed() {
        assert_eq!(super::curseforge_key(Some("  my-key  ".into())), "my-key");
    }

    /// The allowlist is the whole security property, so it must not silently
    /// grow to include a lookalike domain.
    #[test]
    fn only_provider_hosts_are_browsable() {
        assert!(BROWSABLE_HOSTS.contains(&"modrinth.com"));
        assert!(BROWSABLE_HOSTS.contains(&"legacy.curseforge.com"));
        assert!(!BROWSABLE_HOSTS.contains(&"modrinth.com.evil.test"));
        assert!(!BROWSABLE_HOSTS.contains(&"localhost"));
        for host in BROWSABLE_HOSTS {
            assert_eq!(host, host.to_ascii_lowercase(), "matching lowercases first");
        }
    }
}

/// One project, with its description already rendered to safe HTML.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPage {
    #[serde(flatten)]
    pub detail: packwand_providers::BrowseDetail,
    /// The description as sanitized HTML, ready for `v-html`.
    ///
    /// Rendered here rather than in the webview so unsanitized third-party
    /// markup never exists in the DOM — see [`crate::commands::richtext`].
    pub body_html: String,
}

/// Fetches one project so it can be read inside the app.
#[tauri::command]
pub async fn providers_project(
    provider: ProviderKind,
    id: String,
    token: Option<String>,
) -> CommandResult<ProjectPage> {
    tokio::task::spawn_blocking(move || {
        let transport = UreqTransport::new();
        let detail = match provider {
            ProviderKind::Modrinth => ModrinthClient::new(transport).project(&id)?,
            ProviderKind::CurseForge => {
                CurseForgeClient::new(transport, curseforge_key(token)).project(&id)?
            }
            other => {
                return Err(SerializableError::new(
                    "not_browsable",
                    format!("{} has no project pages to read", other.name()),
                ));
            }
        };
        let body_html = crate::commands::richtext::render(&detail.body, detail.body_format);
        Ok(ProjectPage { detail, body_html })
    })
    .await
    .map_err(|error| SerializableError::new("task", error.to_string()))?
}
