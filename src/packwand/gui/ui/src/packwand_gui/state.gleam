import gleam/int
import gleam/list
import gleam/option.{type Option, None}
import gleam/string
import packwand_gui/manifest_form
import packwand_gui/model as domain

pub type View {
  Overview
  Exports
  Mods
  Changelog
  Logs
  Settings
}

pub type ModProgressStatus {
  ProgressPending
  ProgressPinned
  ProgressFailed
  ProgressSkipped
}

pub type ModProgress {
  ModProgress(name: String, status: ModProgressStatus, detail: String)
}

pub fn progress_status_label(status: ModProgressStatus) -> String {
  case status {
    ProgressPending -> "queued"
    ProgressPinned -> "pinned"
    ProgressFailed -> "failed"
    ProgressSkipped -> "skipped"
  }
}

pub type NewPack {
  NewPack(
    id: String,
    name: String,
    kind: String,
    loader: String,
    minecraft: String,
    version: String,
    description: String,
  )
}

pub type Model {
  Model(
    root: String,
    version: String,
    projects: List(domain.Project),
    features: List(domain.Feature),
    selected_id: String,
    selected_subdir: String,
    view: View,
    search: String,
    mods: List(domain.ModEntry),
    mod_slug: String,
    changelog: String,
    manifest: String,
    /// Parsed structured editor state; None when the manifest JSON could not
    /// be parsed (raw mode is the fallback).
    manifest_form: Option(manifest_form.ManifestForm),
    /// True renders typed controls, False the raw JSON textarea.
    manifest_structured: Bool,
    /// Stored newest-first so appending a line is O(1); reverse for display.
    logs: List(String),
    job_status: String,
    refresh_mods_after_job: Bool,
    icon_failed: Bool,
    new_pack: NewPack,
    notice: String,
    bump_version: String,
    bump_configs: Bool,
    mod_progress: List(ModProgress),
    mod_progress_in_block: Bool,
    /// The active "boot a pack for dev testing" session id, if any (see
    /// packwandrs.md — in-process launcher, no Go sidecar involved).
    launcher_session: Option(String),
    /// idle | installing | starting | started | exited | failed | cancelled
    /// ("started" persists through stdout/stderr until a terminal event)
    launcher_status: String,
    /// Newest-first, like `logs`.
    launcher_log: List(String),
    launcher_progress: Option(domain.LauncherProgress),
    /// When true, once the booted game's window appears it's repositioned
    /// (never resized) flush beside the Packwand window — "docked" rather
    /// than a separate, unrelated window. Windows-only for now; a no-op
    /// elsewhere (see `window_dock.rs`).
    dock_game_window: Bool,
    /// Real Microsoft account sign-in state (see `packwand-msa`). Signing
    /// in is optional — offline dev-testing boots work either way.
    auth_signed_in: Bool,
    auth_username: String,
    /// Transient status/error text: "Opening Microsoft sign-in...", a
    /// specific failure (not whitelisted yet, no Xbox account, etc.), or "".
    auth_status_text: String,
  )
}

pub type Msg {
  GotHealth(Result(domain.Health, domain.ApiError))
  GotProjects(Result(domain.ProjectIndex, domain.ApiError))
  GotFeatures(Result(domain.FeatureIndex, domain.ApiError))
  SelectProject(String)
  SelectSubdir(String)
  Navigate(View)
  SetSearch(String)
  SetModSlug(String)
  GotMods(Result(List(domain.ModEntry), domain.ApiError))
  GotChangelog(Result(domain.ContentResponse, domain.ApiError))
  GotManifest(Result(domain.ContentResponse, domain.ApiError))
  RunAction(domain.Action)
  GotAction(domain.Action, Result(domain.ActionResponse, domain.ApiError))
  RunWebview(provider: String, slug: String, file_id: String)
  WebviewStarted(Result(domain.ActionResponse, domain.ApiError))
  JobLine(String)
  JobFinished(String, String)
  SaveManifest
  SetManifest(String)
  SetManifestField(manifest_form.Field)
  SetManifestStructured(Bool)
  ManifestSaved(Result(Nil, domain.ApiError))
  CreateProject
  ProjectCreated(Result(domain.CreatedProject, domain.ApiError))
  SetNewPackID(String)
  SetNewPackName(String)
  SetNewPackType(String)
  SetNewPackLoader(String)
  SetNewPackMinecraft(String)
  SetNewPackVersion(String)
  SetNewPackDescription(String)
  CopyChangelog
  IconFailed
  SetBumpVersion(String)
  SetBumpConfigs(Bool)
  BootPack(path: String)
  SetDockGameWindow(Bool)
  PackBooted(Result(String, domain.ApiError))
  GotLauncherEvent(String)
  GotLauncherProgress(String)
  CancelBoot
  BootCancelled(Result(Nil, domain.ApiError))
  RequestAuthLogin
  AuthLoginStarted(Result(Nil, domain.ApiError))
  GotAuthEvent(String)
  RequestAuthLogout
  AuthLogoutDone(Result(Nil, domain.ApiError))
  GotAuthStatus(Result(domain.AuthStatus, domain.ApiError))
}

pub fn initial() -> Model {
  Model(
    root: "Loading repo...",
    version: "",
    projects: [],
    features: [],
    selected_id: "",
    selected_subdir: "",
    view: Overview,
    search: "",
    mods: [],
    mod_slug: "",
    changelog: "",
    manifest: "",
    manifest_form: None,
    manifest_structured: False,
    logs: [],
    job_status: "idle",
    refresh_mods_after_job: False,
    icon_failed: False,
    new_pack: NewPack("", "", "modpack", "fabric", "", "0.1.0", ""),
    notice: "",
    bump_version: "",
    bump_configs: False,
    mod_progress: [],
    mod_progress_in_block: False,
    launcher_session: None,
    launcher_status: "idle",
    launcher_log: [],
    launcher_progress: None,
    dock_game_window: True,
    auth_signed_in: False,
    auth_username: "",
    auth_status_text: "",
  )
}

