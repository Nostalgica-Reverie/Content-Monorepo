import * as $int from "../../gleam_stdlib/gleam/int.mjs";
import * as $list from "../../gleam_stdlib/gleam/list.mjs";
import * as $option from "../../gleam_stdlib/gleam/option.mjs";
import { None } from "../../gleam_stdlib/gleam/option.mjs";
import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import { Ok, Error, toList, prepend as listPrepend, CustomType as $CustomType } from "../gleam.mjs";
import * as $manifest_form from "../packwand_gui/manifest_form.mjs";
import * as $domain from "../packwand_gui/model.mjs";

export class Overview extends $CustomType {}
export const View$Overview = () => new Overview();
export const View$isOverview = (value) => value instanceof Overview;

export class Editor extends $CustomType {}
export const View$Editor = () => new Editor();
export const View$isEditor = (value) => value instanceof Editor;

export class Instances extends $CustomType {}
export const View$Instances = () => new Instances();
export const View$isInstances = (value) => value instanceof Instances;

export class Exports extends $CustomType {}
export const View$Exports = () => new Exports();
export const View$isExports = (value) => value instanceof Exports;

export class Mods extends $CustomType {}
export const View$Mods = () => new Mods();
export const View$isMods = (value) => value instanceof Mods;

export class Changelog extends $CustomType {}
export const View$Changelog = () => new Changelog();
export const View$isChangelog = (value) => value instanceof Changelog;

export class Logs extends $CustomType {}
export const View$Logs = () => new Logs();
export const View$isLogs = (value) => value instanceof Logs;

export class Settings extends $CustomType {}
export const View$Settings = () => new Settings();
export const View$isSettings = (value) => value instanceof Settings;

export class OpenFile extends $CustomType {
  constructor(path, content, saved, kind, ref_id) {
    super();
    this.path = path;
    this.content = content;
    this.saved = saved;
    this.kind = kind;
    this.ref_id = ref_id;
  }
}
export const OpenFile$OpenFile = (path, content, saved, kind, ref_id) =>
  new OpenFile(path, content, saved, kind, ref_id);
export const OpenFile$isOpenFile = (value) => value instanceof OpenFile;
export const OpenFile$OpenFile$path = (value) => value.path;
export const OpenFile$OpenFile$0 = (value) => value.path;
export const OpenFile$OpenFile$content = (value) => value.content;
export const OpenFile$OpenFile$1 = (value) => value.content;
export const OpenFile$OpenFile$saved = (value) => value.saved;
export const OpenFile$OpenFile$2 = (value) => value.saved;
export const OpenFile$OpenFile$kind = (value) => value.kind;
export const OpenFile$OpenFile$3 = (value) => value.kind;
export const OpenFile$OpenFile$ref_id = (value) => value.ref_id;
export const OpenFile$OpenFile$4 = (value) => value.ref_id;

export class ProgressPending extends $CustomType {}
export const ModProgressStatus$ProgressPending = () => new ProgressPending();
export const ModProgressStatus$isProgressPending = (value) =>
  value instanceof ProgressPending;

export class ProgressPinned extends $CustomType {}
export const ModProgressStatus$ProgressPinned = () => new ProgressPinned();
export const ModProgressStatus$isProgressPinned = (value) =>
  value instanceof ProgressPinned;

export class ProgressFailed extends $CustomType {}
export const ModProgressStatus$ProgressFailed = () => new ProgressFailed();
export const ModProgressStatus$isProgressFailed = (value) =>
  value instanceof ProgressFailed;

export class ProgressSkipped extends $CustomType {}
export const ModProgressStatus$ProgressSkipped = () => new ProgressSkipped();
export const ModProgressStatus$isProgressSkipped = (value) =>
  value instanceof ProgressSkipped;

export class ModProgress extends $CustomType {
  constructor(name, status, detail) {
    super();
    this.name = name;
    this.status = status;
    this.detail = detail;
  }
}
export const ModProgress$ModProgress = (name, status, detail) =>
  new ModProgress(name, status, detail);
export const ModProgress$isModProgress = (value) =>
  value instanceof ModProgress;
export const ModProgress$ModProgress$name = (value) => value.name;
export const ModProgress$ModProgress$0 = (value) => value.name;
export const ModProgress$ModProgress$status = (value) => value.status;
export const ModProgress$ModProgress$1 = (value) => value.status;
export const ModProgress$ModProgress$detail = (value) => value.detail;
export const ModProgress$ModProgress$2 = (value) => value.detail;

export class NewPack extends $CustomType {
  constructor(id, name, kind, loader, minecraft, version, description) {
    super();
    this.id = id;
    this.name = name;
    this.kind = kind;
    this.loader = loader;
    this.minecraft = minecraft;
    this.version = version;
    this.description = description;
  }
}
export const NewPack$NewPack = (id, name, kind, loader, minecraft, version, description) =>
  new NewPack(id, name, kind, loader, minecraft, version, description);
export const NewPack$isNewPack = (value) => value instanceof NewPack;
export const NewPack$NewPack$id = (value) => value.id;
export const NewPack$NewPack$0 = (value) => value.id;
export const NewPack$NewPack$name = (value) => value.name;
export const NewPack$NewPack$1 = (value) => value.name;
export const NewPack$NewPack$kind = (value) => value.kind;
export const NewPack$NewPack$2 = (value) => value.kind;
export const NewPack$NewPack$loader = (value) => value.loader;
export const NewPack$NewPack$3 = (value) => value.loader;
export const NewPack$NewPack$minecraft = (value) => value.minecraft;
export const NewPack$NewPack$4 = (value) => value.minecraft;
export const NewPack$NewPack$version = (value) => value.version;
export const NewPack$NewPack$5 = (value) => value.version;
export const NewPack$NewPack$description = (value) => value.description;
export const NewPack$NewPack$6 = (value) => value.description;

