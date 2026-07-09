import * as $decode from "../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $int from "../../gleam_stdlib/gleam/int.mjs";
import * as $list from "../../gleam_stdlib/gleam/list.mjs";
import * as $option from "../../gleam_stdlib/gleam/option.mjs";
import { None, Some } from "../../gleam_stdlib/gleam/option.mjs";
import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import * as $attribute from "../../lustre/lustre/attribute.mjs";
import * as $element from "../../lustre/lustre/element.mjs";
import * as $html from "../../lustre/lustre/element/html.mjs";
import * as $event from "../../lustre/lustre/event.mjs";
import { Ok, toList, Empty as $Empty, prepend as listPrepend, isEqual } from "../gleam.mjs";
import * as $manifest_form from "../packwand_gui/manifest_form.mjs";
import * as $model from "../packwand_gui/model.mjs";
import {
  AddMod,
  Build,
  Bump,
  DocsModlist,
  DocsPages,
  Doctor,
  ExportCurseforge,
  ExportModrinth,
  FreezeMod,
  Lint,
  NixGen,
  PacksIndex,
  PinMod,
  RefreshSubdir,
  Rehash,
  RemoveMod,
  SetSide,
  UnfreezeMod,
  UnpinMod,
  UpdateAll,
  UpdateMod,
  ValidateProject,
  WorkspaceRefresh,
  WorkspaceStatus,
  WorkspaceSync,
  WorkspaceUpdate,
  project_summary,
} from "../packwand_gui/model.mjs";
import * as $state from "../packwand_gui/state.mjs";
import {
  BootPack,
  CancelBoot,
  Changelog,
  CopyChangelog,
  CreateProject,
  Exports,
  IconFailed,
  Logs,
  Mods,
  Navigate,
  Overview,
  RequestAuthLogin,
  RequestAuthLogout,
  RunAction,
  RunWebview,
  SaveManifest,
  SelectProject,
  SelectSubdir,
  SetBumpConfigs,
  SetBumpVersion,
  SetDockGameWindow,
  SetManifest,
  SetManifestField,
  SetManifestStructured,
  SetModSlug,
  SetNewPackDescription,
  SetNewPackID,
  SetNewPackLoader,
  SetNewPackMinecraft,
  SetNewPackName,
  SetNewPackType,
  SetNewPackVersion,
  SetSearch,
  Settings,
  job_running,
  launcher_running,
  progress_status_label,
  query_matches,
  selected_project,
} from "../packwand_gui/state.mjs";
import { currentHash as current_hash } from "./ffi.mjs";

export function hash(view) {
  if (view instanceof Overview) {
    return "overview";
  } else if (view instanceof Exports) {
    return "exports";
  } else if (view instanceof Mods) {
    return "mods";
  } else if (view instanceof Changelog) {
    return "changelog";
  } else if (view instanceof Logs) {
    return "logs";
  } else {
    return "settings";
  }
}

export function from_name(value) {
  if (value === "exports") {
    return new Exports();
  } else if (value === "mods") {
    return new Mods();
  } else if (value === "changelog") {
    return new Changelog();
  } else if (value === "logs") {
    return new Logs();
  } else if (value === "settings") {
    return new Settings();
  } else {
    return new Overview();
  }
}

export function from_hash() {
  return from_name(current_hash());
}

function empty_row(value) {
  return $html.div(
    toList([$attribute.class$("row")]),
    toList([$html.span(toList([]), toList([$html.text(value)]))]),
  );
}

function pill(value) {
  return $html.span(
    toList([$attribute.class$("pill")]),
    toList([$html.text(value)]),
  );
}

function panel_with_head(class$, title, action, children) {
  return $html.section(
    toList([$attribute.class$("panel " + class$)]),
    listPrepend(
      $html.div(
        toList([$attribute.class$("panel-head")]),
        toList([$html.h2(toList([]), toList([$html.text(title)])), action]),
      ),
      children,
    ),
  );
}

function search_row(category, title, detail_text) {
  return $html.div(
    toList([$attribute.class$("row search-item")]),
    toList([
      $html.div(
        toList([]),
        toList([
          $html.strong(toList([]), toList([$html.text(title)])),
          $html.span(toList([]), toList([$html.text(detail_text)])),
        ]),
      ),
      $html.span(
        toList([$attribute.class$("result-kind")]),
        toList([$html.text(category)]),
      ),
    ]),
  );
}

function fallback(value, default$) {
  let $ = $string.trim(value);
  if ($ === "") {
    return default$;
  } else {
    return value;
  }
}

function search_results_panel(model, project) {
  let _block;
  let _pipe = toList([
    project.id,
    project.name,
    project.kind,
    project.dir,
    project.version,
    project.minecraft,
    project.loader,
  ]);
  _block = $string.join(_pipe, " ");
  let project_text = _block;
  let _block$1;
  let $ = query_matches(model.search, project_text);
  if ($) {
    _block$1 = toList([
      search_row("Project", project.id, project_summary(project)),
    ]);
  } else {
    _block$1 = toList([]);
  }
  let project_rows = _block$1;
  let _block$2;
  let _pipe$1 = project.subdirs;
  let _pipe$2 = $list.filter(
    _pipe$1,
    (item) => {
      return query_matches(
        model.search,
        (((item.key + " ") + item.path) + " ") + item.platform,
      );
    },
  );
  _block$2 = $list.map(
    _pipe$2,
    (item) => { return search_row("Subdir", item.key, item.path); },
  );
  let subdir_rows = _block$2;
  let _block$3;
  let _pipe$3 = model.mods;
  let _pipe$4 = $list.filter(
    _pipe$3,
    (item) => {
      return query_matches(
        model.search,
        (((((item.name + " ") + item.slug) + " ") + item.filename) + " ") + item.platform,
      );
    },
  );
  _block$3 = $list.map(
    _pipe$4,
    (item) => {
      return search_row(
        "Mod",
        fallback(item.name, item.slug),
        (item.slug + " / ") + item.platform,
      );
    },
  );
  let mod_rows = _block$3;
  let _block$4;
  let _pipe$5 = project.variants;
  let _pipe$6 = $list.filter(
    _pipe$5,
    (item) => {
      return query_matches(
        model.search,
        (((((item.id + " ") + item.minecraft) + " ") + item.loader) + " ") + item.version,
      );
    },
  );
  _block$4 = $list.map(
    _pipe$6,
    (item) => {
      return search_row(
        "Variant",
        fallback(item.id, item.minecraft),
        (item.loader + " / ") + item.version,
      );
    },
  );
  let variant_rows = _block$4;
  let _block$5;
  let _pipe$7 = model.changelog;
  let _pipe$8 = $string.split(_pipe$7, "\n");
  let _pipe$9 = $list.filter(
    _pipe$8,
    (line) => {
      return ($string.trim(line) !== "") && query_matches(model.search, line);
    },
  );
  _block$5 = $list.map(
    _pipe$9,
    (line) => { return search_row("Changelog", line, "changelog.md"); },
  );
  let changelog_rows = _block$5;
  let rows = $list.flatten(
    toList([project_rows, subdir_rows, mod_rows, variant_rows, changelog_rows]),
  );
  return panel_with_head(
    "span-12 search-results",
    "Search Results",
    pill($int.to_string($list.length(rows)) + " matches"),
    toList([
      $html.div(
        toList([$attribute.class$("list")]),
        (() => {
          if (rows instanceof $Empty) {
            return toList([empty_row("No matches in this pack.")]);
          } else {
            return rows;
          }
        })(),
      ),
    ]),
  );
}

