import * as $json from "../../gleam_json/gleam/json.mjs";
import * as $decode from "../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import * as $uri from "../../gleam_stdlib/gleam/uri.mjs";
import * as $effect from "../../lustre/lustre/effect.mjs";
import { Ok, Error, toList } from "../gleam.mjs";
import * as $domain from "../packwand_gui/model.mjs";
import { requestJson as request_json } from "./ffi.mjs";

function health_decoder() {
  return $decode.optional_field(
    "root",
    "",
    $decode.string,
    (root) => {
      return $decode.optional_field(
        "version",
        "",
        $decode.string,
        (version) => {
          return $decode.success(new $domain.Health(root, version));
        },
      );
    },
  );
}

function request(method, url, body, decoder, to_msg) {
  return $effect.from(
    (dispatch) => {
      return request_json(
        method,
        url,
        body,
        (response) => {
          let _block;
          let $ = $json.parse(response, decoder);
          if ($ instanceof Ok) {
            _block = $;
          } else {
            let error = $[0];
            _block = new Error(new $domain.DecodeError($string.inspect(error)));
          }
          let result = _block;
          return dispatch(to_msg(result));
        },
        (error) => {
          return dispatch(to_msg(new Error(new $domain.ApiError(error))));
        },
      );
    },
  );
}

export function health(to_msg) {
  return request("GET", "/api/v1/version", "", health_decoder(), to_msg);
}

function subdir_decoder() {
  return $decode.optional_field(
    "key",
    "",
    $decode.string,
    (key) => {
      return $decode.field(
        "path",
        $decode.string,
        (path) => {
          return $decode.optional_field(
            "platform",
            "",
            $decode.string,
            (platform) => {
              return $decode.optional_field(
                "mod_count",
                0,
                $decode.int,
                (mod_count) => {
                  return $decode.optional_field(
                    "has_index",
                    false,
                    $decode.bool,
                    (has_index) => {
                      return $decode.optional_field(
                        "has_pack",
                        false,
                        $decode.bool,
                        (has_pack) => {
                          return $decode.success(
                            new $domain.Subdir(
                              key,
                              path,
                              platform,
                              mod_count,
                              has_index,
                              has_pack,
                            ),
                          );
                        },
                      );
                    },
                  );
                },
              );
            },
          );
        },
      );
    },
  );
}

function variant_decoder() {
  return $decode.optional_field(
    "id",
    "",
    $decode.string,
    (id) => {
      return $decode.optional_field(
        "mc_version",
        "",
        $decode.string,
        (minecraft) => {
          return $decode.optional_field(
            "loader",
            "",
            $decode.string,
            (loader) => {
              return $decode.optional_field(
                "version",
                "",
                $decode.string,
                (version) => {
                  return $decode.success(
                    new $domain.Variant(id, minecraft, loader, version),
                  );
                },
              );
            },
          );
        },
      );
    },
  );
}