export class Model extends $CustomType {
  constructor(root, version, projects, features, selected_id, selected_subdir, view, search, mods, mod_slug, changelog, manifest, manifest_form, manifest_structured, logs, job_status, refresh_mods_after_job, icon_failed, new_pack, notice, bump_version, bump_configs, mod_progress, mod_progress_in_block, launcher_session, launcher_status, launcher_log, launcher_progress, dock_game_window, auth_signed_in, auth_username, auth_status_text, editor_tree, open_files, active_path, editor_diags, editor_valid, editor_checked, completions, completion_open, completion_prefix, completion_anchor, new_file_path, preflight_status, preflight, preflight_job, pending_boot, problem_filter, collapsed_tree_groups, collapsed_tree_folders, instances) {
    super();
    this.root = root;
    this.version = version;
    this.projects = projects;
    this.features = features;
    this.selected_id = selected_id;
    this.selected_subdir = selected_subdir;
    this.view = view;
    this.search = search;
    this.mods = mods;
    this.mod_slug = mod_slug;
    this.changelog = changelog;
    this.manifest = manifest;
    this.manifest_form = manifest_form;
    this.manifest_structured = manifest_structured;
    this.logs = logs;
    this.job_status = job_status;
    this.refresh_mods_after_job = refresh_mods_after_job;
    this.icon_failed = icon_failed;
    this.new_pack = new_pack;
    this.notice = notice;
    this.bump_version = bump_version;
    this.bump_configs = bump_configs;
    this.mod_progress = mod_progress;
    this.mod_progress_in_block = mod_progress_in_block;
    this.launcher_session = launcher_session;
    this.launcher_status = launcher_status;
    this.launcher_log = launcher_log;
    this.launcher_progress = launcher_progress;
    this.dock_game_window = dock_game_window;
    this.auth_signed_in = auth_signed_in;
    this.auth_username = auth_username;
    this.auth_status_text = auth_status_text;
    this.editor_tree = editor_tree;
    this.open_files = open_files;
    this.active_path = active_path;
    this.editor_diags = editor_diags;
    this.editor_valid = editor_valid;
    this.editor_checked = editor_checked;
    this.completions = completions;
    this.completion_open = completion_open;
    this.completion_prefix = completion_prefix;
    this.completion_anchor = completion_anchor;
    this.new_file_path = new_file_path;
    this.preflight_status = preflight_status;
    this.preflight = preflight;
    this.preflight_job = preflight_job;
    this.pending_boot = pending_boot;
    this.problem_filter = problem_filter;
    this.collapsed_tree_groups = collapsed_tree_groups;
    this.collapsed_tree_folders = collapsed_tree_folders;
    this.instances = instances;
  }
}
export const Model$Model = (root, version, projects, features, selected_id, selected_subdir, view, search, mods, mod_slug, changelog, manifest, manifest_form, manifest_structured, logs, job_status, refresh_mods_after_job, icon_failed, new_pack, notice, bump_version, bump_configs, mod_progress, mod_progress_in_block, launcher_session, launcher_status, launcher_log, launcher_progress, dock_game_window, auth_signed_in, auth_username, auth_status_text, editor_tree, open_files, active_path, editor_diags, editor_valid, editor_checked, completions, completion_open, completion_prefix, completion_anchor, new_file_path, preflight_status, preflight, preflight_job, pending_boot, problem_filter, collapsed_tree_groups, collapsed_tree_folders, instances) =>
  new Model(root,
  version,
  projects,
  features,
  selected_id,
  selected_subdir,
  view,
  search,
  mods,
  mod_slug,
  changelog,
  manifest,
  manifest_form,
  manifest_structured,
  logs,
  job_status,
  refresh_mods_after_job,
  icon_failed,
  new_pack,
  notice,
  bump_version,
  bump_configs,
  mod_progress,
  mod_progress_in_block,
  launcher_session,
  launcher_status,
  launcher_log,
  launcher_progress,
  dock_game_window,
  auth_signed_in,
  auth_username,
  auth_status_text,
  editor_tree,
  open_files,
  active_path,
  editor_diags,
  editor_valid,
  editor_checked,
  completions,
  completion_open,
  completion_prefix,
  completion_anchor,
  new_file_path,
  preflight_status,
  preflight,
  preflight_job,
  pending_boot,
  problem_filter,
  collapsed_tree_groups,
  collapsed_tree_folders,
  instances);
export const Model$isModel = (value) => value instanceof Model;
export const Model$Model$root = (value) => value.root;
export const Model$Model$0 = (value) => value.root;
export const Model$Model$version = (value) => value.version;
export const Model$Model$1 = (value) => value.version;
export const Model$Model$projects = (value) => value.projects;
export const Model$Model$2 = (value) => value.projects;
export const Model$Model$features = (value) => value.features;
export const Model$Model$3 = (value) => value.features;
export const Model$Model$selected_id = (value) => value.selected_id;
export const Model$Model$4 = (value) => value.selected_id;
export const Model$Model$selected_subdir = (value) => value.selected_subdir;
export const Model$Model$5 = (value) => value.selected_subdir;
export const Model$Model$view = (value) => value.view;
export const Model$Model$6 = (value) => value.view;
export const Model$Model$search = (value) => value.search;
export const Model$Model$7 = (value) => value.search;
export const Model$Model$mods = (value) => value.mods;
export const Model$Model$8 = (value) => value.mods;
export const Model$Model$mod_slug = (value) => value.mod_slug;
export const Model$Model$9 = (value) => value.mod_slug;
export const Model$Model$changelog = (value) => value.changelog;
export const Model$Model$10 = (value) => value.changelog;
export const Model$Model$manifest = (value) => value.manifest;
export const Model$Model$11 = (value) => value.manifest;
export const Model$Model$manifest_form = (value) => value.manifest_form;
export const Model$Model$12 = (value) => value.manifest_form;
export const Model$Model$manifest_structured = (value) =>
  value.manifest_structured;
export const Model$Model$13 = (value) => value.manifest_structured;
export const Model$Model$logs = (value) => value.logs;
export const Model$Model$14 = (value) => value.logs;
export const Model$Model$job_status = (value) => value.job_status;
export const Model$Model$15 = (value) => value.job_status;
export const Model$Model$refresh_mods_after_job = (value) =>
  value.refresh_mods_after_job;
export const Model$Model$16 = (value) => value.refresh_mods_after_job;
export const Model$Model$icon_failed = (value) => value.icon_failed;
export const Model$Model$17 = (value) => value.icon_failed;
export const Model$Model$new_pack = (value) => value.new_pack;
export const Model$Model$18 = (value) => value.new_pack;
export const Model$Model$notice = (value) => value.notice;
export const Model$Model$19 = (value) => value.notice;
export const Model$Model$bump_version = (value) => value.bump_version;
export const Model$Model$20 = (value) => value.bump_version;
export const Model$Model$bump_configs = (value) => value.bump_configs;
export const Model$Model$21 = (value) => value.bump_configs;
export const Model$Model$mod_progress = (value) => value.mod_progress;
export const Model$Model$22 = (value) => value.mod_progress;
export const Model$Model$mod_progress_in_block = (value) =>
  value.mod_progress_in_block;
export const Model$Model$23 = (value) => value.mod_progress_in_block;
export const Model$Model$launcher_session = (value) => value.launcher_session;
export const Model$Model$24 = (value) => value.launcher_session;
export const Model$Model$launcher_status = (value) => value.launcher_status;
export const Model$Model$25 = (value) => value.launcher_status;
export const Model$Model$launcher_log = (value) => value.launcher_log;
export const Model$Model$26 = (value) => value.launcher_log;
export const Model$Model$launcher_progress = (value) => value.launcher_progress;
export const Model$Model$27 = (value) => value.launcher_progress;
export const Model$Model$dock_game_window = (value) => value.dock_game_window;
export const Model$Model$28 = (value) => value.dock_game_window;
export const Model$Model$auth_signed_in = (value) => value.auth_signed_in;
export const Model$Model$29 = (value) => value.auth_signed_in;
export const Model$Model$auth_username = (value) => value.auth_username;
export const Model$Model$30 = (value) => value.auth_username;
export const Model$Model$auth_status_text = (value) => value.auth_status_text;
export const Model$Model$31 = (value) => value.auth_status_text;
export const Model$Model$editor_tree = (value) => value.editor_tree;
export const Model$Model$32 = (value) => value.editor_tree;
export const Model$Model$open_files = (value) => value.open_files;
export const Model$Model$33 = (value) => value.open_files;
export const Model$Model$active_path = (value) => value.active_path;
export const Model$Model$34 = (value) => value.active_path;
export const Model$Model$editor_diags = (value) => value.editor_diags;
export const Model$Model$35 = (value) => value.editor_diags;
export const Model$Model$editor_valid = (value) => value.editor_valid;
export const Model$Model$36 = (value) => value.editor_valid;
export const Model$Model$editor_checked = (value) => value.editor_checked;
export const Model$Model$37 = (value) => value.editor_checked;
export const Model$Model$completions = (value) => value.completions;
export const Model$Model$38 = (value) => value.completions;
export const Model$Model$completion_open = (value) => value.completion_open;
export const Model$Model$39 = (value) => value.completion_open;
export const Model$Model$completion_prefix = (value) => value.completion_prefix;
export const Model$Model$40 = (value) => value.completion_prefix;
export const Model$Model$completion_anchor = (value) => value.completion_anchor;
export const Model$Model$41 = (value) => value.completion_anchor;
export const Model$Model$new_file_path = (value) => value.new_file_path;
export const Model$Model$42 = (value) => value.new_file_path;
export const Model$Model$preflight_status = (value) => value.preflight_status;
export const Model$Model$43 = (value) => value.preflight_status;
export const Model$Model$preflight = (value) => value.preflight;
export const Model$Model$44 = (value) => value.preflight;
export const Model$Model$preflight_job = (value) => value.preflight_job;
export const Model$Model$45 = (value) => value.preflight_job;
export const Model$Model$pending_boot = (value) => value.pending_boot;
export const Model$Model$46 = (value) => value.pending_boot;
export const Model$Model$problem_filter = (value) => value.problem_filter;
export const Model$Model$47 = (value) => value.problem_filter;
export const Model$Model$collapsed_tree_groups = (value) =>
  value.collapsed_tree_groups;