function notice(value) {
  if (value === "") {
    return $html.text("");
  } else {
    return $html.p(
      toList([$attribute.class$("notice")]),
      toList([$html.text(value)]),
    );
  }
}

function panel(class$, title, children) {
  return panel_with_head(class$, title, $html.text(""), children);
}

function button(class$, label, message) {
  return $html.button(
    toList([
      $attribute.class$(class$),
      $attribute.type_("button"),
      $event.on_click(message),
    ]),
    toList([$html.text(label)]),
  );
}

function account_panel(model) {
  let _block;
  let $ = model.auth_signed_in;
  if ($) {
    _block = toList([
      $html.p(
        toList([$attribute.class$("panel-copy")]),
        toList([$html.text(("Signed in as " + model.auth_username) + ".")]),
      ),
      button("ghost", "Sign out", new RequestAuthLogout()),
    ]);
  } else {
    _block = toList([
      $html.p(
        toList([$attribute.class$("panel-copy")]),
        toList([
          $html.text(
            "Sign in with a real Microsoft account to boot as yourself instead of an offline dev-testing session. Optional — offline boots work either way.",
          ),
        ]),
      ),
      button("", "Sign in with Microsoft", new RequestAuthLogin()),
    ]);
  }
  let body = _block;
  return panel(
    "span-12",
    "Account",
    $list.append(body, toList([notice(model.auth_status_text)])),
  );
}

function form_input(label, value, placeholder, message, class$) {
  return $html.label(
    toList([$attribute.class$(class$)]),
    toList([
      $html.span(toList([]), toList([$html.text(label)])),
      $html.input(
        toList([
          $attribute.value(value),
          $attribute.placeholder(placeholder),
          $event.on_input(message),
        ]),
      ),
    ]),
  );
}

function new_pack_panel(model) {
  let draft = model.new_pack;
  return panel_with_head(
    "span-12",
    "New Pack",
    button("", "Create", new CreateProject()),
    toList([
      $html.div(
        toList([$attribute.class$("form-grid")]),
        toList([
          form_input(
            "ID",
            draft.id,
            "my-new-pack",
            (var0) => { return new SetNewPackID(var0); },
            "",
          ),
          form_input(
            "Name",
            draft.name,
            "My New Pack",
            (var0) => { return new SetNewPackName(var0); },
            "",
          ),
          $html.label(
            toList([]),
            toList([
              $html.span(toList([]), toList([$html.text("Type")])),
              $html.select(
                toList([
                  $attribute.value(draft.kind),
                  $event.on_change(
                    (var0) => { return new SetNewPackType(var0); },
                  ),
                ]),
                toList([
                  $html.option(toList([$attribute.value("modpack")]), "modpack"),
                  $html.option(
                    toList([$attribute.value("resourcepack")]),
                    "resourcepack",
                  ),
                  $html.option(
                    toList([$attribute.value("datapack")]),
                    "datapack",
                  ),
                ]),
              ),
            ]),
          ),
          form_input(
            "Loader",
            draft.loader,
            "fabric",
            (var0) => { return new SetNewPackLoader(var0); },
            "",
          ),
          form_input(
            "Minecraft",
            draft.minecraft,
            "1.21.1",
            (var0) => { return new SetNewPackMinecraft(var0); },
            "",
          ),
          form_input(
            "Version",
            draft.version,
            "0.1.0",
            (var0) => { return new SetNewPackVersion(var0); },
            "",
          ),
          form_input(
            "Description",
            draft.description,
            "Optional summary",
            (var0) => { return new SetNewPackDescription(var0); },
            "span-form",
          ),
        ]),
      ),
      notice(model.notice),
    ]),
  );
}

function feature_row(feature) {
  return $html.div(
    toList([$attribute.class$("feature-row")]),
    toList([
      $html.code(
        toList([]),
        toList([$html.text("packwand " + feature.command)]),
      ),
      $html.span(
        toList([$attribute.class$("feature-summary")]),
        toList([$html.text(fallback(feature.summary, feature.usage))]),
      ),
      $html.span(
        toList([
          $attribute.classes(
            toList([
              ["status-badge", true],
              ["integrated", feature.gui_status === "integrated"],
            ]),
          ),
        ]),
        toList([
          $html.text(
            (() => {
              let $ = feature.gui_status;
              if ($ === "integrated") {
                return "GUI";
              } else {
                return "CLI";
              }
            })(),
          ),
        ]),
      ),
    ]),
  );
}

function capabilities_panel(features) {
  let runnable = $list.filter(
    features,
    (feature) => { return feature.runnable; },
  );
  let integrated = $list.filter(
    runnable,
    (feature) => { return feature.gui_status === "integrated"; },
  );
  return panel_with_head(
    "span-12 capabilities-panel",
    "Packwand Feature Coverage",
    pill(
      (($int.to_string($list.length(integrated)) + " / ") + $int.to_string(
        $list.length(runnable),
      )) + " commands integrated",
    ),
    toList([
      $html.p(
        toList([$attribute.class$("panel-copy")]),
        toList([
          $html.text(
            "This matrix is generated from Packwand's live command tree. CLI-only commands remain available in the terminal but are not exposed as unrestricted web actions.",
          ),
        ]),
      ),
      $html.div(
        toList([$attribute.class$("feature-list")]),
        (() => {
          let _pipe = runnable;
          return $list.map(_pipe, feature_row);
        })(),
      ),
    ]),
  );
}

function manifest_raw_panel(model) {
  return panel_with_head(
    "span-12",
    "Manifest (raw JSON)",
    $html.div(
      toList([$attribute.class$("panel-actions")]),
      toList([
        button("ghost", "Form Editor", new SetManifestStructured(true)),
        button("ghost", "Save Manifest", new SaveManifest()),
      ]),
    ),
    toList([
      $html.textarea(
        toList([
          $attribute.spellcheck(false),
          $event.on_input((var0) => { return new SetManifest(var0); }),
        ]),
        model.manifest,
      ),
      notice(model.notice),
    ]),
  );
}

function issue_list(issues) {
  if (issues instanceof $Empty) {
    return $html.text("");
  } else {
    return $html.ul(
      toList([$attribute.class$("validation-list")]),
      $list.map(
        issues,
        (issue) => {
          return $html.li(
            toList([
              $attribute.class$(
                (() => {
                  let $ = issue.severity;
                  if ($ instanceof $manifest_form.IssueError) {
                    return "validation-error";
                  } else {
                    return "validation-warning";
                  }
                })(),
              ),
            ]),
            toList([$html.text(issue.message)]),
          );
        },
      ),
    );
  }
}

function validation_summary(issues) {
  if (issues instanceof $Empty) {
    return $html.p(
      toList([$attribute.class$("notice validation-ok")]),
      toList([$html.text("Manifest is valid.")]),
    );
  } else {
    return $html.div(
      toList([$attribute.class$("validation-summary")]),
      toList([
        $html.h3(toList([]), toList([$html.text("Validation")])),
        issue_list(issues),
      ]),
    );
  }
}

