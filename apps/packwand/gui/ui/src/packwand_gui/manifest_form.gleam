//// Structured manifest editor model: a typed mirror of
//// tools/manifest/schema.json with parse/serialize/validate. The raw JSON
//// textarea remains available as an escape hatch; this module round-trips
//// every field the schema defines.

import gleam/dict
import gleam/dynamic/decode
import gleam/int
import gleam/json
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/string

pub type RoleKind {
  RoleNone
  RoleBase
  RoleConsumer
}

pub type Mapping {
  Mapping(source: String, target: String)
}

pub type FormVariant {
  FormVariant(
    mc_version: String,
    id: String,
    name: String,
    version: String,
    release_type: String,
    loader: String,
  )
}

pub type Automation {
  Automation(
    auto_update: Option(Bool),
    server_promo: Option(Bool),
    sync_exclude: List(String),
    freeze: List(#(String, List(String))),
  )
}

pub type ManifestForm {
  ManifestForm(
    schema: String,
    id: String,
    name: String,
    kind: String,
    loader: String,
    release_type: String,
    version: String,
    mc_version: String,
    use_variants: Bool,
    variants: List(FormVariant),
    modrinth_id: String,
    curseforge_id: String,
    github_id: String,
    gitea_id: String,
    gitlab_id: String,
    lifecycle: String,
    role_kind: RoleKind,
    role_pack: String,
    role_mappings: List(Mapping),
    shared_assets: String,
    automation: Option(Automation),
  )
}

// — field updates —

pub type VariantField {
  VMcVersion(String)
  VId(String)
  VName(String)
  VVersion(String)
  VReleaseType(String)
  VLoader(String)
}

pub type Field {
  FId(String)
  FName(String)
  FKind(String)
  FLoader(String)
  FReleaseType(String)
  FVersion(String)
  FMcVersion(String)
  FUseVariants(Bool)
  FVariantAdd
  FVariantRemove(Int)
  FVariant(Int, VariantField)
  FModrinthId(String)
  FCurseforgeId(String)
  FGithubId(String)
  FGiteaId(String)
  FGitlabId(String)
  FLifecycle(String)
  FRoleKind(String)
  FRolePack(String)
  FMappingAdd
  FMappingRemove(Int)
  FMappingSource(Int, String)
  FMappingTarget(Int, String)
  FSharedAssets(String)
  FAutoUpdate(String)
  FServerPromo(String)
}

pub fn apply(form: ManifestForm, field: Field) -> ManifestForm {
  case field {
    FId(v) -> ManifestForm(..form, id: v)
    FName(v) -> ManifestForm(..form, name: v)
    FKind(v) -> ManifestForm(..form, kind: v)
    FLoader(v) -> ManifestForm(..form, loader: v)
    FReleaseType(v) -> ManifestForm(..form, release_type: v)
    FVersion(v) -> ManifestForm(..form, version: v)
    FMcVersion(v) -> ManifestForm(..form, mc_version: v)
    FUseVariants(v) ->
      case v, form.variants {
        True, [] ->
          ManifestForm(..form, use_variants: True, variants: [empty_variant()])
        _, _ -> ManifestForm(..form, use_variants: v)
      }
    FVariantAdd ->
      ManifestForm(
        ..form,
        variants: list.append(form.variants, [empty_variant()]),
      )
    FVariantRemove(index) ->
      ManifestForm(..form, variants: remove_at(form.variants, index))
    FVariant(index, vf) ->
      ManifestForm(
        ..form,
        variants: update_at(form.variants, index, apply_variant(_, vf)),
      )
    FModrinthId(v) -> ManifestForm(..form, modrinth_id: v)
    FCurseforgeId(v) -> ManifestForm(..form, curseforge_id: v)
    FGithubId(v) -> ManifestForm(..form, github_id: v)
    FGiteaId(v) -> ManifestForm(..form, gitea_id: v)
    FGitlabId(v) -> ManifestForm(..form, gitlab_id: v)
    FLifecycle(v) -> ManifestForm(..form, lifecycle: v)
    FRoleKind(v) ->
      case v {
        "base" -> ManifestForm(..form, role_kind: RoleBase)
        "consumer" ->
          case form.role_mappings {
            [] ->
              ManifestForm(
                ..form,
                role_kind: RoleConsumer,
                role_mappings: [Mapping("", "")],
              )
            _ -> ManifestForm(..form, role_kind: RoleConsumer)
          }
        _ -> ManifestForm(..form, role_kind: RoleNone)
      }
    FRolePack(v) -> ManifestForm(..form, role_pack: v)
    FMappingAdd ->
      ManifestForm(
        ..form,
        role_mappings: list.append(form.role_mappings, [Mapping("", "")]),
      )
    FMappingRemove(index) ->
      ManifestForm(..form, role_mappings: remove_at(form.role_mappings, index))
    FMappingSource(index, v) ->
      ManifestForm(
        ..form,
        role_mappings: update_at(form.role_mappings, index, fn(m) {
          Mapping(..m, source: v)
        }),
      )
    FMappingTarget(index, v) ->
      ManifestForm(
        ..form,
        role_mappings: update_at(form.role_mappings, index, fn(m) {
          Mapping(..m, target: v)
        }),
      )
    FSharedAssets(v) -> ManifestForm(..form, shared_assets: v)
    FAutoUpdate(v) ->
      ManifestForm(
        ..form,
        automation: set_automation_bool(form.automation, v, fn(settings, value) {
          Automation(..settings, auto_update: value)
        }),
      )
    FServerPromo(v) ->
      ManifestForm(
        ..form,
        automation: set_automation_bool(form.automation, v, fn(settings, value) {
          Automation(..settings, server_promo: value)
        }),
      )
  }
}

fn apply_variant(variant: FormVariant, field: VariantField) -> FormVariant {
  case field {
    VMcVersion(v) -> FormVariant(..variant, mc_version: v)
    VId(v) -> FormVariant(..variant, id: v)
    VName(v) -> FormVariant(..variant, name: v)
    VVersion(v) -> FormVariant(..variant, version: v)
    VReleaseType(v) -> FormVariant(..variant, release_type: v)
    VLoader(v) -> FormVariant(..variant, loader: v)
  }
}

fn empty_variant() -> FormVariant {
  FormVariant("", "", "", "", "", "")
}

fn empty_automation() -> Automation {
  Automation(None, None, [], [])
}

fn set_automation_bool(
  automation: Option(Automation),
  raw: String,
  set: fn(Automation, Option(Bool)) -> Automation,
) -> Option(Automation) {
  let value = case raw {
    "true" -> Some(True)
    "false" -> Some(False)
    _ -> None
  }
  let settings = option.unwrap(automation, empty_automation())
  let updated = set(settings, value)
  case updated {
    Automation(None, None, [], []) -> None
    _ -> Some(updated)
  }
}

fn remove_at(items: List(a), index: Int) -> List(a) {
  items
  |> list.index_map(fn(item, i) { #(i, item) })
  |> list.filter(fn(pair) { pair.0 != index })
  |> list.map(fn(pair) { pair.1 })
}

fn update_at(items: List(a), index: Int, updater: fn(a) -> a) -> List(a) {
  list.index_map(items, fn(item, i) {
    case i == index {
      True -> updater(item)
      False -> item
    }
  })
}

// — parsing —

pub fn parse(raw: String) -> Result(ManifestForm, String) {
  case json.parse(raw, form_decoder()) {
    Ok(form) -> Ok(form)
    Error(error) -> Error(describe_parse_error(error))
  }
}

fn describe_parse_error(error: json.DecodeError) -> String {
  case error {
    json.UnexpectedEndOfInput -> "Unexpected end of JSON input."
    json.UnexpectedByte(byte) -> "Unexpected byte " <> byte <> " in JSON."
    json.UnexpectedSequence(seq) -> "Unexpected sequence " <> seq <> " in JSON."
    json.UnableToDecode(_) -> "The manifest JSON has an unexpected shape."
  }
}

type RoleData {
  RoleData(kind: RoleKind, pack: String, mappings: List(Mapping))
}

fn form_decoder() -> decode.Decoder(ManifestForm) {
  use schema <- decode.optional_field("$schema", "", decode.string)
  use id <- decode.optional_field("id", "", decode.string)
  use name <- decode.optional_field("name", "", decode.string)
  use kind <- decode.optional_field("type", "", decode.string)
  use loader <- decode.optional_field("loader", "", decode.string)
  use release_type <- decode.optional_field("release_type", "", decode.string)
  use version <- decode.optional_field("version", "", decode.string)
  use mc_version <- decode.optional_field("mc_version", "", decode.string)
  use variants <- decode.optional_field(
    "variants",
    [],
    decode.list(form_variant_decoder()),
  )
  use modrinth_id <- decode.optional_field("modrinth_id", "", decode.string)
  use curseforge_id <- decode.optional_field("curseforge_id", "", decode.string)
  use github_id <- decode.optional_field("github_id", "", decode.string)
  use gitea_id <- decode.optional_field("gitea_id", "", decode.string)
  use gitlab_id <- decode.optional_field("gitlab_id", "", decode.string)
  use lifecycle <- decode.optional_field("lifecycle", "", decode.string)
  use role <- decode.optional_field(
    "role",
    RoleData(RoleNone, "", []),
    role_decoder(),
  )
  use shared_assets <- decode.optional_field("shared_assets", "", decode.string)
  use automation <- decode.optional_field(
    "automation",
    None,
    decode.map(automation_decoder(), Some),
  )
  decode.success(ManifestForm(
    schema:,
    id:,
    name:,
    kind:,
    loader:,
    release_type:,
    version:,
    mc_version:,
    use_variants: variants != [],
    variants:,
    modrinth_id:,
    curseforge_id:,
    github_id:,
    gitea_id:,
    gitlab_id:,
    lifecycle:,
    role_kind: role.kind,
    role_pack: role.pack,
    role_mappings: role.mappings,
    shared_assets:,
    automation:,
  ))
}

fn form_variant_decoder() -> decode.Decoder(FormVariant) {
  use mc_version <- decode.optional_field("mc_version", "", decode.string)
  use id <- decode.optional_field("id", "", decode.string)
  use name <- decode.optional_field("name", "", decode.string)
  use version <- decode.optional_field("version", "", decode.string)
  use release_type <- decode.optional_field("release_type", "", decode.string)
  use loader <- decode.optional_field("loader", "", decode.string)
  decode.success(FormVariant(
    mc_version:,
    id:,
    name:,
    version:,
    release_type:,
    loader:,
  ))
}

fn role_decoder() -> decode.Decoder(RoleData) {
  let as_string =
    decode.string
    |> decode.map(fn(value) {
      case value {
        "base" -> RoleData(RoleBase, "", [])
        _ -> RoleData(RoleNone, "", [])
      }
    })
  let as_consumer = {
    use pack <- decode.subfield(["performance_base", "pack"], decode.string)
    use mappings <- decode.subfield(
      ["performance_base", "mappings"],
      decode.list(mapping_decoder()),
    )
    decode.success(RoleData(RoleConsumer, pack, mappings))
  }
  decode.one_of(as_string, or: [as_consumer])
}

fn mapping_decoder() -> decode.Decoder(Mapping) {
  use source <- decode.optional_field("source", "", decode.string)
  use target <- decode.optional_field("target", "", decode.string)
  decode.success(Mapping(source:, target:))
}

fn automation_decoder() -> decode.Decoder(Automation) {
  use auto_update <- decode.optional_field(
    "auto_update",
    None,
    decode.map(decode.bool, Some),
  )
  use server_promo <- decode.optional_field(
    "server_promo",
    None,
    decode.map(decode.bool, Some),
  )
  use sync_exclude <- decode.optional_field(
    "sync_exclude",
    [],
    decode.list(decode.string),
  )
  use freeze <- decode.optional_field("freeze", [], freeze_decoder())
  decode.success(Automation(auto_update:, server_promo:, sync_exclude:, freeze:))
}

fn freeze_decoder() -> decode.Decoder(List(#(String, List(String)))) {
  decode.dict(decode.string, decode.list(decode.string))
  |> decode.map(fn(entries) {
    entries
    |> dict.to_list
    |> list.sort(fn(a, b) { string.compare(a.0, b.0) })
  })
}

// — serialization —

/// Serialize the form back to manifest JSON (2-space indented).
pub fn serialize(form: ManifestForm) -> String {
  let optional_string = fn(key, value) {
    case string.trim(value) {
      "" -> []
      _ -> [#(key, json.string(value))]
    }
  }

  let shape = case form.use_variants {
    True -> [
      #(
        "variants",
        json.array(form.variants, fn(v) {
          json.object(
            [#("mc_version", json.string(v.mc_version))]
            |> list.append(optional_string("id", v.id))
            |> list.append(optional_string("name", v.name))
            |> list.append(optional_string("version", v.version))
            |> list.append(optional_string("release_type", v.release_type))
            |> list.append(optional_string("loader", v.loader)),
          )
        }),
      ),
    ]
    False -> [#("mc_version", json.string(form.mc_version))]
  }

  let role = case form.role_kind {
    RoleNone -> json.string("none")
    RoleBase -> json.string("base")
    RoleConsumer ->
      json.object([
        #(
          "performance_base",
          json.object([
            #("pack", json.string(form.role_pack)),
            #(
              "mappings",
              json.array(form.role_mappings, fn(m) {
                json.object([
                  #("source", json.string(m.source)),
                  #("target", json.string(m.target)),
                ])
              }),
            ),
          ]),
        ),
      ])
  }

  let platform_ids =
    list.flatten([
      optional_string("modrinth_id", form.modrinth_id),
      optional_string("curseforge_id", form.curseforge_id),
      optional_string("github_id", form.github_id),
      optional_string("gitea_id", form.gitea_id),
      optional_string("gitlab_id", form.gitlab_id),
    ])
  // The schema requires at least one platform id key; keep an explicit empty
  // modrinth_id (matching existing unpublished manifests) when none is set.
  let platform_ids = case platform_ids {
    [] -> [#("modrinth_id", json.string(""))]
    _ -> platform_ids
  }

  let automation = case form.automation {
    None -> []
    Some(settings) -> [#("automation", automation_json(settings))]
  }

  let pairs =
    list.flatten([
      optional_string("$schema", form.schema),
      [
        #("id", json.string(form.id)),
        #("name", json.string(form.name)),
        #("type", json.string(form.kind)),
      ],
      optional_string("loader", form.loader),
      optional_string("version", form.version),
      shape,
      [#("release_type", json.string(form.release_type)), #("role", role)],
      optional_string("lifecycle", form.lifecycle),
      platform_ids,
      optional_string("shared_assets", form.shared_assets),
      automation,
    ])

  json.object(pairs)
  |> json.to_string
  |> pretty_json
}

fn automation_json(settings: Automation) -> json.Json {
  let bool_field = fn(key, value) {
    case value {
      Some(b) -> [#(key, json.bool(b))]
      None -> []
    }
  }
  let sync_exclude = case settings.sync_exclude {
    [] -> []
    values -> [#("sync_exclude", json.array(values, json.string))]
  }
  let freeze = case settings.freeze {
    [] -> []
    entries -> [
      #(
        "freeze",
        json.object(
          list.map(entries, fn(entry) {
            #(entry.0, json.array(entry.1, json.string))
          }),
        ),
      ),
    ]
  }
  json.object(
    list.flatten([
      bool_field("auto_update", settings.auto_update),
      bool_field("server_promo", settings.server_promo),
      sync_exclude,
      freeze,
    ]),
  )
}

@external(javascript, "../packwand_gui/ffi.mjs", "prettyJson")
fn pretty_json(raw: String) -> String

// — validation —

pub type Severity {
  IssueError
  IssueWarning
}

pub type Issue {
  Issue(field: String, severity: Severity, message: String)
}

pub fn validate(form: ManifestForm) -> List(Issue) {
  let required = fn(field, value, label) {
    case string.trim(value) {
      "" -> [Issue(field, IssueError, label <> " is required.")]
      _ -> []
    }
  }

  let identity =
    list.flatten([
      required("id", form.id, "Pack ID"),
      required("name", form.name, "Name"),
      required("type", form.kind, "Type"),
      required("release_type", form.release_type, "Release type"),
      required("version", form.version, "Version"),
    ])

  let loader = case
    form.kind == "modpack"
    && string.trim(form.loader) == ""
    && !all_variants_have_loaders(form)
  {
    True -> [
      Issue(
          "loader",
          IssueError,
        "Modpacks must declare a loader (pack-level or on every variant).",
      ),
    ]
    False -> []
  }

  let shape = case form.use_variants {
    True ->
      case form.variants {
        [] -> [Issue(
          "variants",
          IssueError, "Add at least one variant.")]
        variants ->
          variants
          |> list.index_map(fn(variant, index) {
            let label = "variants[" <> int_to_string(index) <> "]"
            case string.trim(variant.mc_version) {
              "" -> [
                Issue(
                  label,
                  IssueError,
                  "Variant " <> int_to_string(index + 1) <> " needs mc_version.",
                ),
              ]
              _ -> []
            }
          })
          |> list.flatten
      }
    False -> required("mc_version", form.mc_version, "Minecraft version")
  }

  let platforms = case
    [
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
    ]
    |> list.any(fn(value) { string.trim(value) != "" })
  {
    True -> []
    False -> {
      let severity = case form.kind {
        "resourcepack" -> IssueWarning
        _ -> IssueError
      }
      [
        Issue(
          "platforms",
          severity,
          "Set at least one platform id (Modrinth, CurseForge, GitHub, Gitea, or GitLab).",
        ),
      ]
    }
  }

  let role = case form.role_kind {
    RoleConsumer ->
      list.flatten([
        required("role_pack", form.role_pack, "Performance base pack"),
        case form.role_mappings {
          [] -> [
            Issue(
          "role_mappings",
          IssueError, "Add at least one base mapping."),
          ]
          mappings ->
            mappings
            |> list.index_map(fn(mapping, index) {
              validate_mapping(mapping, index)
            })
            |> list.flatten
        },
      ])
    _ -> []
  }

  let lifecycle = case form.lifecycle {
    "" | "active" | "maintenance" | "archived" | "eol" -> []
    other -> [
      Issue(
          "lifecycle",
          IssueError,
        "Invalid lifecycle '" <> other <> "' (active, maintenance, archived, eol).",
      ),
    ]
  }

  list.flatten([identity, loader, shape, platforms, role, lifecycle])
}

fn validate_mapping(mapping: Mapping, index: Int) -> List(Issue) {
  let label = "mapping[" <> int_to_string(index) <> "]"
  let prefix = "Mapping " <> int_to_string(index + 1)
  let source_suffix = platform_suffix(mapping.source)
  let target_suffix = platform_suffix(mapping.target)
  list.flatten([
    case source_suffix {
      "" -> [Issue(label, IssueError, prefix <> ": source must end in -mr or -cf.")]
      _ -> []
    },
    case target_suffix {
      "" -> [Issue(label, IssueError, prefix <> ": target must end in -mr or -cf.")]
      _ -> []
    },
    case source_suffix != "" && target_suffix != "" && source_suffix != target_suffix {
      True -> [
        Issue(
          label,
          IssueError,
          prefix
            <> ": source and target must share a platform suffix (MR/CF must never cross).",
        ),
      ]
      False -> []
    },
  ])
}

fn platform_suffix(value: String) -> String {
  case string.ends_with(value, "-mr"), string.ends_with(value, "-cf") {
    True, _ -> "mr"
    _, True -> "cf"
    _, _ -> ""
  }
}

fn all_variants_have_loaders(form: ManifestForm) -> Bool {
  form.use_variants
  && form.variants != []
  && list.all(form.variants, fn(v) { string.trim(v.loader) != "" })
}

pub fn errors(issues: List(Issue)) -> List(Issue) {
  list.filter(issues, fn(issue) { issue.severity == IssueError })
}

pub fn field_issues(issues: List(Issue), field: String) -> List(Issue) {
  list.filter(issues, fn(issue) { issue.field == field })
}

fn int_to_string(value: Int) -> String {
  int.to_string(value)
}