export const Model$Model$48 = (value) => value.collapsed_tree_groups;
export const Model$Model$collapsed_tree_folders = (value) =>
  value.collapsed_tree_folders;
export const Model$Model$49 = (value) => value.collapsed_tree_folders;
export const Model$Model$instances = (value) => value.instances;
export const Model$Model$50 = (value) => value.instances;

export class GotHealth extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotHealth = ($0) => new GotHealth($0);
export const Msg$isGotHealth = (value) => value instanceof GotHealth;
export const Msg$GotHealth$0 = (value) => value[0];

export class GotProjects extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotProjects = ($0) => new GotProjects($0);
export const Msg$isGotProjects = (value) => value instanceof GotProjects;
export const Msg$GotProjects$0 = (value) => value[0];

export class GotFeatures extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotFeatures = ($0) => new GotFeatures($0);
export const Msg$isGotFeatures = (value) => value instanceof GotFeatures;
export const Msg$GotFeatures$0 = (value) => value[0];

export class SelectProject extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SelectProject = ($0) => new SelectProject($0);
export const Msg$isSelectProject = (value) => value instanceof SelectProject;
export const Msg$SelectProject$0 = (value) => value[0];

export class SelectSubdir extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SelectSubdir = ($0) => new SelectSubdir($0);
export const Msg$isSelectSubdir = (value) => value instanceof SelectSubdir;
export const Msg$SelectSubdir$0 = (value) => value[0];

export class Navigate extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$Navigate = ($0) => new Navigate($0);
export const Msg$isNavigate = (value) => value instanceof Navigate;
export const Msg$Navigate$0 = (value) => value[0];

export class SetSearch extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetSearch = ($0) => new SetSearch($0);
export const Msg$isSetSearch = (value) => value instanceof SetSearch;
export const Msg$SetSearch$0 = (value) => value[0];

export class SetModSlug extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetModSlug = ($0) => new SetModSlug($0);
export const Msg$isSetModSlug = (value) => value instanceof SetModSlug;
export const Msg$SetModSlug$0 = (value) => value[0];

export class GotMods extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotMods = ($0) => new GotMods($0);
export const Msg$isGotMods = (value) => value instanceof GotMods;
export const Msg$GotMods$0 = (value) => value[0];

export class GotChangelog extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotChangelog = ($0) => new GotChangelog($0);
export const Msg$isGotChangelog = (value) => value instanceof GotChangelog;
export const Msg$GotChangelog$0 = (value) => value[0];

export class GotManifest extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotManifest = ($0) => new GotManifest($0);
export const Msg$isGotManifest = (value) => value instanceof GotManifest;
export const Msg$GotManifest$0 = (value) => value[0];

export class RunAction extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$RunAction = ($0) => new RunAction($0);
export const Msg$isRunAction = (value) => value instanceof RunAction;
export const Msg$RunAction$0 = (value) => value[0];

export class GotAction extends $CustomType {
  constructor($0, $1) {
    super();
    this[0] = $0;
    this[1] = $1;
  }
}
export const Msg$GotAction = ($0, $1) => new GotAction($0, $1);
export const Msg$isGotAction = (value) => value instanceof GotAction;
export const Msg$GotAction$0 = (value) => value[0];
export const Msg$GotAction$1 = (value) => value[1];

export class RunWebview extends $CustomType {
  constructor(provider, slug, file_id) {
    super();
    this.provider = provider;
    this.slug = slug;
    this.file_id = file_id;
  }
}
export const Msg$RunWebview = (provider, slug, file_id) =>
  new RunWebview(provider, slug, file_id);
export const Msg$isRunWebview = (value) => value instanceof RunWebview;
export const Msg$RunWebview$provider = (value) => value.provider;
export const Msg$RunWebview$0 = (value) => value.provider;
export const Msg$RunWebview$slug = (value) => value.slug;
export const Msg$RunWebview$1 = (value) => value.slug;
export const Msg$RunWebview$file_id = (value) => value.file_id;
export const Msg$RunWebview$2 = (value) => value.file_id;

export class WebviewStarted extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$WebviewStarted = ($0) => new WebviewStarted($0);
export const Msg$isWebviewStarted = (value) => value instanceof WebviewStarted;
export const Msg$WebviewStarted$0 = (value) => value[0];

export class JobLine extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$JobLine = ($0) => new JobLine($0);
export const Msg$isJobLine = (value) => value instanceof JobLine;
export const Msg$JobLine$0 = (value) => value[0];

export class JobFinished extends $CustomType {
  constructor($0, $1) {
    super();
    this[0] = $0;
    this[1] = $1;
  }
}
export const Msg$JobFinished = ($0, $1) => new JobFinished($0, $1);
export const Msg$isJobFinished = (value) => value instanceof JobFinished;
export const Msg$JobFinished$0 = (value) => value[0];
export const Msg$JobFinished$1 = (value) => value[1];

export class SaveManifest extends $CustomType {}
export const Msg$SaveManifest = () => new SaveManifest();
export const Msg$isSaveManifest = (value) => value instanceof SaveManifest;

export class SetManifest extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetManifest = ($0) => new SetManifest($0);
export const Msg$isSetManifest = (value) => value instanceof SetManifest;
export const Msg$SetManifest$0 = (value) => value[0];

export class SetManifestField extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetManifestField = ($0) => new SetManifestField($0);
export const Msg$isSetManifestField = (value) =>
  value instanceof SetManifestField;
export const Msg$SetManifestField$0 = (value) => value[0];

export class SetManifestStructured extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetManifestStructured = ($0) => new SetManifestStructured($0);
export const Msg$isSetManifestStructured = (value) =>
  value instanceof SetManifestStructured;
export const Msg$SetManifestStructured$0 = (value) => value[0];

export class ManifestSaved extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$ManifestSaved = ($0) => new ManifestSaved($0);
export const Msg$isManifestSaved = (value) => value instanceof ManifestSaved;
export const Msg$ManifestSaved$0 = (value) => value[0];

export class SaveChangelog extends $CustomType {}
export const Msg$SaveChangelog = () => new SaveChangelog();
export const Msg$isSaveChangelog = (value) => value instanceof SaveChangelog;