function automation_bool(form, get) {
  let $ = form.automation;
  if ($ instanceof Some) {
    let settings = $[0];
    let $1 = get(settings);
    if ($1 instanceof Some) {
      let $2 = $1[0];
      if ($2) {
        return "true";
      } else {
        return "false";
      }
    } else {
      return "";
    }
  } else {
    return "";
  }
}

function tri_state_select(label, value, to_field) {
  return $html.label(
    toList([]),
    toList([
      $html.span(toList([]), toList([$html.text(label)])),
      $html.select(
        toList([
          $attribute.value(value),
          $event.on_change((v) => { return new SetManifestField(to_field(v)); }),
        ]),
        toList([
          $html.option(toList([$attribute.value("")]), "default"),
          $html.option(toList([$attribute.value("true")]), "enabled"),
          $html.option(toList([$attribute.value("false")]), "disabled"),
        ]),
      ),
    ]),
  );
}

function form_input_list(label, value, placeholder, message, list_id) {
  return $html.label(
    toList([]),
    toList([
      $html.span(toList([]), toList([$html.text(label)])),
      $html.input(
        toList([
          $attribute.value(value),
          $attribute.placeholder(placeholder),
          $attribute.attribute("list", list_id),
          $event.on_input(message),
        ]),
      ),
    ]),
  );
}

function labelled_control(issues, field, label, class$, control) {
  let field_errors = $manifest_form.field_issues(issues, field);
  let _block;
  if (field_errors instanceof $Empty) {
    _block = class$;
  } else {
    _block = $string.trim(class$ + " has-error");
  }
  let error_class = _block;
  return $html.label(
    toList([$attribute.class$(error_class)]),
    listPrepend(
      $html.span(toList([]), toList([$html.text(label)])),
      listPrepend(
        control,
        $list.map(
          field_errors,
          (issue) => {
            return $html.em(
              toList([$attribute.class$("field-error")]),
              toList([$html.text(issue.message)]),
            );
          },
        ),
      ),
    ),
  );
}

function manifest_input_list(
  issues,
  field,
  label,
  value,
  placeholder,
  to_field,
  list_id
) {
  return labelled_control(
    issues,
    field,
    label,
    "",
    $html.input(
      toList([
        $attribute.value(value),
        $attribute.placeholder(placeholder),
        $attribute.attribute("list", list_id),
        $event.on_input((v) => { return new SetManifestField(to_field(v)); }),
      ]),
    ),
  );
}

function mappings_editor(form, issues) {
  return $html.div(
    toList([$attribute.class$("mappings-editor")]),
    toList([
      $html.h3(toList([]), toList([$html.text("Base Mappings")])),
      issue_list($manifest_form.field_issues(issues, "role_mappings")),
      $html.div(
        toList([$attribute.class$("list")]),
        $list.index_map(
          form.role_mappings,
          (mapping, index) => {
            return $html.div(
              toList([$attribute.class$("row mapping-row")]),
              toList([
                $html.div(
                  toList([$attribute.class$("form-grid")]),
                  toList([
                    manifest_input_list(
                      issues,
                      ("mapping[" + $int.to_string(index)) + "]",
                      "Source (in base)",
                      mapping.source,
                      "1.21.1-mr",
                      (v) => {
                        return new $manifest_form.FMappingSource(index, v);
                      },
                      "pw-subdir-keys",
                    ),
                    form_input_list(
                      "Target (this pack)",
                      mapping.target,
                      "1.21.1-mr",
                      (v) => {
                        return new SetManifestField(
                          new $manifest_form.FMappingTarget(index, v),
                        );
                      },
                      "pw-subdir-keys",
                    ),
                  ]),
                ),
                button(
                  "ghost danger",
                  "Remove",
                  new SetManifestField(new $manifest_form.FMappingRemove(index)),
                ),
              ]),
            );
          },
        ),
      ),
      button(
        "ghost",
        "Add Mapping",
        new SetManifestField(new $manifest_form.FMappingAdd()),
      ),
    ]),
  );
}

function manifest_input(
  issues,
  field,
  label,
  value,
  placeholder,
  to_field,
  class$
) {
  return labelled_control(
    issues,
    field,
    label,
    class$,
    $html.input(
      toList([
        $attribute.value(value),
        $attribute.placeholder(placeholder),
        $event.on_input((v) => { return new SetManifestField(to_field(v)); }),
      ]),
    ),
  );
}

function variants_editor(form, issues) {
  return $html.div(
    toList([$attribute.class$("variants-editor")]),
    toList([
      $html.h3(toList([]), toList([$html.text("Variants")])),
      issue_list($manifest_form.field_issues(issues, "variants")),
      $html.div(
        toList([$attribute.class$("list")]),
        $list.index_map(
          form.variants,
          (variant, index) => {
            return $html.div(
              toList([$attribute.class$("row variant-row")]),
              toList([
                $html.div(
                  toList([$attribute.class$("form-grid")]),
                  toList([
                    manifest_input(
                      issues,
                      ("variants[" + $int.to_string(index)) + "]",
                      "MC version",
                      variant.mc_version,
                      "1.21.1",
                      (v) => {
                        return new $manifest_form.FVariant(
                          index,
                          new $manifest_form.VMcVersion(v),
                        );
                      },
                      "",
                    ),
                    form_input(
                      "ID",
                      variant.id,
                      "optional",
                      (v) => {
                        return new SetManifestField(
                          new $manifest_form.FVariant(
                            index,
                            new $manifest_form.VId(v),
                          ),
                        );
                      },
                      "",
                    ),
                    form_input(
                      "Name",
                      variant.name,
                      "optional",
                      (v) => {
                        return new SetManifestField(
                          new $manifest_form.FVariant(
                            index,
                            new $manifest_form.VName(v),
                          ),
                        );
                      },
                      "",
                    ),
                    form_input(
                      "Version",
                      variant.version,
                      "optional",
                      (v) => {
                        return new SetManifestField(
                          new $manifest_form.FVariant(
                            index,
                            new $manifest_form.VVersion(v),
                          ),
                        );
                      },
                      "",
                    ),
                    form_input(
                      "Loader",
                      variant.loader,
                      "inherits pack",
                      (v) => {
                        return new SetManifestField(
                          new $manifest_form.FVariant(
                            index,
                            new $manifest_form.VLoader(v),
                          ),
                        );
                      },
                      "",
                    ),
                  ]),
                ),
                button(
                  "ghost danger",
                  "Remove",
                  new SetManifestField(new $manifest_form.FVariantRemove(index)),
                ),
              ]),
            );
          },
        ),
      ),
      button(
        "ghost",
        "Add Variant",
        new SetManifestField(new $manifest_form.FVariantAdd()),
      ),
    ]),
  );
}

function manifest_select(issues, field, label, value, options, to_field) {
  return labelled_control(
    issues,
    field,
    label,
    "",
    $html.select(
      toList([
        $attribute.value(value),
        $event.on_change((v) => { return new SetManifestField(to_field(v)); }),
      ]),
      $list.map(
        options,
        (option_value) => {
          return $html.option(
            toList([$attribute.value(option_value)]),
            (() => {
              if (option_value === "") {
                return "(unset)";
              } else {
                return option_value;
              }
            })(),
          );
        },
      ),
    ),
  );
}