pub fn selected_project(model: Model) -> Result(domain.Project, Nil) {
  list.find(model.projects, fn(project) { project.id == model.selected_id })
}

pub fn query_matches(query: String, text: String) -> Bool {
  let needle = query |> string.trim |> string.lowercase
  needle == "" || string.contains(string.lowercase(text), needle)
}

pub fn append_log(model: Model, line: String) -> Model {
  Model(..model, logs: [line, ..model.logs])
}

pub fn reset_progress(model: Model) -> Model {
  Model(..model, mod_progress: [], mod_progress_in_block: False)
}

/// Best-effort parse of packwand's `update --all` / `workspace update --all
/// --check` text output into a per-mod checklist. The CLI has no structured
/// event payload for this (see codex.md §2.2), so this matches the specific
/// line shapes cmd/update.go and workspace.go's CheckUpdatesInDir print:
/// "Updates found:" blocks of "<name>: <change>" lines, workspace check's
/// "  ~ <name>: <change>" lines, and the pinned/failed/no-updater lines.
pub fn record_progress_line(model: Model, raw_line: String) -> Model {
  let trimmed = string.trim(raw_line)
  case trimmed {
    "Updates found:" -> Model(..model, mod_progress_in_block: True)
    "All files are up to date!" | "Cancelled!" | "Files updated!" | "" ->
      Model(..model, mod_progress_in_block: False)
    _ ->
      case string.starts_with(trimmed, "dry-run:") {
        True -> Model(..model, mod_progress_in_block: False)
        False ->
          case string.starts_with(trimmed, "~ ") {
            True ->
              add_pending_pair(model, string.drop_start(trimmed, 2))
            False -> record_progress_prefixed(model, trimmed)
          }
      }
  }
}

fn record_progress_prefixed(model: Model, line: String) -> Model {
  let pinned_prefix = "Update skipped for pinned mod "
  let failed_prefix = "Failed to check updates for "
  let no_updater_prefix = "A supported update system for \""
  case string.starts_with(line, pinned_prefix) {
    True ->
      upsert_progress(
        model,
        string.drop_start(line, string.length(pinned_prefix)),
        ProgressPinned,
        "",
      )
    False ->
      case string.starts_with(line, failed_prefix) {
        True -> {
          let rest = string.drop_start(line, string.length(failed_prefix))
          case string.split_once(rest, ": ") {
            Ok(#(name, detail)) ->
              upsert_progress(model, name, ProgressFailed, detail)
            Error(_) -> upsert_progress(model, rest, ProgressFailed, "")
          }
        }
        False ->
          case string.starts_with(line, no_updater_prefix) {
            True -> {
              let rest = string.drop_start(line, string.length(no_updater_prefix))
              case string.split_once(rest, "\"") {
                Ok(#(name, _)) ->
                  upsert_progress(
                    model,
                    name,
                    ProgressSkipped,
                    "no supported update system",
                  )
                Error(_) -> model
              }
            }
            False ->
              case model.mod_progress_in_block {
                True -> add_pending_pair(model, line)
                False -> model
              }
          }
      }
  }
}

fn add_pending_pair(model: Model, line: String) -> Model {
  case string.split_once(line, ": ") {
    Ok(#(name, detail)) -> upsert_progress(model, name, ProgressPending, detail)
    Error(_) -> model
  }
}

fn upsert_progress(
  model: Model,
  name: String,
  status: ModProgressStatus,
  detail: String,
) -> Model {
  let name = string.trim(name)
  let entry = ModProgress(name:, status:, detail: string.trim(detail))
  let exists = list.any(model.mod_progress, fn(p) { p.name == name })
  let updated = case exists {
    True ->
      list.map(model.mod_progress, fn(p) {
        case p.name == name {
          True -> entry
          False -> p
        }
      })
    False -> list.append(model.mod_progress, [entry])
  }
  Model(..model, mod_progress: updated)
}

pub fn job_running(model: Model) -> Bool {
  model.job_status == "starting" || model.job_status == "running"
}

pub fn launcher_running(model: Model) -> Bool {
  model.launcher_status == "installing"
  || model.launcher_status == "starting"
  || model.launcher_status == "started"
}

pub fn append_launcher_log(model: Model, line: String) -> Model {
  Model(..model, launcher_log: [line, ..model.launcher_log])
}

/// Folds one decoded `LauncherEvent` into the model: updates status and
/// appends a human-readable log line. `kind` mirrors the Rust `LaunchEvent`
/// serde tag (`packwand-launch`'s `supervisor.rs`).
pub fn apply_launcher_event(model: Model, event: domain.LauncherEvent) -> Model {
  let #(status, line) = case event.kind {
    "starting" -> #("starting", "Starting...")
    "started" -> #(
      "started",
      "Started (pid " <> int.to_string(event.pid) <> ")",
    )
    "stdout" | "stderr" -> #(model.launcher_status, event.line)
    "exited" -> #(
      "exited",
      "Exited (code " <> int.to_string(event.code) <> ")",
    )
    "failed" -> #("failed", "Failed: " <> event.error)
    "cancelled" -> #("cancelled", "Cancelled")
    _ -> #(model.launcher_status, "")
  }
  let with_status = Model(..model, launcher_status: status)
  case line {
    "" -> with_status
    _ -> append_launcher_log(with_status, line)
  }
}

pub fn http_error(error: domain.ApiError) -> String {
  case error {
    domain.ApiError(message) -> message
    domain.DecodeError(message) ->
      "The Packwand API returned invalid data: " <> message
  }
}