export class SetChangelog extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetChangelog = ($0) => new SetChangelog($0);
export const Msg$isSetChangelog = (value) => value instanceof SetChangelog;
export const Msg$SetChangelog$0 = (value) => value[0];

export class ChangelogSaved extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$ChangelogSaved = ($0) => new ChangelogSaved($0);
export const Msg$isChangelogSaved = (value) => value instanceof ChangelogSaved;
export const Msg$ChangelogSaved$0 = (value) => value[0];

export class CreateProject extends $CustomType {}
export const Msg$CreateProject = () => new CreateProject();
export const Msg$isCreateProject = (value) => value instanceof CreateProject;

export class ProjectCreated extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$ProjectCreated = ($0) => new ProjectCreated($0);
export const Msg$isProjectCreated = (value) => value instanceof ProjectCreated;
export const Msg$ProjectCreated$0 = (value) => value[0];

export class SetNewPackID extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewPackID = ($0) => new SetNewPackID($0);
export const Msg$isSetNewPackID = (value) => value instanceof SetNewPackID;
export const Msg$SetNewPackID$0 = (value) => value[0];

export class SetNewPackName extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewPackName = ($0) => new SetNewPackName($0);
export const Msg$isSetNewPackName = (value) => value instanceof SetNewPackName;
export const Msg$SetNewPackName$0 = (value) => value[0];

export class SetNewPackType extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewPackType = ($0) => new SetNewPackType($0);
export const Msg$isSetNewPackType = (value) => value instanceof SetNewPackType;
export const Msg$SetNewPackType$0 = (value) => value[0];

export class SetNewPackLoader extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewPackLoader = ($0) => new SetNewPackLoader($0);
export const Msg$isSetNewPackLoader = (value) =>
  value instanceof SetNewPackLoader;
export const Msg$SetNewPackLoader$0 = (value) => value[0];

export class SetNewPackMinecraft extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewPackMinecraft = ($0) => new SetNewPackMinecraft($0);
export const Msg$isSetNewPackMinecraft = (value) =>
  value instanceof SetNewPackMinecraft;
export const Msg$SetNewPackMinecraft$0 = (value) => value[0];

export class SetNewPackVersion extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewPackVersion = ($0) => new SetNewPackVersion($0);
export const Msg$isSetNewPackVersion = (value) =>
  value instanceof SetNewPackVersion;
export const Msg$SetNewPackVersion$0 = (value) => value[0];

export class SetNewPackDescription extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewPackDescription = ($0) => new SetNewPackDescription($0);
export const Msg$isSetNewPackDescription = (value) =>
  value instanceof SetNewPackDescription;
export const Msg$SetNewPackDescription$0 = (value) => value[0];

export class CopyChangelog extends $CustomType {}
export const Msg$CopyChangelog = () => new CopyChangelog();
export const Msg$isCopyChangelog = (value) => value instanceof CopyChangelog;

export class IconFailed extends $CustomType {}
export const Msg$IconFailed = () => new IconFailed();
export const Msg$isIconFailed = (value) => value instanceof IconFailed;

export class SetBumpVersion extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetBumpVersion = ($0) => new SetBumpVersion($0);
export const Msg$isSetBumpVersion = (value) => value instanceof SetBumpVersion;
export const Msg$SetBumpVersion$0 = (value) => value[0];

export class SetBumpConfigs extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetBumpConfigs = ($0) => new SetBumpConfigs($0);
export const Msg$isSetBumpConfigs = (value) => value instanceof SetBumpConfigs;
export const Msg$SetBumpConfigs$0 = (value) => value[0];

export class BootPack extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Msg$BootPack = (path) => new BootPack(path);
export const Msg$isBootPack = (value) => value instanceof BootPack;
export const Msg$BootPack$path = (value) => value.path;
export const Msg$BootPack$0 = (value) => value.path;

export class SetDockGameWindow extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetDockGameWindow = ($0) => new SetDockGameWindow($0);
export const Msg$isSetDockGameWindow = (value) =>
  value instanceof SetDockGameWindow;
export const Msg$SetDockGameWindow$0 = (value) => value[0];

export class PackBooted extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$PackBooted = ($0) => new PackBooted($0);
export const Msg$isPackBooted = (value) => value instanceof PackBooted;
export const Msg$PackBooted$0 = (value) => value[0];

export class GotLauncherEvent extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotLauncherEvent = ($0) => new GotLauncherEvent($0);
export const Msg$isGotLauncherEvent = (value) =>
  value instanceof GotLauncherEvent;
export const Msg$GotLauncherEvent$0 = (value) => value[0];

export class GotLauncherProgress extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotLauncherProgress = ($0) => new GotLauncherProgress($0);
export const Msg$isGotLauncherProgress = (value) =>
  value instanceof GotLauncherProgress;
export const Msg$GotLauncherProgress$0 = (value) => value[0];

export class CancelBoot extends $CustomType {}
export const Msg$CancelBoot = () => new CancelBoot();
export const Msg$isCancelBoot = (value) => value instanceof CancelBoot;

export class BootCancelled extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$BootCancelled = ($0) => new BootCancelled($0);
export const Msg$isBootCancelled = (value) => value instanceof BootCancelled;
export const Msg$BootCancelled$0 = (value) => value[0];

export class RequestAuthLogin extends $CustomType {}
export const Msg$RequestAuthLogin = () => new RequestAuthLogin();
export const Msg$isRequestAuthLogin = (value) =>
  value instanceof RequestAuthLogin;

export class AuthLoginStarted extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$AuthLoginStarted = ($0) => new AuthLoginStarted($0);
export const Msg$isAuthLoginStarted = (value) =>
  value instanceof AuthLoginStarted;
export const Msg$AuthLoginStarted$0 = (value) => value[0];

export class GotAuthEvent extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotAuthEvent = ($0) => new GotAuthEvent($0);
export const Msg$isGotAuthEvent = (value) => value instanceof GotAuthEvent;
export const Msg$GotAuthEvent$0 = (value) => value[0];

export class RequestAuthLogout extends $CustomType {}
export const Msg$RequestAuthLogout = () => new RequestAuthLogout();
export const Msg$isRequestAuthLogout = (value) =>
  value instanceof RequestAuthLogout;

export class AuthLogoutDone extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$AuthLogoutDone = ($0) => new AuthLogoutDone($0);
export const Msg$isAuthLogoutDone = (value) => value instanceof AuthLogoutDone;
export const Msg$AuthLogoutDone$0 = (value) => value[0];

export class GotAuthStatus extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotAuthStatus = ($0) => new GotAuthStatus($0);
export const Msg$isGotAuthStatus = (value) => value instanceof GotAuthStatus;
export const Msg$GotAuthStatus$0 = (value) => value[0];

export class GotTree extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotTree = ($0) => new GotTree($0);
export const Msg$isGotTree = (value) => value instanceof GotTree;
export const Msg$GotTree$0 = (value) => value[0];

export class ReloadTree extends $CustomType {}
export const Msg$ReloadTree = () => new ReloadTree();
export const Msg$isReloadTree = (value) => value instanceof ReloadTree;

export class OpenPath extends $CustomType {
  constructor(path, kind, ref_id) {
    super();
    this.path = path;
    this.kind = kind;
    this.ref_id = ref_id;
  }
}
export const Msg$OpenPath = (path, kind, ref_id) =>
  new OpenPath(path, kind, ref_id);