function datalist(id, values) {
  return $html.datalist(
    toList([$attribute.id(id)]),
    $list.map(
      values,
      (value) => { return $html.option(toList([$attribute.value(value)]), ""); },
    ),
  );
}

function button_disabled(class$, label, message, is_disabled) {
  return $html.button(
    toList([
      $attribute.class$(class$),
      $attribute.type_("button"),
      $attribute.disabled(is_disabled),
      $attribute.aria_disabled(is_disabled),
      $event.on_click(message),
    ]),
    toList([$html.text(label)]),
  );
}

function manifest_form_panel(model, form) {
  let issues = $manifest_form.validate(form);
  let pack_ids = $list.map(model.projects, (project) => { return project.id; });
  let _block;
  let _pipe = model.projects;
  let _pipe$1 = $list.flat_map(
    _pipe,
    (project) => {
      return listPrepend(
        project.minecraft,
        $list.map(project.variants, (variant) => { return variant.minecraft; }),
      );
    },
  );
  let _pipe$2 = $list.filter(_pipe$1, (value) => { return value !== ""; });
  _block = $list.unique(_pipe$2);
  let mc_versions = _block;
  let _block$1;
  let _pipe$3 = model.projects;
  let _pipe$4 = $list.flat_map(
    _pipe$3,
    (project) => {
      return $list.map(project.subdirs, (subdir) => { return subdir.key; });
    },
  );
  let _pipe$5 = $list.filter(_pipe$4, (value) => { return value !== ""; });
  _block$1 = $list.unique(_pipe$5);
  let subdir_keys = _block$1;
  return panel_with_head(
    "span-12 manifest-form",
    "Manifest",
    $html.div(
      toList([$attribute.class$("panel-actions")]),
      toList([
        button("ghost", "Raw JSON", new SetManifestStructured(false)),
        button_disabled(
          "",
          "Save Manifest",
          new SaveManifest(),
          !isEqual($manifest_form.errors(issues), toList([])),
        ),
      ]),
    ),
    toList([
      datalist("pw-loaders", toList(["fabric", "forge", "neoforge", "quilt"])),
      datalist("pw-mc-versions", mc_versions),
      datalist("pw-pack-ids", pack_ids),
      datalist("pw-subdir-keys", subdir_keys),
      $html.h3(toList([]), toList([$html.text("Identity")])),
      $html.div(
        toList([$attribute.class$("form-grid")]),
        toList([
          manifest_input(
            issues,
            "id",
            "ID",
            form.id,
            "my-pack",
            (v) => { return new $manifest_form.FId(v); },
            "",
          ),
          manifest_input(
            issues,
            "name",
            "Name",
            form.name,
            "My Pack",
            (v) => { return new $manifest_form.FName(v); },
            "",
          ),
          manifest_select(
            issues,
            "type",
            "Type",
            form.kind,
            toList(["modpack", "datapack", "resourcepack"]),
            (v) => { return new $manifest_form.FKind(v); },
          ),
          manifest_select(
            issues,
            "release_type",
            "Release type",
            form.release_type,
            toList(["release", "beta", "alpha"]),
            (v) => { return new $manifest_form.FReleaseType(v); },
          ),
          manifest_select(
            issues,
            "lifecycle",
            "Lifecycle",
            form.lifecycle,
            toList(["", "active", "maintenance", "archived", "eol"]),
            (v) => { return new $manifest_form.FLifecycle(v); },
          ),
          manifest_input(
            issues,
            "version",
            "Version",
            form.version,
            "26.07",
            (v) => { return new $manifest_form.FVersion(v); },
            "",
          ),
          manifest_input_list(
            issues,
            "loader",
            "Loader",
            form.loader,
            "fabric",
            (v) => { return new $manifest_form.FLoader(v); },
            "pw-loaders",
          ),
        ]),
      ),
      $html.h3(toList([]), toList([$html.text("Minecraft")])),
      $html.div(
        toList([$attribute.class$("form-grid")]),
        toList([
          $html.label(
            toList([]),
            toList([
              $html.span(toList([]), toList([$html.text("Shape")])),
              $html.select(
                toList([
                  $attribute.value(
                    (() => {
                      let $ = form.use_variants;
                      if ($) {
                        return "variants";
                      } else {
                        return "single";
                      }
                    })(),
                  ),
                  $event.on_change(
                    (v) => {
                      return new SetManifestField(
                        new $manifest_form.FUseVariants(v === "variants"),
                      );
                    },
                  ),
                ]),
                toList([
                  $html.option(
                    toList([$attribute.value("single")]),
                    "single version (mc_version)",
                  ),
                  $html.option(
                    toList([$attribute.value("variants")]),
                    "multi-variant (variants)",
                  ),
                ]),
              ),
            ]),
          ),
          (() => {
            let $ = form.use_variants;
            if ($) {
              return $html.text("");
            } else {
              return manifest_input_list(
                issues,
                "mc_version",
                "Minecraft version",
                form.mc_version,
                "1.21.1",
                (v) => { return new $manifest_form.FMcVersion(v); },
                "pw-mc-versions",
              );
            }
          })(),
        ]),
      ),
      (() => {
        let $ = form.use_variants;
        if ($) {
          return variants_editor(form, issues);
        } else {
          return $html.text("");
        }
      })(),
      $html.h3(toList([]), toList([$html.text("Distribution")])),
      issue_list($manifest_form.field_issues(issues, "platforms")),
      $html.div(
        toList([$attribute.class$("form-grid")]),
        toList([
          manifest_input(
            issues,
            "modrinth_id",
            "Modrinth ID",
            form.modrinth_id,
            "",
            (v) => { return new $manifest_form.FModrinthId(v); },
            "",
          ),
          manifest_input(
            issues,
            "curseforge_id",
            "CurseForge ID",
            form.curseforge_id,
            "",
            (v) => { return new $manifest_form.FCurseforgeId(v); },
            "",
          ),
          manifest_input(
            issues,
            "github_id",
            "GitHub (owner/repo)",
            form.github_id,
            "",
            (v) => { return new $manifest_form.FGithubId(v); },
            "",
          ),
          manifest_input(
            issues,
            "gitea_id",
            "Gitea (owner/repo)",
            form.gitea_id,
            "",
            (v) => { return new $manifest_form.FGiteaId(v); },
            "",
          ),
          manifest_input(
            issues,
            "gitlab_id",
            "GitLab (owner/repo)",
            form.gitlab_id,
            "",
            (v) => { return new $manifest_form.FGitlabId(v); },
            "",
          ),
        ]),
      ),
      $html.h3(toList([]), toList([$html.text("Role & Assets")])),
      $html.div(
        toList([$attribute.class$("form-grid")]),
        toList([
          $html.label(
            toList([]),
            toList([
              $html.span(toList([]), toList([$html.text("Role")])),
              $html.select(
                toList([
                  $attribute.value(
                    (() => {
                      let $ = form.role_kind;
                      if ($ instanceof $manifest_form.RoleNone) {
                        return "none";
                      } else if ($ instanceof $manifest_form.RoleBase) {
                        return "base";
                      } else {
                        return "consumer";
                      }
                    })(),
                  ),
                  $event.on_change(
                    (v) => {
                      return new SetManifestField(
                        new $manifest_form.FRoleKind(v),
                      );
                    },
                  ),
                ]),
                toList([
                  $html.option(
                    toList([$attribute.value("none")]),
                    "none (standalone)",
                  ),
                  $html.option(
                    toList([$attribute.value("base")]),
                    "base (performance base)",
                  ),
                  $html.option(
                    toList([$attribute.value("consumer")]),
                    "consumer (uses a performance base)",
                  ),
                ]),
              ),
            ]),
          ),
          (() => {
            let $ = form.role_kind;
            if ($ instanceof $manifest_form.RoleConsumer) {
              return manifest_input_list(
                issues,
                "role_pack",
                "Base pack",
                form.role_pack,
                "performance-base-id",
                (v) => { return new $manifest_form.FRolePack(v); },
                "pw-pack-ids",
              );
            } else {
              return $html.text("");
            }
          })(),
          manifest_input_list(
            issues,
            "shared_assets",
            "Shared assets pack",
            form.shared_assets,
            "",
            (v) => { return new $manifest_form.FSharedAssets(v); },
            "pw-pack-ids",
          ),
        ]),
      ),
      (() => {
        let $ = form.role_kind;
        if ($ instanceof $manifest_form.RoleConsumer) {
          return mappings_editor(form, issues);
        } else {
          return $html.text("");
        }
      })(),
      $html.h3(toList([]), toList([$html.text("Automation")])),
      $html.div(
        toList([$attribute.class$("form-grid")]),
        toList([
          tri_state_select(
            "Auto-update",
            automation_bool(
              form,
              (settings) => { return settings.auto_update; },
            ),
            (v) => { return new $manifest_form.FAutoUpdate(v); },
          ),
          tri_state_select(
            "Server promo",
            automation_bool(
              form,
              (settings) => { return settings.server_promo; },
            ),
            (v) => { return new $manifest_form.FServerPromo(v); },
          ),
        ]),
      ),
      validation_summary(issues),
      notice(model.notice),
    ]),
  );
}

