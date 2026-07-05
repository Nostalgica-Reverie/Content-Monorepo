import * as $int from "../gleam_stdlib/gleam/int.mjs";
import * as $list from "../gleam_stdlib/gleam/list.mjs";
import * as $option from "../gleam_stdlib/gleam/option.mjs";
import { None, Some } from "../gleam_stdlib/gleam/option.mjs";
import * as $string from "../gleam_stdlib/gleam/string.mjs";
import * as $lustre from "../lustre/lustre.mjs";
import * as $effect from "../lustre/lustre/effect.mjs";
import { Ok, toList, Empty as $Empty, makeError } from "./gleam.mjs";
import * as $api from "./packwand_gui/api.mjs";
import {
  watchJob as watch_job,
  copyText as copy_text,
  setViewHash as set_view_hash,
  watchViewHash as watch_view_hash,
} from "./packwand_gui/ffi.mjs";
import * as $manifest_form from "./packwand_gui/manifest_form.mjs";
import * as $model from "./packwand_gui/model.mjs";
import {
  ContentResponse,
  CreatedProject,
  FeatureIndex,
  ProjectIndex,
  action_name,
  action_refreshes_mods,
} from "./packwand_gui/model.mjs";
import * as $state from "./packwand_gui/state.mjs";
import {
  CopyChangelog,
  CreateProject,
  GotAction,
  GotChangelog,
  GotFeatures,
  GotHealth,
  GotManifest,
  GotMods,
  GotProjects,
  IconFailed,
  JobFinished,
  JobLine,
  ManifestSaved,
  Model,
  Navigate,
  NewPack,
  ProjectCreated,
  RunAction,
  RunWebview,
  SaveManifest,
  SelectProject,
  SelectSubdir,
  SetBumpConfigs,
  SetBumpVersion,
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
  WebviewStarted,
  append_log,
  http_error,
  initial,
  record_progress_line,
  reset_progress,
  selected_project,
} from "./packwand_gui/state.mjs";
import * as $view from "./packwand_gui/view.mjs";

const FILEPATH = "src\\packwand_gui.gleam";

function copy_effect(value) {
  return $effect.from((_) => { return copy_text(value); });
}

function with_error(model, error) {
  let message = http_error(error);
  return [
    append_log(
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        "failed",
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        message,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      message,
    ),
    $effect.none(),
  ];
}

function create_project(model) {
  let draft = model.new_pack;
  let $ = ($string.trim(draft.id) === "") || ($string.trim(draft.name) === "");
  if ($) {
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        "A project ID and name are required.",
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else {
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        "Creating project...",
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $api.create_project(
        draft.id,
        draft.name,
        draft.kind,
        draft.loader,
        draft.minecraft,
        draft.version,
        draft.description,
        (var0) => { return new ProjectCreated(var0); },
      ),
    ];
  }
}

function load_mods(path) {
  if (path === "") {
    return $effect.none();
  } else {
    return $api.mods(path, (var0) => { return new GotMods(var0); });
  }
}

function watch_job_effect(id) {
  return $effect.from(
    (dispatch) => {
      return watch_job(
        id,
        (line) => { return dispatch(new JobLine(line)); },
        (status, error) => { return dispatch(new JobFinished(status, error)); },
      );
    },
  );
}

function set_hash_effect(value) {
  return $effect.from((_) => { return set_view_hash(value); });
}

function select_project(model) {
  let $ = selected_project(model);
  if ($ instanceof Ok) {
    let project = $[0];
    let _block;
    let $1 = project.subdirs;
    if ($1 instanceof $Empty) {
      _block = "";
    } else {
      let first = $1.head;
      _block = first.path;
    }
    let subdir = _block;
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        subdir,
        model.view,
        "",
        toList([]),
        model.mod_slug,
        "",
        "",
        new None(),
        false,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        false,
        model.new_pack,
        model.notice,
        "",
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.batch(
        toList([
          load_mods(subdir),
          $api.changelog(
            project.id,
            (var0) => { return new GotChangelog(var0); },
          ),
          $api.manifest(project.id, (var0) => { return new GotManifest(var0); }),
        ]),
      ),
    ];
  } else {
    return [model, $effect.none()];
  }
}