export const Msg$isOpenPath = (value) => value instanceof OpenPath;
export const Msg$OpenPath$path = (value) => value.path;
export const Msg$OpenPath$0 = (value) => value.path;
export const Msg$OpenPath$kind = (value) => value.kind;
export const Msg$OpenPath$1 = (value) => value.kind;
export const Msg$OpenPath$ref_id = (value) => value.ref_id;
export const Msg$OpenPath$2 = (value) => value.ref_id;

export class GotFileContent extends $CustomType {
  constructor(path, kind, ref_id, result) {
    super();
    this.path = path;
    this.kind = kind;
    this.ref_id = ref_id;
    this.result = result;
  }
}
export const Msg$GotFileContent = (path, kind, ref_id, result) =>
  new GotFileContent(path, kind, ref_id, result);
export const Msg$isGotFileContent = (value) => value instanceof GotFileContent;
export const Msg$GotFileContent$path = (value) => value.path;
export const Msg$GotFileContent$0 = (value) => value.path;
export const Msg$GotFileContent$kind = (value) => value.kind;
export const Msg$GotFileContent$1 = (value) => value.kind;
export const Msg$GotFileContent$ref_id = (value) => value.ref_id;
export const Msg$GotFileContent$2 = (value) => value.ref_id;
export const Msg$GotFileContent$result = (value) => value.result;
export const Msg$GotFileContent$3 = (value) => value.result;

export class SelectTab extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SelectTab = ($0) => new SelectTab($0);
export const Msg$isSelectTab = (value) => value instanceof SelectTab;
export const Msg$SelectTab$0 = (value) => value[0];

export class CloseTab extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$CloseTab = ($0) => new CloseTab($0);
export const Msg$isCloseTab = (value) => value instanceof CloseTab;
export const Msg$CloseTab$0 = (value) => value[0];

export class SetBuffer extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetBuffer = ($0) => new SetBuffer($0);
export const Msg$isSetBuffer = (value) => value instanceof SetBuffer;
export const Msg$SetBuffer$0 = (value) => value[0];

export class BufferCheckDue extends $CustomType {}
export const Msg$BufferCheckDue = () => new BufferCheckDue();
export const Msg$isBufferCheckDue = (value) => value instanceof BufferCheckDue;

export class GotCheck extends $CustomType {
  constructor(path, result) {
    super();
    this.path = path;
    this.result = result;
  }
}
export const Msg$GotCheck = (path, result) => new GotCheck(path, result);
export const Msg$isGotCheck = (value) => value instanceof GotCheck;
export const Msg$GotCheck$path = (value) => value.path;
export const Msg$GotCheck$0 = (value) => value.path;
export const Msg$GotCheck$result = (value) => value.result;
export const Msg$GotCheck$1 = (value) => value.result;

export class SaveBuffer extends $CustomType {}
export const Msg$SaveBuffer = () => new SaveBuffer();
export const Msg$isSaveBuffer = (value) => value instanceof SaveBuffer;

export class BufferSaved extends $CustomType {
  constructor(path, result) {
    super();
    this.path = path;
    this.result = result;
  }
}
export const Msg$BufferSaved = (path, result) => new BufferSaved(path, result);
export const Msg$isBufferSaved = (value) => value instanceof BufferSaved;
export const Msg$BufferSaved$path = (value) => value.path;
export const Msg$BufferSaved$0 = (value) => value.path;
export const Msg$BufferSaved$result = (value) => value.result;
export const Msg$BufferSaved$1 = (value) => value.result;

export class RequestCompletions extends $CustomType {}
export const Msg$RequestCompletions = () => new RequestCompletions();
export const Msg$isRequestCompletions = (value) =>
  value instanceof RequestCompletions;

export class GotCursor extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotCursor = ($0) => new GotCursor($0);
export const Msg$isGotCursor = (value) => value instanceof GotCursor;
export const Msg$GotCursor$0 = (value) => value[0];

export class GotCompletions extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotCompletions = ($0) => new GotCompletions($0);
export const Msg$isGotCompletions = (value) => value instanceof GotCompletions;
export const Msg$GotCompletions$0 = (value) => value[0];

export class ApplyCompletion extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$ApplyCompletion = ($0) => new ApplyCompletion($0);
export const Msg$isApplyCompletion = (value) =>
  value instanceof ApplyCompletion;
export const Msg$ApplyCompletion$0 = (value) => value[0];

export class DismissCompletions extends $CustomType {}
export const Msg$DismissCompletions = () => new DismissCompletions();
export const Msg$isDismissCompletions = (value) =>
  value instanceof DismissCompletions;

export class CopyRef extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$CopyRef = ($0) => new CopyRef($0);
export const Msg$isCopyRef = (value) => value instanceof CopyRef;
export const Msg$CopyRef$0 = (value) => value[0];

export class SetNewFilePath extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetNewFilePath = ($0) => new SetNewFilePath($0);
export const Msg$isSetNewFilePath = (value) => value instanceof SetNewFilePath;
export const Msg$SetNewFilePath$0 = (value) => value[0];

export class CreateNewFile extends $CustomType {}
export const Msg$CreateNewFile = () => new CreateNewFile();
export const Msg$isCreateNewFile = (value) => value instanceof CreateNewFile;

export class NewFileCreated extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$NewFileCreated = ($0) => new NewFileCreated($0);
export const Msg$isNewFileCreated = (value) => value instanceof NewFileCreated;
export const Msg$NewFileCreated$0 = (value) => value[0];

export class DuplicateToSibling extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$DuplicateToSibling = ($0) => new DuplicateToSibling($0);
export const Msg$isDuplicateToSibling = (value) =>
  value instanceof DuplicateToSibling;
export const Msg$DuplicateToSibling$0 = (value) => value[0];

export class FileDuplicated extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$FileDuplicated = ($0) => new FileDuplicated($0);
export const Msg$isFileDuplicated = (value) => value instanceof FileDuplicated;
export const Msg$FileDuplicated$0 = (value) => value[0];

export class RunPreflight extends $CustomType {}
export const Msg$RunPreflight = () => new RunPreflight();
export const Msg$isRunPreflight = (value) => value instanceof RunPreflight;

export class RunLocalCI extends $CustomType {}
export const Msg$RunLocalCI = () => new RunLocalCI();
export const Msg$isRunLocalCI = (value) => value instanceof RunLocalCI;

export class LocalCIStarted extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$LocalCIStarted = ($0) => new LocalCIStarted($0);
export const Msg$isLocalCIStarted = (value) => value instanceof LocalCIStarted;
export const Msg$LocalCIStarted$0 = (value) => value[0];

export class PreflightStarted extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$PreflightStarted = ($0) => new PreflightStarted($0);
export const Msg$isPreflightStarted = (value) =>
  value instanceof PreflightStarted;
export const Msg$PreflightStarted$0 = (value) => value[0];

export class GotPreflightResult extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotPreflightResult = ($0) => new GotPreflightResult($0);
export const Msg$isGotPreflightResult = (value) =>
  value instanceof GotPreflightResult;
export const Msg$GotPreflightResult$0 = (value) => value[0];

export class RequestBoot extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$RequestBoot = ($0) => new RequestBoot($0);
export const Msg$isRequestBoot = (value) => value instanceof RequestBoot;
export const Msg$RequestBoot$0 = (value) => value[0];