function manifest_panel(model) {
  let $ = model.manifest_structured;
  let $1 = model.manifest_form;
  if ($ && $1 instanceof Some) {
    let form = $1[0];
    return manifest_form_panel(model, form);
  } else {
    return manifest_raw_panel(model);
  }
}

function generate_panel(model) {
  let disabled = (model.selected_subdir === "") || job_running(model);
  return panel(
    "span-7",
    "Generate",
    toList([
      $html.div(
        toList([$attribute.class$("action-row")]),
        toList([
          button_disabled(
            "ghost",
            "Nix Checksums",
            new RunAction(new NixGen(model.selected_subdir)),
            disabled,
          ),
          button_disabled(
            "ghost",
            "Write Modlist",
            new RunAction(new DocsModlist(model.selected_subdir)),
            disabled,
          ),
          button_disabled(
            "ghost",
            "Regenerate Docs Pages",
            new RunAction(new DocsPages()),
            job_running(model),
          ),
        ]),
      ),
    ]),
  );
}

function bump_panel(model, project) {
  let trimmed = $string.trim(model.bump_version);
  return panel_with_head(
    "span-5",
    "Bump Version",
    button_disabled(
      "",
      "Bump",
      new RunAction(new Bump(project.dir, trimmed, model.bump_configs)),
      job_running(model) || (trimmed === ""),
    ),
    toList([
      $html.div(
        toList([$attribute.class$("form-grid")]),
        toList([
          form_input(
            "New version",
            model.bump_version,
            project.version,
            (var0) => { return new SetBumpVersion(var0); },
            "",
          ),
          $html.label(
            toList([]),
            toList([
              $html.span(
                toList([]),
                toList([$html.text("Also update in-pack configs")]),
              ),
              $html.input(
                toList([
                  $attribute.type_("checkbox"),
                  $attribute.checked(model.bump_configs),
                  $event.on_check(
                    (var0) => { return new SetBumpConfigs(var0); },
                  ),
                ]),
              ),
            ]),
          ),
        ]),
      ),
      notice(model.notice),
    ]),
  );
}

function subdir_row(subdir) {
  let _block;
  let $ = subdir.mod_count;
  if ($ === 0) {
    _block = "";
  } else {
    _block = (" - " + $int.to_string(subdir.mod_count)) + " mods";
  }
  let count = _block;
  return $html.div(
    toList([$attribute.class$("row search-item")]),
    toList([
      $html.div(
        toList([]),
        toList([
          $html.strong(toList([]), toList([$html.text(subdir.key)])),
          $html.span(toList([]), toList([$html.text(subdir.path + count)])),
        ]),
      ),
      $html.span(
        toList([]),
        toList([$html.text(fallback(subdir.platform, "content"))]),
      ),
    ]),
  );
}

function subdir_panel(model, subdirs) {
  let _block;
  let _pipe = subdirs;
  let _pipe$1 = $list.filter(
    _pipe,
    (subdir) => {
      return query_matches(
        model.search,
        (((subdir.key + " ") + subdir.path) + " ") + subdir.platform,
      );
    },
  );
  _block = $list.map(_pipe$1, subdir_row);
  let rows = _block;
  return panel_with_head(
    "span-5",
    "Subdirs",
    pill($int.to_string($list.length(subdirs)) + " subdir(s)"),
    toList([
      $html.div(
        toList([$attribute.class$("list")]),
        (() => {
          if (rows instanceof $Empty) {
            return toList([empty_row("No subdirs indexed.")]);
          } else {
            return rows;
          }
        })(),
      ),
    ]),
  );
}

function detail(label, value) {
  return $html.div(
    toList([$attribute.class$("detail")]),
    toList([
      $html.span(toList([]), toList([$html.text(label)])),
      $html.strong(
        toList([$attribute.title(value)]),
        toList([$html.text(value)]),
      ),
    ]),
  );
}

function project_panel(model, project) {
  let fields = toList([
    ["Name", project.name],
    ["Directory", project.dir],
    ["Manifest", project.manifest_path],
    ["Lifecycle", fallback(project.lifecycle, "active")],
    [
      "Auto Update",
      (() => {
        let $ = project.auto_update;
        if ($) {
          return "enabled";
        } else {
          return "disabled";
        }
      })(),
    ],
    ["Modrinth", fallback(project.modrinth_id, "-")],
    ["CurseForge", fallback(project.curseforge_id, "-")],
    ["GitHub", fallback(project.github_id, "-")],
    ["Gitea", fallback(project.gitea_id, "-")],
    ["GitLab", fallback(project.gitlab_id, "-")],
  ]);
  let _block;
  let _pipe = fields;
  let _pipe$1 = $list.filter(
    _pipe,
    (field) => {
      return query_matches(model.search, (field[0] + " ") + field[1]);
    },
  );
  _block = $list.map(_pipe$1, (field) => { return detail(field[0], field[1]); });
  let details = _block;
  return panel_with_head(
    "span-7",
    "Project",
    pill(fallback(project.role, "none")),
    toList([$html.div(toList([$attribute.class$("details")]), details)]),
  );
}

