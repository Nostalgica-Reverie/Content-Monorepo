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
  request("GET", "/api/health", "", health_decoder(), to_msg)
}

pub fn projects(
  to_msg: fn(Result(domain.ProjectIndex, domain.ApiError)) -> msg,
) -> Effect(msg) {
  request("GET", "/api/projects", "", project_index_decoder(), to_msg)
}

pub fn mods(
  path: String,
  to_msg: fn(Result(List(domain.ModEntry), domain.ApiError)) -> msg,
) -> Effect(msg) {
  request(
    "GET",
    "/api/mods?subdir=" <> uri.percent_encode(path),
    "",
    decode.list(mod_decoder()),
    to_msg,
  )
}

pub fn changelog(
  id: String,
  to_msg: fn(Result(domain.ContentResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  content("/api/projects/" <> uri.percent_encode(id) <> "/changelog", to_msg)
}

pub fn manifest(
  id: String,
  to_msg: fn(Result(domain.ContentResponse, domain.ApiError)) -> msg,
) -> Effect(msg) {
  content("/api/projects/" <> uri.percent_encode(id) <> "/manifest", to_msg)
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
    "/api/projects/" <> uri.percent_encode(id) <> "/manifest",
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
    "/api/projects",
    json.to_string(body),
    created_project_decoder(),
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
    ])
  request(
    "POST",
    "/api/actions",
    json.to_string(body),
    action_response_decoder(),
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
  decode.success(domain.ModEntry(
    slug:,
    name:,
    filename:,
    side:,
    pin:,
    platform:,
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