export class SetProblemFilter extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$SetProblemFilter = ($0) => new SetProblemFilter($0);
export const Msg$isSetProblemFilter = (value) =>
  value instanceof SetProblemFilter;
export const Msg$SetProblemFilter$0 = (value) => value[0];

export class ToggleTreeGroup extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$ToggleTreeGroup = ($0) => new ToggleTreeGroup($0);
export const Msg$isToggleTreeGroup = (value) =>
  value instanceof ToggleTreeGroup;
export const Msg$ToggleTreeGroup$0 = (value) => value[0];

export class ToggleTreeFolder extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$ToggleTreeFolder = ($0) => new ToggleTreeFolder($0);
export const Msg$isToggleTreeFolder = (value) =>
  value instanceof ToggleTreeFolder;
export const Msg$ToggleTreeFolder$0 = (value) => value[0];

export class ReloadInstances extends $CustomType {}
export const Msg$ReloadInstances = () => new ReloadInstances();
export const Msg$isReloadInstances = (value) =>
  value instanceof ReloadInstances;

export class GotInstances extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$GotInstances = ($0) => new GotInstances($0);
export const Msg$isGotInstances = (value) => value instanceof GotInstances;
export const Msg$GotInstances$0 = (value) => value[0];

const token_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_:./#-";

export function progress_status_label(status) {
  if (status instanceof ProgressPending) {
    return "queued";
  } else if (status instanceof ProgressPinned) {
    return "pinned";
  } else if (status instanceof ProgressFailed) {
    return "failed";
  } else {
    return "skipped";
  }
}

export function initial() {
  return new Model(
    "Loading repo...",
    "",
    toList([]),
    toList([]),
    "",
    "",
    new Overview(),
    "",
    toList([]),
    "",
    "",
    "",
    new None(),
    false,
    toList([]),
    "idle",
    false,
    false,
    new NewPack("", "", "modpack", "fabric", "", "0.1.0", ""),
    "",
    "",
    false,
    toList([]),
    false,
    new None(),
    "idle",
    toList([]),
    new None(),
    true,
    false,
    "",
    "",
    toList([]),
    toList([]),
    "",
    toList([]),
    true,
    false,
    toList([]),
    false,
    "",
    [0, 0],
    "",
    "idle",
    new None(),
    new None(),
    new None(),
    "all",
    toList([]),
    toList([]),
    toList([]),
  );
}

export function selected_project(model) {
  return $list.find(
    model.projects,
    (project) => { return project.id === model.selected_id; },
  );
}

export function query_matches(query, text) {
  let _block;
  let _pipe = query;
  let _pipe$1 = $string.trim(_pipe);
  _block = $string.lowercase(_pipe$1);
  let needle = _block;
  return (needle === "") || $string.contains($string.lowercase(text), needle);
}

export function append_log(model, line) {
  return new Model(
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
    listPrepend(line, model.logs),
    model.job_status,
    model.refresh_mods_after_job,
    model.icon_failed,
    model.new_pack,
    model.notice,
    model.bump_version,
    model.bump_configs,
    model.mod_progress,
    model.mod_progress_in_block,
    model.launcher_session,
    model.launcher_status,
    model.launcher_log,
    model.launcher_progress,
    model.dock_game_window,
    model.auth_signed_in,
    model.auth_username,
    model.auth_status_text,
    model.editor_tree,
    model.open_files,
    model.active_path,
    model.editor_diags,
    model.editor_valid,
    model.editor_checked,
    model.completions,
    model.completion_open,
    model.completion_prefix,
    model.completion_anchor,
    model.new_file_path,
    model.preflight_status,
    model.preflight,
    model.preflight_job,
    model.pending_boot,
    model.problem_filter,
    model.collapsed_tree_groups,
    model.collapsed_tree_folders,
    model.instances,
  );
}

export function reset_progress(model) {
  return new Model(
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
    model.bump_configs,
    toList([]),
    false,
    model.launcher_session,
    model.launcher_status,
    model.launcher_log,
    model.launcher_progress,
    model.dock_game_window,
    model.auth_signed_in,
    model.auth_username,
    model.auth_status_text,
    model.editor_tree,
    model.open_files,
    model.active_path,
    model.editor_diags,
    model.editor_valid,
    model.editor_checked,
    model.completions,
    model.completion_open,
    model.completion_prefix,
    model.completion_anchor,
    model.new_file_path,
    model.preflight_status,
    model.preflight,
    model.preflight_job,
    model.pending_boot,
    model.problem_filter,
    model.collapsed_tree_groups,
    model.collapsed_tree_folders,
    model.instances,
  );
}

function upsert_progress(model, name, status, detail) {
  let name$1 = $string.trim(name);
  let entry = new ModProgress(name$1, status, $string.trim(detail));
  let exists = $list.any(
    model.mod_progress,
    (p) => { return p.name === name$1; },
  );
  let _block;
  if (exists) {
    _block = $list.map(
      model.mod_progress,
      (p) => {
        let $ = p.name === name$1;
        if ($) {
          return entry;
        } else {
          return p;
        }
      },
    );
  } else {
    _block = $list.append(model.mod_progress, toList([entry]));
  }
  let updated = _block;
  return new Model(
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
    model.bump_configs,
    updated,
    model.mod_progress_in_block,
    model.launcher_session,
    model.launcher_status,
    model.launcher_log,
    model.launcher_progress,
    model.dock_game_window,
    model.auth_signed_in,
    model.auth_username,
    model.auth_status_text,
    model.editor_tree,
    model.open_files,
    model.active_path,
    model.editor_diags,
    model.editor_valid,
    model.editor_checked,
    model.completions,
    model.completion_open,
    model.completion_prefix,
    model.completion_anchor,
    model.new_file_path,
    model.preflight_status,
    model.preflight,
    model.preflight_job,
    model.pending_boot,
    model.problem_filter,
    model.collapsed_tree_groups,
    model.collapsed_tree_folders,
    model.instances,
  );
}

function add_pending_pair(model, line) {
  let $ = $string.split_once(line, ": ");
  if ($ instanceof Ok) {
    let name = $[0][0];
    let detail = $[0][1];
    return upsert_progress(model, name, new ProgressPending(), detail);
  } else {
    return model;
  }
}

function record_progress_prefixed(model, line) {
  let pinned_prefix = "Update skipped for pinned mod ";
  let failed_prefix = "Failed to check updates for ";
  let no_updater_prefix = "A supported update system for \"";
  let $ = $string.starts_with(line, pinned_prefix);
  if ($) {
    return upsert_progress(
      model,
      $string.drop_start(line, $string.length(pinned_prefix)),
      new ProgressPinned(),
      "",
    );
  } else {
    let $1 = $string.starts_with(line, failed_prefix);
    if ($1) {
      let rest = $string.drop_start(line, $string.length(failed_prefix));
      let $2 = $string.split_once(rest, ": ");
      if ($2 instanceof Ok) {
        let name = $2[0][0];
        let detail = $2[0][1];
        return upsert_progress(model, name, new ProgressFailed(), detail);
      } else {
        return upsert_progress(model, rest, new ProgressFailed(), "");
      }
    } else {
      let $2 = $string.starts_with(line, no_updater_prefix);
      if ($2) {
        let rest = $string.drop_start(line, $string.length(no_updater_prefix));
        let $3 = $string.split_once(rest, "\"");
        if ($3 instanceof Ok) {
          let name = $3[0][0];
          return upsert_progress(
            model,
            name,
            new ProgressSkipped(),
            "no supported update system",
          );
        } else {
          return model;
        }
      } else {
        let $3 = model.mod_progress_in_block;
        if ($3) {
          return add_pending_pair(model, line);
        } else {
          return model;
        }
      }
    }
  }
}

