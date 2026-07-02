import * as $list from "../../gleam_stdlib/gleam/list.mjs";
import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import { toList, prepend as listPrepend, CustomType as $CustomType } from "../gleam.mjs";
import * as $domain from "../packwand_gui/model.mjs";

export class Overview extends $CustomType {}
export const View$Overview = () => new Overview();
export const View$isOverview = (value) => value instanceof Overview;

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
  constructor(root, version, projects, features, selected_id, selected_subdir, view, search, mods, mod_slug, changelog, manifest, logs, job_status, refresh_mods_after_job, icon_failed, new_pack, notice) {
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
    this.logs = logs;
    this.job_status = job_status;
    this.refresh_mods_after_job = refresh_mods_after_job;
    this.icon_failed = icon_failed;
    this.new_pack = new_pack;
    this.notice = notice;
  }
}
export const Model$Model = (root, version, projects, features, selected_id, selected_subdir, view, search, mods, mod_slug, changelog, manifest, logs, job_status, refresh_mods_after_job, icon_failed, new_pack, notice) =>
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
  logs,
  job_status,
  refresh_mods_after_job,
  icon_failed,
  new_pack,
  notice);
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
export const Model$Model$logs = (value) => value.logs;
export const Model$Model$12 = (value) => value.logs;
export const Model$Model$job_status = (value) => value.job_status;
export const Model$Model$13 = (value) => value.job_status;
export const Model$Model$refresh_mods_after_job = (value) =>
  value.refresh_mods_after_job;
export const Model$Model$14 = (value) => value.refresh_mods_after_job;
export const Model$Model$icon_failed = (value) => value.icon_failed;
export const Model$Model$15 = (value) => value.icon_failed;
export const Model$Model$new_pack = (value) => value.new_pack;
export const Model$Model$16 = (value) => value.new_pack;
export const Model$Model$notice = (value) => value.notice;
export const Model$Model$17 = (value) => value.notice;

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
  constructor(slug, file_id) {
    super();
    this.slug = slug;
    this.file_id = file_id;
  }
}
export const Msg$RunWebview = (slug, file_id) => new RunWebview(slug, file_id);
export const Msg$isRunWebview = (value) => value instanceof RunWebview;
export const Msg$RunWebview$slug = (value) => value.slug;
export const Msg$RunWebview$0 = (value) => value.slug;
export const Msg$RunWebview$file_id = (value) => value.file_id;
export const Msg$RunWebview$1 = (value) => value.file_id;

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

export class ManifestSaved extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Msg$ManifestSaved = ($0) => new ManifestSaved($0);
export const Msg$isManifestSaved = (value) => value instanceof ManifestSaved;
export const Msg$ManifestSaved$0 = (value) => value[0];

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
    toList([]),
    "idle",
    false,
    false,
    new NewPack("", "", "modpack", "fabric", "", "0.1.0", ""),
    "",
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
    listPrepend(line, model.logs),
    model.job_status,
    model.refresh_mods_after_job,
    model.icon_failed,
    model.new_pack,
    model.notice,
  );
}

export function job_running(model) {
  return (model.job_status === "starting") || (model.job_status === "running");
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
