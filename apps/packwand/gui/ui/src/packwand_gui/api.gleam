import gleam/dynamic/decode
import gleam/json
import gleam/string
import gleam/uri
import lustre/effect.{type Effect}
import packwand_gui/model as domain

@external(javascript, "./ffi.mjs", "requestJson")
fn request_json(
  method: String,
  url: String,
  body: String,
  success: fn(String) -> Nil,
  failure: fn(String) -> Nil,
) -> Nil

pub fn health(
  to_msg: fn(Result(domain.Health, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request("GET", "/api/v1/version", "", health_decoder(), to_msg)
}

pub fn projects(
  to_msg: fn(Result(domain.ProjectIndex, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request("GET", "/api/v1/packs", "", project_index_decoder(), to_msg)
}

pub fn features(
  to_msg: fn(Result(domain.FeatureIndex, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request("GET", "/api/v1/capabilities", "", feature_index_decoder(), to_msg)
}

pub fn mods(
  path: String,
  to_msg: fn(Result(List(domain.ModEntry), domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "GET",
    "/api/v1/mods?subdir=" <> uri.percent_encode(path),
    "",
    decode.list(mod_decoder()),
    to_msg,
  )
}

pub fn changelog(
  id: String,
  to_msg: fn(Result(domain.ContentResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  content("/api/v1/packs/" <> uri.percent_encode(id) <> "/changelog", to_msg)
}

pub fn manifest(
  id: String,
  to_msg: fn(Result(domain.ContentResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  content("/api/v1/packs/" <> uri.percent_encode(id) <> "/manifest", to_msg)
}

fn content(
  url: String,
  to_msg: fn(Result(domain.ContentResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request("GET", url, "", content_decoder(), to_msg)
}

pub fn save_manifest(
  id: String,
  content: String,
  to_msg: fn(Result(Nil, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "PUT",
    "/api/v1/packs/" <> uri.percent_encode(id) <> "/manifest",
    json.object([#("content", json.string(content))]) |> json.to_string,
    decode.success(Nil),
    to_msg,
  )
}

pub fn save_changelog(
  id: String,
  content: String,
  to_msg: fn(Result(Nil, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "PUT",
    "/api/v1/packs/" <> uri.percent_encode(id) <> "/changelog",
    json.object([#("content", json.string(content))]) |> json.to_string,
    decode.success(Nil),
    to_msg,
  )
}

pub fn create_project(
  id: String,
  name: String,
  kind: String,
  loader: String,
  minecraft: String,
  version: String,
  description: String,
  to_msg: fn(Result(domain.CreatedProject, domain.ApiError)) -> msg,
) -> Effect(msg) {
  let body =
    json.object([
      #("id", json.string(id)),
      #("name", json.string(name)),
      #("type", json.string(kind)),
      #("loader", json.string(loader)),
      #("mc_version", json.string(minecraft)),
      #("version", json.string(version)),
      #("description", json.string(description)),
    ])
  request(
    "POST",
    "/api/v1/packs",
    json.to_string(body),
    created_project_decoder(),
    to_msg,
  )
}

/// Opens the native mod browser webview (apps/mod-browser-webview) for the
/// given mod on the given provider ("curseforge" or "modrinth"), bridged by
/// the Go server; download/navigation events stream through the returned
/// job's event feed.
pub fn webview_fetch(
  provider: String,
  slug: String,
  file_id: String,
  to_msg: fn(Result(domain.ActionResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  let body =
    json.object([
      #("provider", json.string(provider)),
      #(
        "files",
        json.array(
          [
            json.object([
              #("file_id", json.string(file_id)),
              #("slug", json.string(slug)),
            ]),
          ],
          fn(file) { file },
        ),
      ),
    ])
  request(
    "POST",
    "/api/v1/webview/open",
    json.to_string(body),
    action_response_decoder(),
    to_msg,
  )
}

pub fn action(
  action: domain.Action,
  to_msg: fn(Result(domain.ActionResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  let body =
    json.object([
      #("action", json.string(domain.action_name(action))),
      #("subdir", json.string(domain.action_subdir(action))),
      #("slug", json.string(domain.action_slug(action))),
      #("dry_run", json.bool(domain.action_dry_run(action))),
      #("version", json.string(domain.action_version(action))),
      #("configs", json.bool(domain.action_configs(action))),
      #("side", json.string(domain.action_side(action))),
    ])
  request(
    "POST",
    "/api/v1/actions",
    json.to_string(body),
    action_response_decoder(),
    to_msg,
  )
}

// — IDE editor services (IDE.md §3–§4) —

fn subdir_url(id: String, sub: String, tail: String) -> String {
  "/api/v1/packs/"
  <> uri.percent_encode(id)
  <> "/subdirs/"
  <> uri.percent_encode(sub)
  <> tail
}

pub fn editor_tree(
  id: String,
  sub: String,
  to_msg: fn(Result(List(domain.TreeGroup), domain.ApiError)) -> msg,
) -> Effect(msg) {
  request("GET", subdir_url(id, sub, "/tree"), "", tree_decoder(), to_msg)
}

pub fn read_editor_file(
  id: String,
  sub: String,
  path: String,
  to_msg: fn(Result(domain.ContentResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "GET",
    subdir_url(id, sub, "/file?path=" <> uri.percent_encode(path)),
    "",
    content_decoder(),
    to_msg,
  )
}

pub fn save_editor_file(
  id: String,
  sub: String,
  path: String,
  content: String,
  to_msg: fn(Result(Nil, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "PUT",
    subdir_url(id, sub, "/file"),
    json.object([
      #("path", json.string(path)),
      #("content", json.string(content)),
    ])
      |> json.to_string,
    decode.success(Nil),
    to_msg,
  )
}

/// Paste a new file into the pack, or (when `from_sub` is set) duplicate a
/// file from a sibling subdir of the same pack (IDE.md §4.3).
pub fn create_editor_file(
  id: String,
  sub: String,
  path: String,
  content: String,
  from_sub: String,
  from_path: String,
  to_msg: fn(Result(domain.CreatedFile, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "POST",
    subdir_url(id, sub, "/files"),
    json.object([
      #("path", json.string(path)),
      #("content", json.string(content)),
      #("from_sub", json.string(from_sub)),
      #("from_path", json.string(from_path)),
    ])
      |> json.to_string,
    created_file_decoder(),
    to_msg,
  )
}

/// Check an unsaved editor buffer for structural and reference problems
/// (IDE.md §4.1).
pub fn check_buffer(
  id: String,
  sub: String,
  file: String,
  content: String,
  to_msg: fn(Result(domain.CheckResult, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "POST",
    subdir_url(id, sub, "/check"),
    json.object([
      #("file", json.string(file)),
      #("content", json.string(content)),
    ])
      |> json.to_string,
    check_decoder(),
    to_msg,
  )
}

/// Registry-driven completion (IDE.md §4.2): matching entries for the token
/// being typed, from the subdir's registry of the given kind.
pub fn complete(
  id: String,
  sub: String,
  kind: String,
  query: String,
  to_msg: fn(Result(List(domain.CompletionItem), domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "GET",
    subdir_url(
      id,
      sub,
      "/registry/"
        <> uri.percent_encode(kind)
        <> "/complete?q="
        <> uri.percent_encode(query),
    ),
    "",
    completions_decoder(),
    to_msg,
  )
}

/// Start the pre-launch validation gate as a job (IDE.md §4.4).
pub fn preflight(
  id: String,
  sub: String,
  to_msg: fn(Result(domain.ActionResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "POST",
    subdir_url(id, sub, "/preflight"),
    "",
    action_response_decoder(),
    to_msg,
  )
}

/// Start the CI-equivalent local validation stages as an SSE job (IDE.md §6).
pub fn local_ci(
  id: String,
  sub: String,
  to_msg: fn(Result(domain.ActionResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "POST",
    subdir_url(id, sub, "/ci-local"),
    "",
    action_response_decoder(),
    to_msg,
  )
}

/// Fetch the structured preflight report from a finished job.
pub fn preflight_result(
  job_id: String,
  to_msg: fn(Result(domain.PreflightResult, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "GET",
    "/api/v1/jobs/" <> uri.percent_encode(job_id),
    "",
    job_preflight_decoder(),
    to_msg,
  )
}

fn request(
  method: String,
  url: String,
  body: String,
  decoder: decode.Decoder(value),
  to_msg: fn(Result(value, domain.ApiError)) -> msg,
) -> Effect(msg) {
  effect.from(fn(dispatch) {
    request_json(
      method,
      url,
      body,
      fn(response) {
        let result = case json.parse(response, decoder) {
          Ok(value) -> Ok(value)
          Error(error) -> Error(domain.DecodeError(string.inspect(error)))
        }
        dispatch(to_msg(result))
      },
      fn(error) { dispatch(to_msg(Error(domain.ApiError(error)))) },
    )
  })
}

fn health_decoder() {
  use root <- decode.optional_field("root", "", decode.string)
  use version <- decode.optional_field("version", "", decode.string)
  decode.success(domain.Health(root:, version:))
}

fn project_index_decoder() {
  use projects <- decode.optional_field(
    "projects",
    [],
    decode.list(project_decoder()),
  )
  decode.success(domain.ProjectIndex(projects:))
}

fn feature_index_decoder() {
  use packwand_version <- decode.optional_field(
    "packwand_version",
    "",
    decode.string,
  )
  use features <- decode.optional_field(
    "features",
    [],
    decode.list(feature_decoder()),
  )
  decode.success(domain.FeatureIndex(packwand_version:, features:))
}

fn feature_decoder() {
  use command <- decode.field("command", decode.string)
  use usage <- decode.optional_field("use", "", decode.string)
  use summary <- decode.optional_field("summary", "", decode.string)
  use group <- decode.optional_field("group", "", decode.string)
  use runnable <- decode.optional_field("runnable", False, decode.bool)
  use gui_status <- decode.optional_field(
    "gui_status",
    "cli-only",
    decode.string,
  )
  use gui_action <- decode.optional_field("gui_action", "", decode.string)
  use scope <- decode.optional_field("scope", "", decode.string)
  use destructive <- decode.optional_field("destructive", False, decode.bool)
  decode.success(domain.Feature(
    command:,
    usage:,
    summary:,
    group:,
    runnable:,
    gui_status:,
    gui_action:,
    scope:,
    destructive:,
  ))
}

fn project_decoder() {
  use id <- decode.field("id", decode.string)
  use name <- decode.optional_field("name", "", decode.string)
  use kind <- decode.optional_field("type", "", decode.string)
  use dir <- decode.optional_field("dir", "", decode.string)
  use manifest_path <- decode.optional_field("manifest_path", "", decode.string)
  use version <- decode.optional_field("version", "", decode.string)
  use minecraft <- decode.optional_field("mc_version", "", decode.string)
  use loader <- decode.optional_field("loader", "", decode.string)
  use release_type <- decode.optional_field("release_type", "", decode.string)
  use lifecycle <- decode.optional_field("lifecycle", "", decode.string)
  use role <- decode.optional_field("role", "", decode.string)
  use auto_update <- decode.optional_field("auto_update", False, decode.bool)
  use modrinth_id <- decode.optional_field("modrinth_id", "", decode.string)
  use curseforge_id <- decode.optional_field("curseforge_id", "", decode.string)
  use github_id <- decode.optional_field("github_id", "", decode.string)
  use gitea_id <- decode.optional_field("gitea_id", "", decode.string)
  use gitlab_id <- decode.optional_field("gitlab_id", "", decode.string)
  use docs_path <- decode.optional_field("docs_path", "", decode.string)
  use variants <- decode.optional_field(
    "variants",
    [],
    decode.list(variant_decoder()),
  )
  use subdirs <- decode.optional_field(
    "subdirs",
    [],
    decode.list(subdir_decoder()),
  )
  decode.success(domain.Project(
    id:,
    name:,
    kind:,
    dir:,
    manifest_path:,
    version:,
    minecraft:,
    loader:,
    release_type:,
    lifecycle:,
    role:,
    auto_update:,
    modrinth_id:,
    curseforge_id:,
    github_id:,
    gitea_id:,
    gitlab_id:,
    docs_path:,
    variants:,
    subdirs:,
  ))
}

fn variant_decoder() {
  use id <- decode.optional_field("id", "", decode.string)
  use minecraft <- decode.optional_field("mc_version", "", decode.string)
  use loader <- decode.optional_field("loader", "", decode.string)
  use version <- decode.optional_field("version", "", decode.string)
  decode.success(domain.Variant(id:, minecraft:, loader:, version:))
}

fn subdir_decoder() {
  use key <- decode.optional_field("key", "", decode.string)
  use path <- decode.field("path", decode.string)
  use platform <- decode.optional_field("platform", "", decode.string)
  use mod_count <- decode.optional_field("mod_count", 0, decode.int)
  use has_index <- decode.optional_field("has_index", False, decode.bool)
  use has_pack <- decode.optional_field("has_pack", False, decode.bool)
  decode.success(domain.Subdir(
    key:,
    path:,
    platform:,
    mod_count:,
    has_index:,
    has_pack:,
  ))
}

fn mod_decoder() {
  use slug <- decode.field("slug", decode.string)
  use name <- decode.optional_field("name", "", decode.string)
  use filename <- decode.optional_field("filename", "", decode.string)
  use side <- decode.optional_field("side", "", decode.string)
  use pin <- decode.optional_field("pin", False, decode.bool)
  use platform <- decode.optional_field("platform", "", decode.string)
  use version_id <- decode.optional_field("version_id", "", decode.string)
  decode.success(domain.ModEntry(
    slug:,
    name:,
    filename:,
    side:,
    pin:,
    platform:,
    version_id:,
  ))
}

fn content_decoder() {
  use path <- decode.optional_field("path", "", decode.string)
  use content <- decode.optional_field("content", "", decode.string)
  decode.success(domain.ContentResponse(path:, content:))
}

fn action_response_decoder() {
  use job_id <- decode.field("job_id", decode.string)
  decode.success(domain.ActionResponse(job_id:))
}

fn created_project_decoder() {
  use id <- decode.field("id", decode.string)
  use dir <- decode.field("dir", decode.string)
  decode.success(domain.CreatedProject(id:, dir:))
}

fn tree_decoder() {
  use groups <- decode.optional_field(
    "groups",
    [],
    decode.list(tree_group_decoder()),
  )
  decode.success(groups)
}

fn tree_group_decoder() {
  use name <- decode.optional_field("name", "", decode.string)
  use files <- decode.optional_field(
    "files",
    [],
    decode.list(tree_file_decoder()),
  )
  decode.success(domain.TreeGroup(name:, files:))
}

fn tree_file_decoder() {
  use path <- decode.field("path", decode.string)
  use ref_id <- decode.optional_field("ref_id", "", decode.string)
  use kind <- decode.optional_field("kind", "", decode.string)
  use owner <- decode.optional_field("owner", "", decode.string)
  use editable <- decode.optional_field("editable", False, decode.bool)
  decode.success(domain.TreeFile(path:, ref_id:, kind:, owner:, editable:))
}

fn check_decoder() {
  use valid <- decode.optional_field("valid", False, decode.bool)
  use diagnostics <- decode.optional_field(
    "diagnostics",
    [],
    decode.list(diagnostic_decoder()),
  )
  decode.success(domain.CheckResult(valid:, diagnostics:))
}

fn diagnostic_decoder() {
  use severity <- decode.optional_field("severity", "error", decode.string)
  use line <- decode.optional_field("line", 1, decode.int)
  use col <- decode.optional_field("col", 1, decode.int)
  use message <- decode.optional_field("message", "", decode.string)
  use code <- decode.optional_field("code", "", decode.string)
  decode.success(domain.Diagnostic(severity:, line:, col:, message:, code:))
}

fn completions_decoder() {
  use items <- decode.optional_field(
    "items",
    [],
    decode.list(completion_decoder()),
  )
  decode.success(items)
}

fn completion_decoder() {
  use id <- decode.field("id", decode.string)
  use kind <- decode.optional_field("kind", "", decode.string)
  decode.success(domain.CompletionItem(id:, kind:))
}

fn created_file_decoder() {
  use path <- decode.optional_field("path", "", decode.string)
  decode.success(domain.CreatedFile(path:))
}

fn job_preflight_decoder() {
  use result <- decode.optional_field(
    "result",
    domain.PreflightResult(ok: False, errors: 0, warnings: 0, steps: []),
    preflight_result_decoder(),
  )
  decode.success(result)
}

fn preflight_result_decoder() {
  use ok <- decode.optional_field("ok", False, decode.bool)
  use errors <- decode.optional_field("errors", 0, decode.int)
  use warnings <- decode.optional_field("warnings", 0, decode.int)
  use steps <- decode.optional_field(
    "steps",
    [],
    decode.list(preflight_step_decoder()),
  )
  decode.success(domain.PreflightResult(ok:, errors:, warnings:, steps:))
}

fn preflight_step_decoder() {
  use name <- decode.optional_field("name", "", decode.string)
  use errors <- decode.optional_field("errors", 0, decode.int)
  use warnings <- decode.optional_field("warnings", 0, decode.int)
  use issues <- decode.optional_field(
    "issues",
    [],
    decode.list(preflight_issue_decoder()),
  )
  decode.success(domain.PreflightStep(name:, errors:, warnings:, issues:))
}

fn preflight_issue_decoder() {
  use level <- decode.optional_field("level", "error", decode.string)
  use path <- decode.optional_field("path", "", decode.string)
  use message <- decode.optional_field("message", "", decode.string)
  decode.success(domain.PreflightIssue(level:, path:, message:))
}