/**
 * Best-effort parse of packwand's `update --all` / `workspace update --all
 * --check` text output into a per-mod checklist. The CLI has no structured
 * event payload for this (see codex.md §2.2), so this matches the specific
 * line shapes cmd/update.go and workspace.go's CheckUpdatesInDir print:
 * "Updates found:" blocks of "<name>: <change>" lines, workspace check's
 * "  ~ <name>: <change>" lines, and the pinned/failed/no-updater lines.
 */
export function record_progress_line(model, raw_line) {
  let trimmed = $string.trim(raw_line);
  if (trimmed === "Updates found:") {
    return new Model(
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
      model.bump_configs,
      model.mod_progress,
      true,
      model.launcher_session,
      model.launcher_status,
      model.launcher_log,
      model.launcher_progress,
      model.dock_game_window,
      model.auth_signed_in,
      model.auth_username,
      model.auth_status_text,
      model.editor_tree,
      model.open_files,
      model.active_path,
      model.editor_diags,
      model.editor_valid,
      model.editor_checked,
      model.completions,
      model.completion_open,
      model.completion_prefix,
      model.completion_anchor,
      model.new_file_path,
      model.preflight_status,
      model.preflight,
      model.preflight_job,
      model.pending_boot,
      model.problem_filter,
      model.collapsed_tree_groups,
      model.collapsed_tree_folders,
      model.instances,
    );
  } else if (trimmed === "All files are up to date!") {
    return new Model(
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
      model.bump_configs,
      model.mod_progress,
      false,
      model.launcher_session,
      model.launcher_status,
      model.launcher_log,
      model.launcher_progress,
      model.dock_game_window,
      model.auth_signed_in,
      model.auth_username,
      model.auth_status_text,
      model.editor_tree,
      model.open_files,
      model.active_path,
      model.editor_diags,
      model.editor_valid,
      model.editor_checked,
      model.completions,
      model.completion_open,
      model.completion_prefix,
      model.completion_anchor,
      model.new_file_path,
      model.preflight_status,
      model.preflight,
      model.preflight_job,
      model.pending_boot,
      model.problem_filter,
      model.collapsed_tree_groups,
      model.collapsed_tree_folders,
      model.instances,
    );
  } else if (trimmed === "Cancelled!") {
    return new Model(
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
      model.bump_configs,
      model.mod_progress,
      false,
      model.launcher_session,
      model.launcher_status,
      model.launcher_log,
      model.launcher_progress,
      model.dock_game_window,
      model.auth_signed_in,
      model.auth_username,
      model.auth_status_text,
      model.editor_tree,
      model.open_files,
      model.active_path,
      model.editor_diags,
      model.editor_valid,
      model.editor_checked,
      model.completions,
      model.completion_open,
      model.completion_prefix,
      model.completion_anchor,
      model.new_file_path,
      model.preflight_status,
      model.preflight,
      model.preflight_job,
      model.pending_boot,
      model.problem_filter,
      model.collapsed_tree_groups,
      model.collapsed_tree_folders,
      model.instances,
    );
  } else if (trimmed === "Files updated!") {
    return new Model(
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
      model.bump_configs,
      model.mod_progress,
      false,
      model.launcher_session,
      model.launcher_status,
      model.launcher_log,
      model.launcher_progress,
      model.dock_game_window,
      model.auth_signed_in,
      model.auth_username,
      model.auth_status_text,
      model.editor_tree,
      model.open_files,
      model.active_path,
      model.editor_diags,
      model.editor_valid,
      model.editor_checked,
      model.completions,
      model.completion_open,
      model.completion_prefix,
      model.completion_anchor,
      model.new_file_path,
      model.preflight_status,
      model.preflight,
      model.preflight_job,
      model.pending_boot,
      model.problem_filter,
      model.collapsed_tree_groups,
      model.collapsed_tree_folders,
      model.instances,
    );
  } else if (trimmed === "") {
    return new Model(
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
      model.bump_configs,
      model.mod_progress,
      false,
      model.launcher_session,
      model.launcher_status,
      model.launcher_log,
      model.launcher_progress,
      model.dock_game_window,
      model.auth_signed_in,
      model.auth_username,
      model.auth_status_text,
      model.editor_tree,
      model.open_files,
      model.active_path,
      model.editor_diags,
      model.editor_valid,
      model.editor_checked,
      model.completions,
      model.completion_open,
      model.completion_prefix,
      model.completion_anchor,
      model.new_file_path,
      model.preflight_status,
      model.preflight,
      model.preflight_job,
      model.pending_boot,
      model.problem_filter,
      model.collapsed_tree_groups,
      model.collapsed_tree_folders,
      model.instances,
    );
  } else {
    let $ = $string.starts_with(trimmed, "dry-run:");
    if ($) {
      return new Model(
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
        model.bump_configs,
        model.mod_progress,
        false,
        model.launcher_session,
        model.launcher_status,
        model.launcher_log,
        model.launcher_progress,
        model.dock_game_window,
        model.auth_signed_in,
        model.auth_username,
        model.auth_status_text,
        model.editor_tree,
        model.open_files,
        model.active_path,
        model.editor_diags,
        model.editor_valid,
        model.editor_checked,
        model.completions,
        model.completion_open,
        model.completion_prefix,
        model.completion_anchor,
        model.new_file_path,
        model.preflight_status,
        model.preflight,
        model.preflight_job,
        model.pending_boot,
        model.problem_filter,
        model.collapsed_tree_groups,
        model.collapsed_tree_folders,
        model.instances,
      );
    } else {
      let $1 = $string.starts_with(trimmed, "~ ");
      if ($1) {
        return add_pending_pair(model, $string.drop_start(trimmed, 2));
      } else {
        return record_progress_prefixed(model, trimmed);
      }
    }
  }
}

export function job_running(model) {
  return (model.job_status === "starting") || (model.job_status === "running");
}

export function launcher_running(model) {
  return ((model.launcher_status === "installing") || (model.launcher_status === "starting")) || (model.launcher_status === "started");
}

export function append_launcher_log(model, line) {
  return new Model(
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
    model.bump_configs,
    model.mod_progress,
    model.mod_progress_in_block,
    model.launcher_session,
    model.launcher_status,
    listPrepend(line, model.launcher_log),
    model.launcher_progress,
    model.dock_game_window,
    model.auth_signed_in,
    model.auth_username,
    model.auth_status_text,
    model.editor_tree,
    model.open_files,
    model.active_path,
    model.editor_diags,
    model.editor_valid,
    model.editor_checked,
    model.completions,
    model.completion_open,
    model.completion_prefix,
    model.completion_anchor,
    model.new_file_path,
    model.preflight_status,
    model.preflight,
    model.preflight_job,
    model.pending_boot,
    model.problem_filter,
    model.collapsed_tree_groups,
    model.collapsed_tree_folders,
    model.instances,
  );
}

/**
 * Folds one decoded `LauncherEvent` into the model: updates status and
 * appends a human-readable log line. `kind` mirrors the Rust `LaunchEvent`
 * serde tag (`packwand-launch`'s `supervisor.rs`).
 */