function project_decoder() {
  return $decode.field(
    "id",
    $decode.string,
    (id) => {
      return $decode.optional_field(
        "name",
        "",
        $decode.string,
        (name) => {
          return $decode.optional_field(
            "type",
            "",
            $decode.string,
            (kind) => {
              return $decode.optional_field(
                "dir",
                "",
                $decode.string,
                (dir) => {
                  return $decode.optional_field(
                    "manifest_path",
                    "",
                    $decode.string,
                    (manifest_path) => {
                      return $decode.optional_field(
                        "version",
                        "",
                        $decode.string,
                        (version) => {
                          return $decode.optional_field(
                            "mc_version",
                            "",
                            $decode.string,
                            (minecraft) => {
                              return $decode.optional_field(
                                "loader",
                                "",
                                $decode.string,
                                (loader) => {
                                  return $decode.optional_field(
                                    "release_type",
                                    "",
                                    $decode.string,
                                    (release_type) => {
                                      return $decode.optional_field(
                                        "lifecycle",
                                        "",
                                        $decode.string,
                                        (lifecycle) => {
                                          return $decode.optional_field(
                                            "role",
                                            "",
                                            $decode.string,
                                            (role) => {
                                              return $decode.optional_field(
                                                "auto_update",
                                                false,
                                                $decode.bool,
                                                (auto_update) => {
                                                  return $decode.optional_field(
                                                    "modrinth_id",
                                                    "",
                                                    $decode.string,
                                                    (modrinth_id) => {
                                                      return $decode.optional_field(
                                                        "curseforge_id",
                                                        "",
                                                        $decode.string,
                                                        (curseforge_id) => {
                                                          return $decode.optional_field(
                                                            "github_id",
                                                            "",
                                                            $decode.string,
                                                            (github_id) => {
                                                              return $decode.optional_field(
                                                                "gitea_id",
                                                                "",
                                                                $decode.string,
                                                                (gitea_id) => {
                                                                  return $decode.optional_field(
                                                                    "gitlab_id",
                                                                    "",
                                                                    $decode.string,
                                                                    (gitlab_id) => {
                                                                      return $decode.optional_field(
                                                                        "docs_path",
                                                                        "",
                                                                        $decode.string,
                                                                        (
                                                                            docs_path
                                                                          ) => {
                                                                          return $decode.optional_field(
                                                                            "variants",
                                                                            toList([]),
                                                                            $decode.list(
                                                                              variant_decoder(),
                                                                            ),
                                                                            (
                                                                                variants
                                                                              ) => {
                                                                              return $decode.optional_field(
                                                                                "subdirs",
                                                                                toList([]),
                                                                                $decode.list(
                                                                                  subdir_decoder(),
                                                                                ),
                                                                                (
                                                                                    subdirs
                                                                                  ) => {
                                                                                  return $decode.success(
                                                                                    new $domain.Project(
                                                                                      id,
                                                                                      name,
                                                                                      kind,
                                                                                      dir,
                                                                                      manifest_path,
                                                                                      version,
                                                                                      minecraft,
                                                                                      loader,
                                                                                      release_type,
                                                                                      lifecycle,
                                                                                      role,
                                                                                      auto_update,
                                                                                      modrinth_id,
                                                                                      curseforge_id,
                                                                                      github_id,
                                                                                      gitea_id,
                                                                                      gitlab_id,
                                                                                      docs_path,
                                                                                      variants,
                                                                                      subdirs,
                                                                                    ),
                                                                                  );
                                                                                },
                                                                              );
                                                                            },
                                                                          );
                                                                        },
                                                                      );
                                                                    },
                                                                  );
                                                                },
                                                              );
                                                            },
                                                          );
                                                        },
                                                      );
                                                    },
                                                  );
                                                },
                                              );
                                            },
                                          );
                                        },
                                      );
                                    },
                                  );
                                },
                              );
                            },
                          );
                        },
                      );
                    },
                  );
                },
              );
            },
          );
        },
      );
    },
  );
}

function project_index_decoder() {
  return $decode.optional_field(
    "projects",
    toList([]),
    $decode.list(project_decoder()),
    (projects) => { return $decode.success(new $domain.ProjectIndex(projects)); },
  );
}

export function projects(to_msg) {
  return request("GET", "/api/v1/packs", "", project_index_decoder(), to_msg);
}

function feature_decoder() {
  return $decode.field(
    "command",
    $decode.string,
    (command) => {
      return $decode.optional_field(
        "use",
        "",
        $decode.string,
        (usage) => {
          return $decode.optional_field(
            "summary",
            "",
            $decode.string,
            (summary) => {
              return $decode.optional_field(
                "group",
                "",
                $decode.string,
                (group) => {
                  return $decode.optional_field(
                    "runnable",
                    false,
                    $decode.bool,
                    (runnable) => {
                      return $decode.optional_field(
                        "gui_status",
                        "cli-only",
                        $decode.string,
                        (gui_status) => {
                          return $decode.optional_field(
                            "gui_action",
                            "",
                            $decode.string,
                            (gui_action) => {
                              return $decode.optional_field(
                                "scope",
                                "",
                                $decode.string,
                                (scope) => {
                                  return $decode.optional_field(
                                    "destructive",
                                    false,
                                    $decode.bool,
                                    (destructive) => {
                                      return $decode.success(
                                        new $domain.Feature(
                                          command,
                                          usage,
                                          summary,
                                          group,
                                          runnable,
                                          gui_status,
                                          gui_action,
                                          scope,
                                          destructive,
                                        ),
                                      );
                                    },
                                  );
                                },
                              );
                            },
                          );
                        },
                      );
                    },
                  );
                },
              );
            },
          );
        },
      );
    },
  );
}

function feature_index_decoder() {
  return $decode.optional_field(
    "packwand_version",
    "",
    $decode.string,
    (packwand_version) => {
      return $decode.optional_field(
        "features",
        toList([]),
        $decode.list(feature_decoder()),
        (features) => {
          return $decode.success(
            new $domain.FeatureIndex(packwand_version, features),
          );
        },
      );
    },
  );
}

export function features(to_msg) {
  return request(
    "GET",
    "/api/v1/capabilities",
    "",
    feature_index_decoder(),
    to_msg,
  );
}