function launcher_progress_line(model) {
  let $ = model.launcher_progress;
  if ($ instanceof Some) {
    let progress = $[0];
    return $html.p(
      toList([$attribute.class$("panel-copy")]),
      toList([
        $html.text(
          ((((($int.to_string(progress.finished_downloads) + "/") + $int.to_string(
            progress.total_downloads,
          )) + " downloads, ") + $int.to_string(
            globalThis.Math.trunc(progress.downloaded_bytes / 1_048_576),
          )) + " MiB") + (() => {
            let $1 = progress.total_bytes;
            if ($1 === 0) {
              return "";
            } else {
              let total = $1;
              return ("/" + $int.to_string(
                globalThis.Math.trunc(total / 1_048_576),
              )) + " MiB";
            }
          })(),
        ),
      ]),
    );
  } else {
    return $html.text("");
  }
}

function launcher_panel(model) {
  let $ = model.launcher_status;
  let $1 = model.launcher_log;
  if ($1 instanceof $Empty && $ === "idle") {
    return $html.text("");
  } else {
    let status = $;
    let log = $1;
    return panel_with_head(
      "span-12",
      "Launcher (dev test boot)",
      pill(status),
      toList([
        launcher_progress_line(model),
        $html.pre(
          toList([]),
          toList([$html.text($string.join($list.reverse(log), "\n"))]),
        ),
      ]),
    );
  }
}

function logs_panel(model) {
  return panel_with_head(
    "span-12",
    "Command Logs",
    pill(model.job_status),
    toList([
      $html.pre(
        toList([$attribute.id("logPane")]),
        toList([$html.text($string.join($list.reverse(model.logs), "\n"))]),
      ),
    ]),
  );
}

function progress_row(item) {
  let status = progress_status_label(item.status);
  return $html.div(
    toList([$attribute.class$("row search-item")]),
    toList([
      $html.div(
        toList([]),
        toList([
          $html.strong(toList([]), toList([$html.text(item.name)])),
          $html.span(toList([]), toList([$html.text(item.detail)])),
        ]),
      ),
      $html.span(
        toList([
          $attribute.classes(
            toList([
              ["status-badge", true],
              ["integrated", (status === "pending") || (status === "queued")],
            ]),
          ),
        ]),
        toList([$html.text(status)]),
      ),
    ]),
  );
}

function progress_panel(model) {
  let $ = model.mod_progress;
  if ($ instanceof $Empty) {
    return $html.text("");
  } else {
    let items = $;
    return panel_with_head(
      "span-12",
      "Batch Progress",
      pill($int.to_string($list.length(items)) + " mod(s)"),
      toList([
        $html.div(
          toList([$attribute.class$("list mod-progress-list")]),
          $list.map(items, progress_row),
        ),
      ]),
    );
  }
}

function changelog_panel(model) {
  let _block;
  let _pipe = model.changelog;
  let _pipe$1 = $string.split(_pipe, "\n");
  let _pipe$2 = $list.filter(
    _pipe$1,
    (line) => { return query_matches(model.search, line); },
  );
  _block = $string.join(_pipe$2, "\n");
  let lines = _block;
  return panel_with_head(
    "span-12",
    "Changelog",
    button("ghost", "Copy Summary", new CopyChangelog()),
    toList([
      $html.pre(
        toList([$attribute.class$("changelog-preview")]),
        toList([
          $html.text(
            (() => {
              if (lines === "") {
                return "No changelog.md content found.";
              } else {
                return lines;
              }
            })(),
          ),
        ]),
      ),
    ]),
  );
}

function non_empty(values) {
  return $list.filter(values, (value) => { return $string.trim(value) !== ""; });
}

function mod_row(model, project, mod) {
  let subdir = model.selected_subdir;
  let _block;
  let $1 = mod.pin;
  if ($1) {
    _block = ["Unpin", new UnpinMod(subdir, mod.slug)];
  } else {
    _block = ["Pin", new PinMod(subdir, mod.slug)];
  }
  let $ = _block;
  let pin_label = $[0];
  let pin_action = $[1];
  let _block$1;
  let $3 = mod.pin;
  if ($3) {
    _block$1 = ["Unfreeze", new UnfreezeMod(subdir, mod.slug)];
  } else {
    _block$1 = ["Freeze", new FreezeMod(subdir, mod.slug)];
  }
  let $2 = _block$1;
  let freeze_label = $2[0];
  let freeze_action = $2[1];
  let side_select = $html.select(
    toList([
      $attribute.value(mod.side),
      $event.on_change(
        (side) => {
          return new RunAction(new SetSide(project.dir, mod.slug, side));
        },
      ),
    ]),
    toList([
      $html.option(toList([$attribute.value("client")]), "client"),
      $html.option(toList([$attribute.value("server")]), "server"),
      $html.option(toList([$attribute.value("both")]), "both"),
      $html.option(toList([$attribute.value("either")]), "either"),
    ]),
  );
  let _block$2;
  let $4 = mod.platform;
  let $5 = mod.version_id;
  if ($5 === "") {
    _block$2 = $html.text("");
  } else if ($4 === "curseforge") {
    let file_id = $5;
    _block$2 = button_disabled(
      "icon-btn",
      "CF Fetch",
      new RunWebview("curseforge", mod.slug, file_id),
      job_running(model),
    );
  } else if ($4 === "modrinth") {
    let file_id = $5;
    _block$2 = button_disabled(
      "icon-btn",
      "MR Fetch",
      new RunWebview("modrinth", mod.slug, file_id),
      job_running(model),
    );
  } else {
    _block$2 = $html.text("");
  }
  let webview_button = _block$2;
  return $html.div(
    toList([$attribute.class$("row search-item")]),
    toList([
      $html.div(
        toList([]),
        toList([
          $html.strong(
            toList([]),
            toList([$html.text(fallback(mod.name, mod.slug))]),
          ),
          $html.span(
            toList([]),
            toList([
              $html.text(
                (() => {
                  let _pipe = toList([
                    mod.slug,
                    mod.filename,
                    mod.side,
                    mod.platform,
                  ]);
                  let _pipe$1 = non_empty(_pipe);
                  return $string.join(_pipe$1, " / ");
                })(),
              ),
            ]),
          ),
        ]),
      ),
      webview_button,
      side_select,
      button_disabled(
        "icon-btn",
        "Update",
        new RunAction(new UpdateMod(subdir, mod.slug)),
        job_running(model),
      ),
      button_disabled(
        "icon-btn",
        pin_label,
        new RunAction(pin_action),
        job_running(model),
      ),
      button_disabled(
        "icon-btn",
        freeze_label,
        new RunAction(freeze_action),
        job_running(model),
      ),
      button_disabled(
        "icon-btn danger",
        "Remove",
        new RunAction(new RemoveMod(subdir, mod.slug)),
        job_running(model),
      ),
    ]),
  );
}