function select_after_projects(model, projects) {
  let _block;
  let $ = $list.find(
    projects,
    (project) => { return project.id === model.selected_id; },
  );
  if ($ instanceof Ok) {
    let project = $[0];
    _block = project.id;
  } else {
    if (projects instanceof $Empty) {
      _block = "";
    } else {
      let first = projects.head;
      _block = first.id;
    }
  }
  let selected = _block;
  return select_project(
    new Model(
      model.root,
      model.version,
      projects,
      model.features,
      selected,
      model.selected_subdir,
      model.view,
      model.search,
      model.mods,
      model.mod_slug,
      model.changelog,
      model.manifest,
      model.manifest_form,
      model.manifest_structured,
      model.logs,
      model.job_status,
      model.refresh_mods_after_job,
      model.icon_failed,
      model.new_pack,
      model.notice,
      model.bump_version,
      model.bump_configs,
      model.mod_progress,
      model.mod_progress_in_block,
    ),
  );
}

function update(model, msg) {
  if (msg instanceof GotHealth) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let health = $[0];
      return [
        new Model(
          health.root,
          health.version,
          model.projects,
          model.features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          model.mods,
          model.mod_slug,
          model.changelog,
          model.manifest,
          model.manifest_form,
          model.manifest_structured,
          model.logs,
          model.job_status,
          model.refresh_mods_after_job,
          model.icon_failed,
          model.new_pack,
          model.notice,
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        $effect.none(),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof GotProjects) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let projects = $[0].projects;
      return select_after_projects(model, projects);
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof GotFeatures) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let version = $[0].packwand_version;
      let features = $[0].features;
      return [
        new Model(
          model.root,
          (() => {
            let $1 = model.version;
            if ($1 === "") {
              return version;
            } else {
              return model.version;
            }
          })(),
          model.projects,
          features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          model.mods,
          model.mod_slug,
          model.changelog,
          model.manifest,
          model.manifest_form,
          model.manifest_structured,
          model.logs,
          model.job_status,
          model.refresh_mods_after_job,
          model.icon_failed,
          model.new_pack,
          model.notice,
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        $effect.none(),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof SelectProject) {
    let id = msg[0];
    return select_project(
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        false,
        model.new_pack,
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
    );
  } else if (msg instanceof SelectSubdir) {
    let path = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        path,
        model.view,
        model.search,
        toList([]),
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      load_mods(path),
    ];
  } else if (msg instanceof Navigate) {
    let next = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        next,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      set_hash_effect($view.hash(next)),
    ];
  } else if (msg instanceof SetSearch) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        value,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetModSlug) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        value,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof GotMods) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let mods = $[0];
      return [
        new Model(
          model.root,
          model.version,
          model.projects,
          model.features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          mods,
          model.mod_slug,
          model.changelog,
          model.manifest,
          model.manifest_form,
          model.manifest_structured,
          model.logs,
          model.job_status,
          model.refresh_mods_after_job,
          model.icon_failed,
          model.new_pack,
          model.notice,
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        $effect.none(),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof GotChangelog) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let content = $[0].content;
      return [
        new Model(
          model.root,
          model.version,
          model.projects,
          model.features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          model.mods,
          model.mod_slug,
          content,
          model.manifest,
          model.manifest_form,
          model.manifest_structured,
          model.logs,
          model.job_status,
          model.refresh_mods_after_job,
          model.icon_failed,
          model.new_pack,
          model.notice,
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        $effect.none(),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof GotManifest) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let content = $[0].content;
      let $1 = $manifest_form.parse(content);
      if ($1 instanceof Ok) {
        let form = $1[0];
        return [
          new Model(
            model.root,
            model.version,
            model.projects,
            model.features,
            model.selected_id,
            model.selected_subdir,
            model.view,
            model.search,
            model.mods,
            model.mod_slug,
            model.changelog,
            content,
            new Some(form),
            true,
            model.logs,
            model.job_status,
            model.refresh_mods_after_job,
            model.icon_failed,
            model.new_pack,
            model.notice,
            model.bump_version,
            model.bump_configs,
            model.mod_progress,
            model.mod_progress_in_block,
          ),
          $effect.none(),
        ];
      } else {
        return [
          new Model(
            model.root,
            model.version,
            model.projects,
            model.features,
            model.selected_id,
            model.selected_subdir,
            model.view,
            model.search,
            model.mods,
            model.mod_slug,
            model.changelog,
            content,
            new None(),
            false,
            model.logs,
            model.job_status,
            model.refresh_mods_after_job,
            model.icon_failed,
            model.new_pack,
            model.notice,
            model.bump_version,
            model.bump_configs,
            model.mod_progress,
            model.mod_progress_in_block,
          ),
          $effect.none(),
        ];
      }
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof RunAction) {
    let action = msg[0];
    let _block;
    let _pipe = model;
    let _pipe$1 = reset_progress(_pipe);
    _block = append_log(_pipe$1, "> packwand " + action_name(action));
    let running = _block;
    return [
      new Model(
        running.root,
        running.version,
        running.projects,
        running.features,
        running.selected_id,
        running.selected_subdir,
        running.view,
        running.search,
        running.mods,
        running.mod_slug,
        running.changelog,
        running.manifest,
        running.manifest_form,
        running.manifest_structured,
        running.logs,
        "starting",
        running.refresh_mods_after_job,
        running.icon_failed,
        running.new_pack,
        "",
        running.bump_version,
        running.bump_configs,
        running.mod_progress,
        running.mod_progress_in_block,
      ),
      $api.action(action, (result) => { return new GotAction(action, result); }),
    ];
  } else if (msg instanceof GotAction) {
    let $ = msg[1];
    if ($ instanceof Ok) {
      let action = msg[0];
      let response = $[0];
      return [
        new Model(
          model.root,
          model.version,
          model.projects,
          model.features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          model.mods,
          model.mod_slug,
          model.changelog,
          model.manifest,
          model.manifest_form,
          model.manifest_structured,
          model.logs,
          "running",
          action_refreshes_mods(action),
          model.icon_failed,
          model.new_pack,
          model.notice,
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        watch_job_effect(response.job_id),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof RunWebview) {
    let provider = msg.provider;
    let slug = msg.slug;
    let file_id = msg.file_id;
    let _block;
    let _pipe = model;
    _block = append_log(
      _pipe,
      (("> mod_browser_webview --provider " + provider) + " ") + slug,
    );
    let running = _block;
    return [
      new Model(
        running.root,
        running.version,
        running.projects,
        running.features,
        running.selected_id,
        running.selected_subdir,
        running.view,
        running.search,
        running.mods,
        running.mod_slug,
        running.changelog,
        running.manifest,
        running.manifest_form,
        running.manifest_structured,
        running.logs,
        "starting",
        running.refresh_mods_after_job,
        running.icon_failed,
        running.new_pack,
        "",
        running.bump_version,
        running.bump_configs,
        running.mod_progress,
        running.mod_progress_in_block,
      ),
      $api.webview_fetch(
        provider,
        slug,
        file_id,
        (var0) => { return new WebviewStarted(var0); },
      ),
    ];
  } else if (msg instanceof WebviewStarted) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let response = $[0];
      return [
        new Model(
          model.root,
          model.version,
          model.projects,
          model.features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          model.mods,
          model.mod_slug,
          model.changelog,
          model.manifest,
          model.manifest_form,
          model.manifest_structured,
          model.logs,
          "running",
          false,
          model.icon_failed,
          model.new_pack,
          model.notice,
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        watch_job_effect(response.job_id),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof JobLine) {
    let line = msg[0];
    return [record_progress_line(append_log(model, line), line), $effect.none()];
  } else if (msg instanceof JobFinished) {
    let status = msg[0];
    let error = msg[1];
    let finished = new Model(
      model.root,
      model.version,
      model.projects,
      model.features,
      model.selected_id,
      model.selected_subdir,
      model.view,
      model.search,
      model.mods,
      model.mod_slug,
      model.changelog,
      model.manifest,
      model.manifest_form,
      model.manifest_structured,
      model.logs,
      status,
      model.refresh_mods_after_job,
      model.icon_failed,
      model.new_pack,
      model.notice,
      model.bump_version,
      model.bump_configs,
      model.mod_progress,
      model.mod_progress_in_block,
    );
    let _block;
    if (error === "") {
      _block = finished;
    } else {
      _block = append_log(finished, error);
    }
    let finished$1 = _block;
    return [
      new Model(
        finished$1.root,
        finished$1.version,
        finished$1.projects,
        finished$1.features,
        finished$1.selected_id,
        finished$1.selected_subdir,
        finished$1.view,
        finished$1.search,
        finished$1.mods,
        finished$1.mod_slug,
        finished$1.changelog,
        finished$1.manifest,
        finished$1.manifest_form,
        finished$1.manifest_structured,
        finished$1.logs,
        finished$1.job_status,
        false,
        finished$1.icon_failed,
        finished$1.new_pack,
        finished$1.notice,
        finished$1.bump_version,
        finished$1.bump_configs,
        finished$1.mod_progress,
        finished$1.mod_progress_in_block,
      ),
      (() => {
        let $ = model.refresh_mods_after_job;
        if ($) {
          return load_mods(model.selected_subdir);
        } else {
          return $effect.none();
        }
      })(),
    ];
  } else if (msg instanceof SaveManifest) {
    let $ = model.selected_id;
    if ($ === "") {
      return [model, $effect.none()];
    } else {
      let id = $;
      let $1 = model.manifest_structured;
      let $2 = model.manifest_form;
      if ($1 && $2 instanceof Some) {
        let form = $2[0];
        let issues = $manifest_form.validate(form);
        let $3 = $manifest_form.errors(issues);
        if ($3 instanceof $Empty) {
          let raw = $manifest_form.serialize(form);
          return [
            new Model(
              model.root,
              model.version,
              model.projects,
              model.features,
              model.selected_id,
              model.selected_subdir,
              model.view,
              model.search,
              model.mods,
              model.mod_slug,
              model.changelog,
              raw,
              model.manifest_form,
              model.manifest_structured,
              model.logs,
              model.job_status,
              model.refresh_mods_after_job,
              model.icon_failed,
              model.new_pack,
              "Saving manifest...",
              model.bump_version,
              model.bump_configs,
              model.mod_progress,
              model.mod_progress_in_block,
            ),
            $api.save_manifest(
              id,
              raw,
              (var0) => { return new ManifestSaved(var0); },
            ),
          ];
        } else {
          let errors = $3;
          return [
            new Model(
              model.root,
              model.version,
              model.projects,
              model.features,
              model.selected_id,
              model.selected_subdir,
              model.view,
              model.search,
              model.mods,
              model.mod_slug,
              model.changelog,
              model.manifest,
              model.manifest_form,
              model.manifest_structured,
              model.logs,
              model.job_status,
              model.refresh_mods_after_job,
              model.icon_failed,
              model.new_pack,
              ("Fix " + $int.to_string($list.length(errors))) + " validation error(s) before saving.",
              model.bump_version,
              model.bump_configs,
              model.mod_progress,
              model.mod_progress_in_block,
            ),
            $effect.none(),
          ];
        }
      } else {
        return [
          new Model(
            model.root,
            model.version,
            model.projects,
            model.features,
            model.selected_id,
            model.selected_subdir,
            model.view,
            model.search,
            model.mods,
            model.mod_slug,
            model.changelog,
            model.manifest,
            model.manifest_form,
            model.manifest_structured,
            model.logs,
            model.job_status,
            model.refresh_mods_after_job,
            model.icon_failed,
            model.new_pack,
            "Saving manifest...",
            model.bump_version,
            model.bump_configs,
            model.mod_progress,
            model.mod_progress_in_block,
          ),
          $api.save_manifest(
            id,
            model.manifest,
            (var0) => { return new ManifestSaved(var0); },
          ),
        ];
      }
    }
  } else if (msg instanceof SetManifest) {
    let content = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        content,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetManifestField) {
    let field = msg[0];
    let $ = model.manifest_form;
    if ($ instanceof Some) {
      let form = $[0];
      return [
        new Model(
          model.root,
          model.version,
          model.projects,
          model.features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          model.mods,
          model.mod_slug,
          model.changelog,
          model.manifest,
          new Some($manifest_form.apply(form, field)),
          model.manifest_structured,
          model.logs,
          model.job_status,
          model.refresh_mods_after_job,
          model.icon_failed,
          model.new_pack,
          "",
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        $effect.none(),
      ];
    } else {
      return [model, $effect.none()];
    }
  } else if (msg instanceof SetManifestStructured) {
    let $ = msg[0];
    if ($) {
      let $1 = $manifest_form.parse(model.manifest);
      if ($1 instanceof Ok) {
        let form = $1[0];
        return [
          new Model(
            model.root,
            model.version,
            model.projects,
            model.features,
            model.selected_id,
            model.selected_subdir,
            model.view,
            model.search,
            model.mods,
            model.mod_slug,
            model.changelog,
            model.manifest,
            new Some(form),
            true,
            model.logs,
            model.job_status,
            model.refresh_mods_after_job,
            model.icon_failed,
            model.new_pack,
            "",
            model.bump_version,
            model.bump_configs,
            model.mod_progress,
            model.mod_progress_in_block,
          ),
          $effect.none(),
        ];
      } else {
        let message = $1[0];
        return [
          new Model(
            model.root,
            model.version,
            model.projects,
            model.features,
            model.selected_id,
            model.selected_subdir,
            model.view,
            model.search,
            model.mods,
            model.mod_slug,
            model.changelog,
            model.manifest,
            model.manifest_form,
            model.manifest_structured,
            model.logs,
            model.job_status,
            model.refresh_mods_after_job,
            model.icon_failed,
            model.new_pack,
            "Cannot open form editor: " + message,
            model.bump_version,
            model.bump_configs,
            model.mod_progress,
            model.mod_progress_in_block,
          ),
          $effect.none(),
        ];
      }
    } else {
      let _block;
      let $1 = model.manifest_form;
      if ($1 instanceof Some) {
        let form = $1[0];
        _block = $manifest_form.serialize(form);
      } else {
        _block = model.manifest;
      }
      let raw = _block;
      return [
        new Model(
          model.root,
          model.version,
          model.projects,
          model.features,
          model.selected_id,
          model.selected_subdir,
          model.view,
          model.search,
          model.mods,
          model.mod_slug,
          model.changelog,
          raw,
          model.manifest_form,
          false,
          model.logs,
          model.job_status,
          model.refresh_mods_after_job,
          model.icon_failed,
          model.new_pack,
          "",
          model.bump_version,
          model.bump_configs,
          model.mod_progress,
          model.mod_progress_in_block,
        ),
        $effect.none(),
      ];
    }
  } else if (msg instanceof ManifestSaved) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      return [
        append_log(
          new Model(
            model.root,
            model.version,
            model.projects,
            model.features,
            model.selected_id,
            model.selected_subdir,
            model.view,
            model.search,
            model.mods,
            model.mod_slug,
            model.changelog,
            model.manifest,
            model.manifest_form,
            model.manifest_structured,
            model.logs,
            model.job_status,
            model.refresh_mods_after_job,
            model.icon_failed,
            model.new_pack,
            "Manifest saved.",
            model.bump_version,
            model.bump_configs,
            model.mod_progress,
            model.mod_progress_in_block,
          ),
          "Manifest saved.",
        ),
        $api.projects((var0) => { return new GotProjects(var0); }),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof CreateProject) {
    return create_project(model);
  } else if (msg instanceof ProjectCreated) {
    let $ = msg[0];
    if ($ instanceof Ok) {
      let id = $[0].id;
      return [
        append_log(
          new Model(
            model.root,
            model.version,
            model.projects,
            model.features,
            id,
            model.selected_subdir,
            model.view,
            model.search,
            model.mods,
            model.mod_slug,
            model.changelog,
            model.manifest,
            model.manifest_form,
            model.manifest_structured,
            model.logs,
            model.job_status,
            model.refresh_mods_after_job,
            model.icon_failed,
            model.new_pack,
            "Project created.",
            model.bump_version,
            model.bump_configs,
            model.mod_progress,
            model.mod_progress_in_block,
          ),
          ("Created project " + id) + ".",
        ),
        $api.projects((var0) => { return new GotProjects(var0); }),
      ];
    } else {
      let error = $[0];
      return with_error(model, error);
    }
  } else if (msg instanceof SetNewPackID) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        (() => {
          let _record = model.new_pack;
          return new NewPack(
            value,
            _record.name,
            _record.kind,
            _record.loader,
            _record.minecraft,
            _record.version,
            _record.description,
          );
        })(),
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetNewPackName) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        (() => {
          let _record = model.new_pack;
          return new NewPack(
            _record.id,
            value,
            _record.kind,
            _record.loader,
            _record.minecraft,
            _record.version,
            _record.description,
          );
        })(),
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetNewPackType) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        (() => {
          let _record = model.new_pack;
          return new NewPack(
            _record.id,
            _record.name,
            value,
            _record.loader,
            _record.minecraft,
            _record.version,
            _record.description,
          );
        })(),
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetNewPackLoader) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        (() => {
          let _record = model.new_pack;
          return new NewPack(
            _record.id,
            _record.name,
            _record.kind,
            value,
            _record.minecraft,
            _record.version,
            _record.description,
          );
        })(),
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetNewPackMinecraft) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        (() => {
          let _record = model.new_pack;
          return new NewPack(
            _record.id,
            _record.name,
            _record.kind,
            _record.loader,
            value,
            _record.version,
            _record.description,
          );
        })(),
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetNewPackVersion) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        (() => {
          let _record = model.new_pack;
          return new NewPack(
            _record.id,
            _record.name,
            _record.kind,
            _record.loader,
            _record.minecraft,
            value,
            _record.description,
          );
        })(),
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetNewPackDescription) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        (() => {
          let _record = model.new_pack;
          return new NewPack(
            _record.id,
            _record.name,
            _record.kind,
            _record.loader,
            _record.minecraft,
            _record.version,
            value,
          );
        })(),
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof CopyChangelog) {
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        "Changelog copied.",
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      copy_effect(model.changelog),
    ];
  } else if (msg instanceof IconFailed) {
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        true,
        model.new_pack,
        model.notice,
        model.bump_version,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else if (msg instanceof SetBumpVersion) {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        model.notice,
        value,
        model.bump_configs,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  } else {
    let value = msg[0];
    return [
      new Model(
        model.root,
        model.version,
        model.projects,
        model.features,
        model.selected_id,
        model.selected_subdir,
        model.view,
        model.search,
        model.mods,
        model.mod_slug,
        model.changelog,
        model.manifest,
        model.manifest_form,
        model.manifest_structured,
        model.logs,
        model.job_status,
        model.refresh_mods_after_job,
        model.icon_failed,
        model.new_pack,
        model.notice,
        model.bump_version,
        value,
        model.mod_progress,
        model.mod_progress_in_block,
      ),
      $effect.none(),
    ];
  }
}

function browser_view_effect() {
  return $effect.from(
    (dispatch) => {
      watch_view_hash(
        (value) => { return dispatch(new Navigate($view.from_name(value))); },
      );
      return dispatch(new Navigate($view.from_hash()));
    },
  );
}

function init(_) {
  return [
    initial(),
    $effect.batch(
      toList([
        $api.health((var0) => { return new GotHealth(var0); }),
        $api.projects((var0) => { return new GotProjects(var0); }),
        $api.features((var0) => { return new GotFeatures(var0); }),
        browser_view_effect(),
      ]),
    ),
  ];
}

export function main() {
  let app = $lustre.application(init, update, $view.render);
  let $ = $lustre.start(app, "#app", undefined);
  if (!($ instanceof Ok)) {
    throw makeError(
      "let_assert",
      FILEPATH,
      "packwand_gui",
      45,
      "main",
      "Pattern match failed, no pattern matched the value.",
      {
        value: $,
        start: 1651,
        end: 1700,
        pattern_start: 1662,
        pattern_end: 1667
      }
    )
  }
  return undefined;
}