function mod_decoder() {
  return $decode.field(
    "slug",
    $decode.string,
    (slug) => {
      return $decode.optional_field(
        "name",
        "",
        $decode.string,
        (name) => {
          return $decode.optional_field(
            "filename",
            "",
            $decode.string,
            (filename) => {
              return $decode.optional_field(
                "side",
                "",
                $decode.string,
                (side) => {
                  return $decode.optional_field(
                    "pin",
                    false,
                    $decode.bool,
                    (pin) => {
                      return $decode.optional_field(
                        "platform",
                        "",
                        $decode.string,
                        (platform) => {
                          return $decode.optional_field(
                            "version_id",
                            "",
                            $decode.string,
                            (version_id) => {
                              return $decode.success(
                                new $domain.ModEntry(
                                  slug,
                                  name,
                                  filename,
                                  side,
                                  pin,
                                  platform,
                                  version_id,
                                ),
                              );
                            },
                          );
                        },
                      );
                    },
                  );
                },
              );
            },
          );
        },
      );
    },
  );
}

export function mods(path, to_msg) {
  return request(
    "GET",
    "/api/v1/mods?subdir=" + $uri.percent_encode(path),
    "",
    $decode.list(mod_decoder()),
    to_msg,
  );
}

function content_decoder() {
  return $decode.optional_field(
    "path",
    "",
    $decode.string,
    (path) => {
      return $decode.optional_field(
        "content",
        "",
        $decode.string,
        (content) => {
          return $decode.success(new $domain.ContentResponse(path, content));
        },
      );
    },
  );
}

function content(url, to_msg) {
  return request("GET", url, "", content_decoder(), to_msg);
}

export function changelog(id, to_msg) {
  return content(
    ("/api/v1/packs/" + $uri.percent_encode(id)) + "/changelog",
    to_msg,
  );
}

export function manifest(id, to_msg) {
  return content(
    ("/api/v1/packs/" + $uri.percent_encode(id)) + "/manifest",
    to_msg,
  );
}

export function save_manifest(id, content, to_msg) {
  return request(
    "PUT",
    ("/api/v1/packs/" + $uri.percent_encode(id)) + "/manifest",
    (() => {
      let _pipe = $json.object(toList([["content", $json.string(content)]]));
      return $json.to_string(_pipe);
    })(),
    $decode.success(undefined),
    to_msg,
  );
}

function created_project_decoder() {
  return $decode.field(
    "id",
    $decode.string,
    (id) => {
      return $decode.field(
        "dir",
        $decode.string,
        (dir) => { return $decode.success(new $domain.CreatedProject(id, dir)); },
      );
    },
  );
}

export function create_project(
  id,
  name,
  kind,
  loader,
  minecraft,
  version,
  description,
  to_msg
) {
  let body = $json.object(
    toList([
      ["id", $json.string(id)],
      ["name", $json.string(name)],
      ["type", $json.string(kind)],
      ["loader", $json.string(loader)],
      ["mc_version", $json.string(minecraft)],
      ["version", $json.string(version)],
      ["description", $json.string(description)],
    ]),
  );
  return request(
    "POST",
    "/api/v1/packs",
    $json.to_string(body),
    created_project_decoder(),
    to_msg,
  );
}

function action_response_decoder() {
  return $decode.field(
    "job_id",
    $decode.string,
    (job_id) => { return $decode.success(new $domain.ActionResponse(job_id)); },
  );
}

/**
 * Opens the native mod browser webview (lib/mod-browser-webview) for the
 * given mod on the given provider ("curseforge" or "modrinth"), bridged by
 * the Go server; download/navigation events stream through the returned
 * job's event feed.
 */
export function webview_fetch(provider, slug, file_id, to_msg) {
  let body = $json.object(
    toList([
      ["provider", $json.string(provider)],
      [
        "files",
        $json.array(
          toList([
            $json.object(
              toList([
                ["file_id", $json.string(file_id)],
                ["slug", $json.string(slug)],
              ]),
            ),
          ]),
          (file) => { return file; },
        ),
      ],
    ]),
  );
  return request(
    "POST",
    "/api/v1/webview/open",
    $json.to_string(body),
    action_response_decoder(),
    to_msg,
  );
}

export function action(action, to_msg) {
  let body = $json.object(
    toList([
      ["action", $json.string($domain.action_name(action))],
      ["subdir", $json.string($domain.action_subdir(action))],
      ["slug", $json.string($domain.action_slug(action))],
      ["dry_run", $json.bool($domain.action_dry_run(action))],
      ["version", $json.string($domain.action_version(action))],
      ["configs", $json.bool($domain.action_configs(action))],
      ["side", $json.string($domain.action_side(action))],
    ]),
  );
  return request(
    "POST",
    "/api/v1/actions",
    $json.to_string(body),
    action_response_decoder(),
    to_msg,
  );
}