function mods_panel(model, project) {
  let _block;
  let _pipe = model.mods;
  let _pipe$1 = $list.filter(
    _pipe,
    (mod) => {
      return query_matches(
        model.search,
        (((((mod.name + " ") + mod.slug) + " ") + mod.filename) + " ") + mod.platform,
      );
    },
  );
  _block = $list.map(_pipe$1, (mod) => { return mod_row(model, project, mod); });
  let rows = _block;
  return panel_with_head(
    "span-12 mods-panel",
    "Mods",
    pill($int.to_string($list.length(model.mods)) + " mods"),
    toList([
      $html.div(
        toList([$attribute.class$("list mod-list")]),
        (() => {
          if (rows instanceof $Empty) {
            return toList([empty_row("No mods found.")]);
          } else {
            return rows;
          }
        })(),
      ),
    ]),
  );
}

function add_mod_panel(model) {
  return panel(
    "span-12 compact-panel",
    "Add Mod",
    toList([
      $html.div(
        toList([$attribute.class$("action-row")]),
        toList([
          $html.input(
            toList([
              $attribute.placeholder("mod slug..."),
              $attribute.value(model.mod_slug),
              $event.on_input((var0) => { return new SetModSlug(var0); }),
            ]),
          ),
          button_disabled(
            "",
            "Add",
            new RunAction(
              new AddMod(model.selected_subdir, $string.trim(model.mod_slug)),
            ),
            (job_running(model) || ($string.trim(model.mod_slug) === "")) || (model.selected_subdir === ""),
          ),
        ]),
      ),
    ]),
  );
}

function platform_matches(platform, expected) {
  return ((platform === expected) || ((platform === "mr") && (expected === "modrinth"))) || ((platform === "cf") && (expected === "curseforge"));
}

function selected_platform(subdirs, path) {
  let $ = $list.find(subdirs, (subdir) => { return subdir.path === path; });
  if ($ instanceof Ok) {
    let subdir = $[0];
    return subdir.platform;
  } else {
    return "";
  }
}

function actions_panel(model, subdirs) {
  let platform = selected_platform(subdirs, model.selected_subdir);
  let disabled = (model.selected_subdir === "") || job_running(model);
  return panel(
    "span-12",
    "Actions",
    toList([
      $html.div(
        toList([$attribute.class$("action-row")]),
        toList([
          $html.select(
            toList([
              $attribute.value(model.selected_subdir),
              $event.on_change((var0) => { return new SelectSubdir(var0); }),
            ]),
            (() => {
              let _pipe = subdirs;
              return $list.map(
                _pipe,
                (subdir) => {
                  return $html.option(
                    toList([$attribute.value(subdir.path)]),
                    subdir.key,
                  );
                },
              );
            })(),
          ),
          button_disabled(
            "",
            "Refresh",
            new RunAction(new RefreshSubdir(model.selected_subdir)),
            disabled,
          ),
          button_disabled(
            "",
            "Update All",
            new RunAction(new UpdateAll(model.selected_subdir)),
            disabled,
          ),
          button_disabled(
            "ghost",
            "Build",
            new RunAction(new Build(model.selected_subdir)),
            disabled,
          ),
          button_disabled(
            "ghost",
            "Rehash",
            new RunAction(new Rehash(model.selected_subdir)),
            disabled,
          ),
          button_disabled(
            "ghost",
            "Modrinth Export",
            new RunAction(new ExportModrinth(model.selected_subdir)),
            disabled || !platform_matches(platform, "modrinth"),
          ),
          button_disabled(
            "ghost",
            "CF Export",
            new RunAction(new ExportCurseforge(model.selected_subdir)),
            disabled || !platform_matches(platform, "curseforge"),
          ),
          (() => {
            let $ = launcher_running(model);
            if ($) {
              return button("ghost", "Stop", new CancelBoot());
            } else {
              return button_disabled(
                "",
                "Boot (dev test)",
                new BootPack(model.selected_subdir),
                model.selected_subdir === "",
              );
            }
          })(),
          $html.label(
            toList([$attribute.class$("dock-toggle")]),
            toList([
              $html.span(
                toList([]),
                toList([$html.text("Dock game window (Windows)")]),
              ),
              $html.input(
                toList([
                  $attribute.type_("checkbox"),
                  $attribute.checked(model.dock_game_window),
                  $event.on_check(
                    (var0) => { return new SetDockGameWindow(var0); },
                  ),
                ]),
              ),
            ]),
          ),
        ]),
      ),
    ]),
  );
}

function variant_row(variant) {
  return $html.div(
    toList([$attribute.class$("mini-row search-item")]),
    toList([
      $html.strong(
        toList([]),
        toList([$html.text(fallback(variant.id, variant.minecraft))]),
      ),
      $html.span(
        toList([]),
        toList([
          $html.text(
            (() => {
              let _pipe = toList([
                variant.minecraft,
                variant.loader,
                variant.version,
              ]);
              let _pipe$1 = non_empty(_pipe);
              return $string.join(_pipe$1, " / ");
            })(),
          ),
        ]),
      ),
    ]),
  );
}

function variant_panel(model, variants) {
  let _block;
  let _pipe = variants;
  let _pipe$1 = $list.filter(
    _pipe,
    (variant) => {
      return query_matches(
        model.search,
        (((((variant.id + " ") + variant.minecraft) + " ") + variant.loader) + " ") + variant.version,
      );
    },
  );
  _block = $list.map(_pipe$1, variant_row);
  let rows = _block;
  return panel(
    "span-12",
    "Variants",
    toList([
      $html.div(
        toList([$attribute.class$("variant-list")]),
        (() => {
          if (rows instanceof $Empty) {
            return toList([
              $html.span(
                toList([$attribute.class$("empty-note")]),
                toList([$html.text("No variants declared.")]),
              ),
            ]);
          } else {
            return rows;
          }
        })(),
      ),
    ]),
  );
}

function sections_for_view(model, project) {
  let $ = model.view;
  if ($ instanceof Overview) {
    return toList([
      project_panel(model, project),
      subdir_panel(model, project.subdirs),
      actions_panel(model, project.subdirs),
      variant_panel(model, project.variants),
    ]);
  } else if ($ instanceof Exports) {
    return toList([actions_panel(model, project.subdirs)]);
  } else if ($ instanceof Mods) {
    return toList([add_mod_panel(model), mods_panel(model, project)]);
  } else if ($ instanceof Changelog) {
    return toList([changelog_panel(model)]);
  } else if ($ instanceof Logs) {
    return toList([
      progress_panel(model),
      logs_panel(model),
      launcher_panel(model),
    ]);
  } else {
    return toList([
      project_panel(model, project),
      subdir_panel(model, project.subdirs),
      bump_panel(model, project),
      generate_panel(model),
      manifest_panel(model),
      capabilities_panel(model.features),
      new_pack_panel(model),
      account_panel(model),
    ]);
  }
}

function sections(model, project) {
  let $ = $string.trim(model.search);
  if ($ === "") {
    return sections_for_view(model, project);
  } else {
    return toList([search_results_panel(model, project)]);
  }
}