export function apply_launcher_event(model, event) {
  let _block;
  let $1 = event.kind;
  if ($1 === "starting") {
    _block = ["starting", "Starting..."];
  } else if ($1 === "started") {
    _block = ["started", ("Started (pid " + $int.to_string(event.pid)) + ")"];
  } else if ($1 === "stdout") {
    _block = [model.launcher_status, event.line];
  } else if ($1 === "stderr") {
    _block = [model.launcher_status, event.line];
  } else if ($1 === "exited") {
    _block = ["exited", ("Exited (code " + $int.to_string(event.code)) + ")"];
  } else if ($1 === "failed") {
    _block = ["failed", "Failed: " + event.error];
  } else if ($1 === "cancelled") {
    _block = ["cancelled", "Cancelled"];
  } else {
    _block = [model.launcher_status, ""];
  }
  let $ = _block;
  let status = $[0];
  let line = $[1];
  let with_status = new Model(
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
    model.bump_configs,
    model.mod_progress,
    model.mod_progress_in_block,
    model.launcher_session,
    status,
    model.launcher_log,
    model.launcher_progress,
    model.dock_game_window,
    model.auth_signed_in,
    model.auth_username,
    model.auth_status_text,
    model.editor_tree,
    model.open_files,
    model.active_path,
    model.editor_diags,
    model.editor_valid,
    model.editor_checked,
    model.completions,
    model.completion_open,
    model.completion_prefix,
    model.completion_anchor,
    model.new_file_path,
    model.preflight_status,
    model.preflight,
    model.preflight_job,
    model.pending_boot,
    model.problem_filter,
    model.collapsed_tree_groups,
    model.collapsed_tree_folders,
    model.instances,
  );
  if (line === "") {
    return with_status;
  } else {
    return append_launcher_log(with_status, line);
  }
}

export function active_file(model) {
  return $list.find(
    model.open_files,
    (file) => { return file.path === model.active_path; },
  );
}

export function file_dirty(file) {
  return file.content !== file.saved;
}

/**
 * The subdir base name ("1.20.1-mr") of a repo-relative subdir path.
 */
export function sub_name(path) {
  let $ = (() => {
    let _pipe = path;
    let _pipe$1 = $string.split(_pipe, "/");
    return $list.last(_pipe$1);
  })();
  if ($ instanceof Ok) {
    let name = $[0];
    return name;
  } else {
    return path;
  }
}

/**
 * Joins the workspace root (absolute, forward-slash normalized) with a
 * repo-relative subdir path into an absolute path the Tauri launcher can
 * canonicalize.
 */
export function workspace_path(model, path) {
  let $ = model.root;
  if ($ === "") {
    return path;
  } else if (path === "") {
    return $;
  } else {
    let root = $;
    let rel = path;
    return (root + "/") + rel;
  }
}

/**
 * Which registry kind completes/validates a file at this path.
 */
export function registry_kind_for_path(path) {
  let $ = $string.starts_with(path, "config/") || $string.starts_with(
    path,
    "defaultconfigs/",
  );
  if ($) {
    return "config";
  } else {
    let $1 = $string.contains(path, "assets/");
    if ($1) {
      return "resourcepack";
    } else {
      let $2 = $string.contains(path, "data/");
      if ($2) {
        return "datapack";
      } else {
        let $3 = $string.starts_with(path, "kubejs/");
        if ($3) {
          return "kubejs";
        } else {
          return "config";
        }
      }
    }
  }
}

/**
 * Whether the check endpoint understands this file type.
 */
export function checkable_path(path) {
  return ($string.ends_with(path, ".json") || $string.ends_with(path, ".mcmeta")) || $string.ends_with(
    path,
    ".toml",
  );
}

export function json_path(path) {
  return $string.ends_with(path, ".json") || $string.ends_with(path, ".mcmeta");
}

/**
 * The reference token ending at `pos` in `content`, and its start index —
 * the client-side counterpart of the server's InferFromFile token scan.
 */
export function token_at(content, pos) {
  let before = $string.slice(content, 0, pos);
  let _block;
  let _pipe = before;
  let _pipe$1 = $string.to_graphemes(_pipe);
  let _pipe$2 = $list.reverse(_pipe$1);
  _block = $list.take_while(
    _pipe$2,
    (char) => { return $string.contains(token_chars, char); },
  );
  let reversed_token = _block;
  let _block$1;
  let _pipe$3 = reversed_token;
  let _pipe$4 = $list.reverse(_pipe$3);
  _block$1 = $string.concat(_pipe$4);
  let token = _block$1;
  return [token, pos - $list.length(reversed_token)];
}

/**
 * Updates the active file's buffer content.
 */
export function set_active_content(model, content) {
  let files = $list.map(
    model.open_files,
    (file) => {
      let $ = file.path === model.active_path;
      if ($) {
        return new OpenFile(
          file.path,
          content,
          file.saved,
          file.kind,
          file.ref_id,
        );
      } else {
        return file;
      }
    },
  );
  return new Model(
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
    model.bump_configs,
    model.mod_progress,
    model.mod_progress_in_block,
    model.launcher_session,
    model.launcher_status,
    model.launcher_log,
    model.launcher_progress,
    model.dock_game_window,
    model.auth_signed_in,
    model.auth_username,
    model.auth_status_text,
    model.editor_tree,
    files,
    model.active_path,
    model.editor_diags,
    model.editor_valid,
    model.editor_checked,
    model.completions,
    model.completion_open,
    model.completion_prefix,
    model.completion_anchor,
    model.new_file_path,
    model.preflight_status,
    model.preflight,
    model.preflight_job,
    model.pending_boot,
    model.problem_filter,
    model.collapsed_tree_groups,
    model.collapsed_tree_folders,
    model.instances,
  );
}

/**
 * Clears per-buffer state when switching or closing tabs.
 */
export function reset_buffer_state(model) {
  return new Model(
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
    model.bump_configs,
    model.mod_progress,
    model.mod_progress_in_block,
    model.launcher_session,
    model.launcher_status,
    model.launcher_log,
    model.launcher_progress,
    model.dock_game_window,
    model.auth_signed_in,
    model.auth_username,
    model.auth_status_text,
    model.editor_tree,
    model.open_files,
    model.active_path,
    toList([]),
    true,
    false,
    toList([]),
    false,
    model.completion_prefix,
    model.completion_anchor,
    model.new_file_path,
    model.preflight_status,
    model.preflight,
    model.preflight_job,
    model.pending_boot,
    model.problem_filter,
    model.collapsed_tree_groups,
    model.collapsed_tree_folders,
    model.instances,
  );
}

/**
 * The sibling subdir (the -mr/-cf counterpart) of the selected subdir, if
 * the selected project has one — target of "duplicate across subdirs"
 * (IDE.md §4.3).
 */
export function sibling_subdir(model) {
  let $ = selected_project(model);
  if ($ instanceof Ok) {
    let project = $[0];
    let $1 = $list.find(
      project.subdirs,
      (subdir) => {
        return (subdir.path !== model.selected_subdir) && (sub_name(subdir.path) !== sub_name(
          model.selected_subdir,
        ));
      },
    );
    if ($1 instanceof Ok) {
      let subdir = $1[0];
      return new Ok(sub_name(subdir.path));
    } else {
      return new Error(undefined);
    }
  } else {
    return new Error(undefined);
  }
}

export function http_error(error) {
  if (error instanceof $domain.ApiError) {
    let message = error[0];
    return message;
  } else {
    let message = error[0];
    return "The Packwand API returned invalid data: " + message;
  }
}