function toolbar(model, project) {
  let $ = ((model.view instanceof Overview) || (model.view instanceof Settings)) || (model.view instanceof Logs);
  if ($) {
    return $html.section(
      toList([$attribute.class$("toolbar")]),
      toList([
        button_disabled(
          "",
          "Status",
          new RunAction(new WorkspaceStatus()),
          job_running(model),
        ),
        button_disabled(
          "",
          "Doctor",
          new RunAction(new Doctor()),
          job_running(model),
        ),
        button_disabled(
          "",
          "Lint",
          new RunAction(new Lint()),
          job_running(model),
        ),
        button_disabled(
          "",
          "Check Updates",
          new RunAction(new WorkspaceUpdate(true)),
          job_running(model),
        ),
        button_disabled(
          "ghost",
          "Dry Sync",
          new RunAction(new WorkspaceSync(true)),
          job_running(model),
        ),
        (() => {
          let $1 = model.view instanceof Settings;
          if ($1) {
            return button_disabled(
              "ghost",
              "Refresh Workspace",
              new RunAction(new WorkspaceRefresh()),
              job_running(model),
            );
          } else {
            return button_disabled(
              "ghost",
              "Validate Pack",
              new RunAction(new ValidateProject(project.dir)),
              job_running(model),
            );
          }
        })(),
      ]),
    );
  } else {
    return $html.text("");
  }
}

function topbar(model, project) {
  return $html.header(
    toList([$attribute.class$("topbar")]),
    toList([
      $html.div(
        toList([]),
        toList([
          $html.h1(toList([]), toList([$html.text(project.id)])),
          $html.p(
            toList([$attribute.id("projectMeta")]),
            toList([$html.text(project_summary(project))]),
          ),
        ]),
      ),
      $html.div(
        toList([$attribute.class$("top-actions")]),
        toList([
          $html.label(
            toList([$attribute.class$("search-wrap")]),
            toList([
              $html.span(toList([]), toList([$html.text("Search")])),
              $html.input(
                toList([
                  $attribute.type_("search"),
                  $attribute.placeholder("current pack..."),
                  $attribute.value(model.search),
                  $event.on_input((var0) => { return new SetSearch(var0); }),
                ]),
              ),
            ]),
          ),
          (() => {
            let $ = $string.trim(model.search);
            if ($ === "") {
              return $html.text("");
            } else {
              return $html.span(
                toList([$attribute.class$("pill")]),
                toList([$html.text("filtering")]),
              );
            }
          })(),
          $html.img(
            toList([
              $attribute.class$("project-icon"),
              $attribute.src(
                (() => {
                  let $ = model.icon_failed;
                  if ($) {
                    return "/lucy.svg";
                  } else {
                    return ("/api/v1/packs/" + project.id) + "/icon";
                  }
                })(),
              ),
              $attribute.alt(""),
              $event.on("error", $decode.success(new IconFailed())),
            ]),
          ),
          button_disabled(
            "ghost",
            "Refresh Index",
            new RunAction(new PacksIndex()),
            job_running(model),
          ),
          button_disabled(
            "",
            "Validate Pack",
            new RunAction(new ValidateProject(project.dir)),
            job_running(model),
          ),
        ]),
      ),
    ]),
  );
}

function main_view(model) {
  let $ = selected_project(model);
  if ($ instanceof Ok) {
    let project = $[0];
    return $html.main(
      toList([]),
      toList([
        topbar(model, project),
        toolbar(model, project),
        $html.section(
          toList([$attribute.class$("grid")]),
          sections(model, project),
        ),
      ]),
    );
  } else {
    return $html.main(
      toList([]),
      toList([
        $html.header(
          toList([$attribute.class$("topbar")]),
          toList([
            $html.div(
              toList([]),
              toList([
                $html.h1(toList([]), toList([$html.text("Packwand")])),
                $html.p(toList([]), toList([$html.text("No projects indexed")])),
              ]),
            ),
          ]),
        ),
        $html.section(
          toList([$attribute.class$("grid empty-workspace")]),
          toList([
            panel_with_head(
              "span-12",
              "Projects",
              button_disabled(
                "",
                "Regenerate Index",
                new RunAction(new PacksIndex()),
                job_running(model),
              ),
              toList([
                $html.p(
                  toList([$attribute.class$("panel-copy")]),
                  toList([
                    $html.text(
                      "No projects are currently indexed. Regenerate the index or scaffold the first project below.",
                    ),
                  ]),
                ),
                notice(model.notice),
              ]),
            ),
            new_pack_panel(model),
            logs_panel(model),
          ]),
        ),
      ]),
    );
  }
}

function nav_button(current, target, label) {
  return $html.button(
    toList([
      $attribute.classes(
        toList([["nav-btn", true], ["active", isEqual(current, target)]]),
      ),
      $attribute.type_("button"),
      $event.on_click(new Navigate(target)),
    ]),
    toList([$html.text(label)]),
  );
}

function project_options(projects, selected_id) {
  let _pipe = projects;
  return $list.map(
    _pipe,
    (project) => {
      return $html.option(
        toList([
          $attribute.value(project.id),
          $attribute.selected(project.id === selected_id),
        ]),
        ((project.id + " (") + project.kind) + ")",
      );
    },
  );
}

function sidebar(model) {
  return $html.aside(
    toList([$attribute.class$("sidebar")]),
    toList([
      $html.div(
        toList([$attribute.class$("brand")]),
        toList([
          $html.img(
            toList([
              $attribute.class$("mark"),
              $attribute.src("/logo.png"),
              $attribute.alt("Packwand"),
            ]),
          ),
          $html.div(
            toList([]),
            toList([
              $html.strong(toList([]), toList([$html.text("Packwand")])),
              $html.span(
                toList([$attribute.title(model.root)]),
                toList([$html.text(model.root)]),
              ),
            ]),
          ),
        ]),
      ),
      $html.label(
        toList([
          $attribute.class$("field-label"),
          $attribute.attribute("for", "projectSelect"),
        ]),
        toList([$html.text("Current Project")]),
      ),
      $html.select(
        toList([
          $attribute.id("projectSelect"),
          $attribute.value(model.selected_id),
          $event.on_change((var0) => { return new SelectProject(var0); }),
        ]),
        project_options(model.projects, model.selected_id),
      ),
      $html.nav(
        toList([]),
        toList([
          nav_button(model.view, new Overview(), "Open"),
          nav_button(model.view, new Exports(), "Exports"),
          nav_button(model.view, new Mods(), "Mods"),
          nav_button(model.view, new Changelog(), "Changelog"),
          nav_button(model.view, new Logs(), "Logs"),
          nav_button(model.view, new Settings(), "Settings"),
        ]),
      ),
      $html.div(
        toList([$attribute.class$("sidebar-footer")]),
        toList([
          $html.div(
            toList([$attribute.class$("language-credit")]),
            toList([
              $html.img(
                toList([
                  $attribute.class$("gleam-logo"),
                  $attribute.src("/lucy.svg"),
                  $attribute.alt("Gleam"),
                ]),
              ),
              $html.span(
                toList([]),
                toList([$html.text("Frontend source in Gleam")]),
              ),
            ]),
          ),
          $html.span(
            toList([]),
            toList([$html.text("packwand " + model.version)]),
          ),
        ]),
      ),
    ]),
  );
}

export function render(model) {
  return $html.div(
    toList([
      $attribute.class$("app"),
      $attribute.data("current-view", hash(model.view)),
    ]),
    toList([sidebar(model), main_view(model)]),
  );
}
