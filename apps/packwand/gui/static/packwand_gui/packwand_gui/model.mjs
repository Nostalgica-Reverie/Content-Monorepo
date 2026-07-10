import * as $decode from "../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $list from "../../gleam_stdlib/gleam/list.mjs";
import * as $option from "../../gleam_stdlib/gleam/option.mjs";
import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import { toList, CustomType as $CustomType } from "../gleam.mjs";

export class Health extends $CustomType {
  constructor(root, version) {
    super();
    this.root = root;
    this.version = version;
  }
}
export const Health$Health = (root, version) => new Health(root, version);
export const Health$isHealth = (value) => value instanceof Health;
export const Health$Health$root = (value) => value.root;
export const Health$Health$0 = (value) => value.root;
export const Health$Health$version = (value) => value.version;
export const Health$Health$1 = (value) => value.version;

export class ApiError extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const ApiError$ApiError = ($0) => new ApiError($0);
export const ApiError$isApiError = (value) => value instanceof ApiError;
export const ApiError$ApiError$0 = (value) => value[0];

export class DecodeError extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const ApiError$DecodeError = ($0) => new DecodeError($0);
export const ApiError$isDecodeError = (value) => value instanceof DecodeError;
export const ApiError$DecodeError$0 = (value) => value[0];

export class ProjectIndex extends $CustomType {
  constructor(projects) {
    super();
    this.projects = projects;
  }
}
export const ProjectIndex$ProjectIndex = (projects) =>
  new ProjectIndex(projects);
export const ProjectIndex$isProjectIndex = (value) =>
  value instanceof ProjectIndex;
export const ProjectIndex$ProjectIndex$projects = (value) => value.projects;
export const ProjectIndex$ProjectIndex$0 = (value) => value.projects;

export class Project extends $CustomType {
  constructor(id, name, kind, dir, manifest_path, version, minecraft, loader, release_type, lifecycle, role, auto_update, modrinth_id, curseforge_id, github_id, gitea_id, gitlab_id, docs_path, variants, subdirs) {
    super();
    this.id = id;
    this.name = name;
    this.kind = kind;
    this.dir = dir;
    this.manifest_path = manifest_path;
    this.version = version;
    this.minecraft = minecraft;
    this.loader = loader;
    this.release_type = release_type;
    this.lifecycle = lifecycle;
    this.role = role;
    this.auto_update = auto_update;
    this.modrinth_id = modrinth_id;
    this.curseforge_id = curseforge_id;
    this.github_id = github_id;
    this.gitea_id = gitea_id;
    this.gitlab_id = gitlab_id;
    this.docs_path = docs_path;
    this.variants = variants;
    this.subdirs = subdirs;
  }
}
export const Project$Project = (id, name, kind, dir, manifest_path, version, minecraft, loader, release_type, lifecycle, role, auto_update, modrinth_id, curseforge_id, github_id, gitea_id, gitlab_id, docs_path, variants, subdirs) =>
  new Project(id,
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
  subdirs);
export const Project$isProject = (value) => value instanceof Project;
export const Project$Project$id = (value) => value.id;
export const Project$Project$0 = (value) => value.id;
export const Project$Project$name = (value) => value.name;
export const Project$Project$1 = (value) => value.name;
export const Project$Project$kind = (value) => value.kind;
export const Project$Project$2 = (value) => value.kind;
export const Project$Project$dir = (value) => value.dir;
export const Project$Project$3 = (value) => value.dir;
export const Project$Project$manifest_path = (value) => value.manifest_path;
export const Project$Project$4 = (value) => value.manifest_path;
export const Project$Project$version = (value) => value.version;
export const Project$Project$5 = (value) => value.version;
export const Project$Project$minecraft = (value) => value.minecraft;
export const Project$Project$6 = (value) => value.minecraft;
export const Project$Project$loader = (value) => value.loader;
export const Project$Project$7 = (value) => value.loader;
export const Project$Project$release_type = (value) => value.release_type;
export const Project$Project$8 = (value) => value.release_type;
export const Project$Project$lifecycle = (value) => value.lifecycle;
export const Project$Project$9 = (value) => value.lifecycle;
export const Project$Project$role = (value) => value.role;
export const Project$Project$10 = (value) => value.role;
export const Project$Project$auto_update = (value) => value.auto_update;
export const Project$Project$11 = (value) => value.auto_update;
export const Project$Project$modrinth_id = (value) => value.modrinth_id;
export const Project$Project$12 = (value) => value.modrinth_id;
export const Project$Project$curseforge_id = (value) => value.curseforge_id;
export const Project$Project$13 = (value) => value.curseforge_id;
export const Project$Project$github_id = (value) => value.github_id;
export const Project$Project$14 = (value) => value.github_id;
export const Project$Project$gitea_id = (value) => value.gitea_id;
export const Project$Project$15 = (value) => value.gitea_id;
export const Project$Project$gitlab_id = (value) => value.gitlab_id;
export const Project$Project$16 = (value) => value.gitlab_id;
export const Project$Project$docs_path = (value) => value.docs_path;
export const Project$Project$17 = (value) => value.docs_path;
export const Project$Project$variants = (value) => value.variants;
export const Project$Project$18 = (value) => value.variants;
export const Project$Project$subdirs = (value) => value.subdirs;
export const Project$Project$19 = (value) => value.subdirs;

export class Variant extends $CustomType {
  constructor(id, minecraft, loader, version) {
    super();
    this.id = id;
    this.minecraft = minecraft;
    this.loader = loader;
    this.version = version;
  }
}
export const Variant$Variant = (id, minecraft, loader, version) =>
  new Variant(id, minecraft, loader, version);
export const Variant$isVariant = (value) => value instanceof Variant;
export const Variant$Variant$id = (value) => value.id;
export const Variant$Variant$0 = (value) => value.id;
export const Variant$Variant$minecraft = (value) => value.minecraft;
export const Variant$Variant$1 = (value) => value.minecraft;
export const Variant$Variant$loader = (value) => value.loader;
export const Variant$Variant$2 = (value) => value.loader;
export const Variant$Variant$version = (value) => value.version;
export const Variant$Variant$3 = (value) => value.version;

export class Subdir extends $CustomType {
  constructor(key, path, platform, mod_count, has_index, has_pack) {
    super();
    this.key = key;
    this.path = path;
    this.platform = platform;
    this.mod_count = mod_count;
    this.has_index = has_index;
    this.has_pack = has_pack;
  }
}
export const Subdir$Subdir = (key, path, platform, mod_count, has_index, has_pack) =>
  new Subdir(key, path, platform, mod_count, has_index, has_pack);
export const Subdir$isSubdir = (value) => value instanceof Subdir;
export const Subdir$Subdir$key = (value) => value.key;
export const Subdir$Subdir$0 = (value) => value.key;
export const Subdir$Subdir$path = (value) => value.path;
export const Subdir$Subdir$1 = (value) => value.path;
export const Subdir$Subdir$platform = (value) => value.platform;
export const Subdir$Subdir$2 = (value) => value.platform;
export const Subdir$Subdir$mod_count = (value) => value.mod_count;
export const Subdir$Subdir$3 = (value) => value.mod_count;
export const Subdir$Subdir$has_index = (value) => value.has_index;
export const Subdir$Subdir$4 = (value) => value.has_index;
export const Subdir$Subdir$has_pack = (value) => value.has_pack;
export const Subdir$Subdir$5 = (value) => value.has_pack;

export class ModEntry extends $CustomType {
  constructor(slug, name, filename, side, pin, platform, version_id) {
    super();
    this.slug = slug;
    this.name = name;
    this.filename = filename;
    this.side = side;
    this.pin = pin;
    this.platform = platform;
    this.version_id = version_id;
  }
}
export const ModEntry$ModEntry = (slug, name, filename, side, pin, platform, version_id) =>
  new ModEntry(slug, name, filename, side, pin, platform, version_id);
export const ModEntry$isModEntry = (value) => value instanceof ModEntry;
export const ModEntry$ModEntry$slug = (value) => value.slug;
export const ModEntry$ModEntry$0 = (value) => value.slug;
export const ModEntry$ModEntry$name = (value) => value.name;
export const ModEntry$ModEntry$1 = (value) => value.name;
export const ModEntry$ModEntry$filename = (value) => value.filename;
export const ModEntry$ModEntry$2 = (value) => value.filename;
export const ModEntry$ModEntry$side = (value) => value.side;
export const ModEntry$ModEntry$3 = (value) => value.side;
export const ModEntry$ModEntry$pin = (value) => value.pin;
export const ModEntry$ModEntry$4 = (value) => value.pin;
export const ModEntry$ModEntry$platform = (value) => value.platform;
export const ModEntry$ModEntry$5 = (value) => value.platform;
export const ModEntry$ModEntry$version_id = (value) => value.version_id;
export const ModEntry$ModEntry$6 = (value) => value.version_id;

export class ContentResponse extends $CustomType {
  constructor(path, content) {
    super();
    this.path = path;
    this.content = content;
  }
}
export const ContentResponse$ContentResponse = (path, content) =>
  new ContentResponse(path, content);
export const ContentResponse$isContentResponse = (value) =>
  value instanceof ContentResponse;
export const ContentResponse$ContentResponse$path = (value) => value.path;
export const ContentResponse$ContentResponse$0 = (value) => value.path;
export const ContentResponse$ContentResponse$content = (value) => value.content;
export const ContentResponse$ContentResponse$1 = (value) => value.content;

export class ActionResponse extends $CustomType {
  constructor(job_id) {
    super();
    this.job_id = job_id;
  }
}
export const ActionResponse$ActionResponse = (job_id) =>
  new ActionResponse(job_id);
export const ActionResponse$isActionResponse = (value) =>
  value instanceof ActionResponse;
export const ActionResponse$ActionResponse$job_id = (value) => value.job_id;
export const ActionResponse$ActionResponse$0 = (value) => value.job_id;

export class CreatedProject extends $CustomType {
  constructor(id, dir) {
    super();
    this.id = id;
    this.dir = dir;
  }
}
export const CreatedProject$CreatedProject = (id, dir) =>
  new CreatedProject(id, dir);
export const CreatedProject$isCreatedProject = (value) =>
  value instanceof CreatedProject;
export const CreatedProject$CreatedProject$id = (value) => value.id;
export const CreatedProject$CreatedProject$0 = (value) => value.id;
export const CreatedProject$CreatedProject$dir = (value) => value.dir;
export const CreatedProject$CreatedProject$1 = (value) => value.dir;

export class FeatureIndex extends $CustomType {
  constructor(packwand_version, features) {
    super();
    this.packwand_version = packwand_version;
    this.features = features;
  }
}
export const FeatureIndex$FeatureIndex = (packwand_version, features) =>
  new FeatureIndex(packwand_version, features);
export const FeatureIndex$isFeatureIndex = (value) =>
  value instanceof FeatureIndex;
export const FeatureIndex$FeatureIndex$packwand_version = (value) =>
  value.packwand_version;
export const FeatureIndex$FeatureIndex$0 = (value) => value.packwand_version;
export const FeatureIndex$FeatureIndex$features = (value) => value.features;
export const FeatureIndex$FeatureIndex$1 = (value) => value.features;

export class Feature extends $CustomType {
  constructor(command, usage, summary, group, runnable, gui_status, gui_action, scope, destructive) {
    super();
    this.command = command;
    this.usage = usage;
    this.summary = summary;
    this.group = group;
    this.runnable = runnable;
    this.gui_status = gui_status;
    this.gui_action = gui_action;
    this.scope = scope;
    this.destructive = destructive;
  }
}
export const Feature$Feature = (command, usage, summary, group, runnable, gui_status, gui_action, scope, destructive) =>
  new Feature(command,
  usage,
  summary,
  group,
  runnable,
  gui_status,
  gui_action,
  scope,
  destructive);
export const Feature$isFeature = (value) => value instanceof Feature;
export const Feature$Feature$command = (value) => value.command;
export const Feature$Feature$0 = (value) => value.command;
export const Feature$Feature$usage = (value) => value.usage;
export const Feature$Feature$1 = (value) => value.usage;
export const Feature$Feature$summary = (value) => value.summary;
export const Feature$Feature$2 = (value) => value.summary;
export const Feature$Feature$group = (value) => value.group;
export const Feature$Feature$3 = (value) => value.group;
export const Feature$Feature$runnable = (value) => value.runnable;
export const Feature$Feature$4 = (value) => value.runnable;
export const Feature$Feature$gui_status = (value) => value.gui_status;
export const Feature$Feature$5 = (value) => value.gui_status;
export const Feature$Feature$gui_action = (value) => value.gui_action;
export const Feature$Feature$6 = (value) => value.gui_action;
export const Feature$Feature$scope = (value) => value.scope;
export const Feature$Feature$7 = (value) => value.scope;
export const Feature$Feature$destructive = (value) => value.destructive;
export const Feature$Feature$8 = (value) => value.destructive;

export class PacksIndex extends $CustomType {}
export const Action$PacksIndex = () => new PacksIndex();
export const Action$isPacksIndex = (value) => value instanceof PacksIndex;

export class ValidateAll extends $CustomType {}
export const Action$ValidateAll = () => new ValidateAll();
export const Action$isValidateAll = (value) => value instanceof ValidateAll;

export class ValidateProject extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$ValidateProject = (path) => new ValidateProject(path);
export const Action$isValidateProject = (value) =>
  value instanceof ValidateProject;
export const Action$ValidateProject$path = (value) => value.path;
export const Action$ValidateProject$0 = (value) => value.path;

export class Doctor extends $CustomType {}
export const Action$Doctor = () => new Doctor();
export const Action$isDoctor = (value) => value instanceof Doctor;

export class Lint extends $CustomType {}
export const Action$Lint = () => new Lint();
export const Action$isLint = (value) => value instanceof Lint;

export class WorkspaceStatus extends $CustomType {}
export const Action$WorkspaceStatus = () => new WorkspaceStatus();
export const Action$isWorkspaceStatus = (value) =>
  value instanceof WorkspaceStatus;

export class WorkspaceSync extends $CustomType {
  constructor(dry_run) {
    super();
    this.dry_run = dry_run;
  }
}
export const Action$WorkspaceSync = (dry_run) => new WorkspaceSync(dry_run);
export const Action$isWorkspaceSync = (value) => value instanceof WorkspaceSync;
export const Action$WorkspaceSync$dry_run = (value) => value.dry_run;
export const Action$WorkspaceSync$0 = (value) => value.dry_run;

export class WorkspaceRefresh extends $CustomType {}
export const Action$WorkspaceRefresh = () => new WorkspaceRefresh();
export const Action$isWorkspaceRefresh = (value) =>
  value instanceof WorkspaceRefresh;

export class WorkspaceUpdate extends $CustomType {
  constructor(check) {
    super();
    this.check = check;
  }
}
export const Action$WorkspaceUpdate = (check) => new WorkspaceUpdate(check);
export const Action$isWorkspaceUpdate = (value) =>
  value instanceof WorkspaceUpdate;
export const Action$WorkspaceUpdate$check = (value) => value.check;
export const Action$WorkspaceUpdate$0 = (value) => value.check;

export class RefreshSubdir extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$RefreshSubdir = (path) => new RefreshSubdir(path);
export const Action$isRefreshSubdir = (value) => value instanceof RefreshSubdir;
export const Action$RefreshSubdir$path = (value) => value.path;
export const Action$RefreshSubdir$0 = (value) => value.path;

export class AddMod extends $CustomType {
  constructor(path, slug) {
    super();
    this.path = path;
    this.slug = slug;
  }
}
export const Action$AddMod = (path, slug) => new AddMod(path, slug);
export const Action$isAddMod = (value) => value instanceof AddMod;
export const Action$AddMod$path = (value) => value.path;
export const Action$AddMod$0 = (value) => value.path;
export const Action$AddMod$slug = (value) => value.slug;
export const Action$AddMod$1 = (value) => value.slug;

export class RemoveMod extends $CustomType {
  constructor(path, slug) {
    super();
    this.path = path;
    this.slug = slug;
  }
}
export const Action$RemoveMod = (path, slug) => new RemoveMod(path, slug);
export const Action$isRemoveMod = (value) => value instanceof RemoveMod;
export const Action$RemoveMod$path = (value) => value.path;
export const Action$RemoveMod$0 = (value) => value.path;
export const Action$RemoveMod$slug = (value) => value.slug;
export const Action$RemoveMod$1 = (value) => value.slug;

export class PinMod extends $CustomType {
  constructor(path, slug) {
    super();
    this.path = path;
    this.slug = slug;
  }
}
export const Action$PinMod = (path, slug) => new PinMod(path, slug);
export const Action$isPinMod = (value) => value instanceof PinMod;
export const Action$PinMod$path = (value) => value.path;
export const Action$PinMod$0 = (value) => value.path;
export const Action$PinMod$slug = (value) => value.slug;
export const Action$PinMod$1 = (value) => value.slug;

export class UnpinMod extends $CustomType {
  constructor(path, slug) {
    super();
    this.path = path;
    this.slug = slug;
  }
}
export const Action$UnpinMod = (path, slug) => new UnpinMod(path, slug);
export const Action$isUnpinMod = (value) => value instanceof UnpinMod;
export const Action$UnpinMod$path = (value) => value.path;
export const Action$UnpinMod$0 = (value) => value.path;
export const Action$UnpinMod$slug = (value) => value.slug;
export const Action$UnpinMod$1 = (value) => value.slug;

export class UpdateMod extends $CustomType {
  constructor(path, slug) {
    super();
    this.path = path;
    this.slug = slug;
  }
}
export const Action$UpdateMod = (path, slug) => new UpdateMod(path, slug);
export const Action$isUpdateMod = (value) => value instanceof UpdateMod;
export const Action$UpdateMod$path = (value) => value.path;
export const Action$UpdateMod$0 = (value) => value.path;
export const Action$UpdateMod$slug = (value) => value.slug;
export const Action$UpdateMod$1 = (value) => value.slug;

export class UpdateAll extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$UpdateAll = (path) => new UpdateAll(path);
export const Action$isUpdateAll = (value) => value instanceof UpdateAll;
export const Action$UpdateAll$path = (value) => value.path;
export const Action$UpdateAll$0 = (value) => value.path;

export class Build extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$Build = (path) => new Build(path);
export const Action$isBuild = (value) => value instanceof Build;
export const Action$Build$path = (value) => value.path;
export const Action$Build$0 = (value) => value.path;

export class Rehash extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$Rehash = (path) => new Rehash(path);
export const Action$isRehash = (value) => value instanceof Rehash;
export const Action$Rehash$path = (value) => value.path;
export const Action$Rehash$0 = (value) => value.path;

export class ExportModrinth extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$ExportModrinth = (path) => new ExportModrinth(path);
export const Action$isExportModrinth = (value) =>
  value instanceof ExportModrinth;
export const Action$ExportModrinth$path = (value) => value.path;
export const Action$ExportModrinth$0 = (value) => value.path;

export class ExportCurseforge extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$ExportCurseforge = (path) => new ExportCurseforge(path);
export const Action$isExportCurseforge = (value) =>
  value instanceof ExportCurseforge;
export const Action$ExportCurseforge$path = (value) => value.path;
export const Action$ExportCurseforge$0 = (value) => value.path;

export class Bump extends $CustomType {
  constructor(path, version, configs) {
    super();
    this.path = path;
    this.version = version;
    this.configs = configs;
  }
}
export const Action$Bump = (path, version, configs) =>
  new Bump(path, version, configs);
export const Action$isBump = (value) => value instanceof Bump;
export const Action$Bump$path = (value) => value.path;
export const Action$Bump$0 = (value) => value.path;
export const Action$Bump$version = (value) => value.version;
export const Action$Bump$1 = (value) => value.version;
export const Action$Bump$configs = (value) => value.configs;
export const Action$Bump$2 = (value) => value.configs;

export class FreezeMod extends $CustomType {
  constructor(path, slug) {
    super();
    this.path = path;
    this.slug = slug;
  }
}
export const Action$FreezeMod = (path, slug) => new FreezeMod(path, slug);
export const Action$isFreezeMod = (value) => value instanceof FreezeMod;
export const Action$FreezeMod$path = (value) => value.path;
export const Action$FreezeMod$0 = (value) => value.path;
export const Action$FreezeMod$slug = (value) => value.slug;
export const Action$FreezeMod$1 = (value) => value.slug;

export class UnfreezeMod extends $CustomType {
  constructor(path, slug) {
    super();
    this.path = path;
    this.slug = slug;
  }
}
export const Action$UnfreezeMod = (path, slug) => new UnfreezeMod(path, slug);
export const Action$isUnfreezeMod = (value) => value instanceof UnfreezeMod;
export const Action$UnfreezeMod$path = (value) => value.path;
export const Action$UnfreezeMod$0 = (value) => value.path;
export const Action$UnfreezeMod$slug = (value) => value.slug;
export const Action$UnfreezeMod$1 = (value) => value.slug;

export class SetSide extends $CustomType {
  constructor(path, slug, side) {
    super();
    this.path = path;
    this.slug = slug;
    this.side = side;
  }
}
export const Action$SetSide = (path, slug, side) =>
  new SetSide(path, slug, side);
export const Action$isSetSide = (value) => value instanceof SetSide;
export const Action$SetSide$path = (value) => value.path;
export const Action$SetSide$0 = (value) => value.path;
export const Action$SetSide$slug = (value) => value.slug;
export const Action$SetSide$1 = (value) => value.slug;
export const Action$SetSide$side = (value) => value.side;
export const Action$SetSide$2 = (value) => value.side;

export class NixGen extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$NixGen = (path) => new NixGen(path);
export const Action$isNixGen = (value) => value instanceof NixGen;
export const Action$NixGen$path = (value) => value.path;
export const Action$NixGen$0 = (value) => value.path;

export class DocsModlist extends $CustomType {
  constructor(path) {
    super();
    this.path = path;
  }
}
export const Action$DocsModlist = (path) => new DocsModlist(path);
export const Action$isDocsModlist = (value) => value instanceof DocsModlist;
export const Action$DocsModlist$path = (value) => value.path;
export const Action$DocsModlist$0 = (value) => value.path;

export class DocsPages extends $CustomType {}
export const Action$DocsPages = () => new DocsPages();
export const Action$isDocsPages = (value) => value instanceof DocsPages;

export class LauncherEvent extends $CustomType {
  constructor(session_id, kind, pid, line, code, error) {
    super();
    this.session_id = session_id;
    this.kind = kind;
    this.pid = pid;
    this.line = line;
    this.code = code;
    this.error = error;
  }
}
export const LauncherEvent$LauncherEvent = (session_id, kind, pid, line, code, error) =>
  new LauncherEvent(session_id, kind, pid, line, code, error);
export const LauncherEvent$isLauncherEvent = (value) =>
  value instanceof LauncherEvent;
export const LauncherEvent$LauncherEvent$session_id = (value) =>
  value.session_id;
export const LauncherEvent$LauncherEvent$0 = (value) => value.session_id;
export const LauncherEvent$LauncherEvent$kind = (value) => value.kind;
export const LauncherEvent$LauncherEvent$1 = (value) => value.kind;
export const LauncherEvent$LauncherEvent$pid = (value) => value.pid;
export const LauncherEvent$LauncherEvent$2 = (value) => value.pid;
export const LauncherEvent$LauncherEvent$line = (value) => value.line;
export const LauncherEvent$LauncherEvent$3 = (value) => value.line;
export const LauncherEvent$LauncherEvent$code = (value) => value.code;
export const LauncherEvent$LauncherEvent$4 = (value) => value.code;
export const LauncherEvent$LauncherEvent$error = (value) => value.error;
export const LauncherEvent$LauncherEvent$5 = (value) => value.error;

export class LauncherProgress extends $CustomType {
  constructor(session_id, finished_downloads, total_downloads, downloaded_bytes, total_bytes) {
    super();
    this.session_id = session_id;
    this.finished_downloads = finished_downloads;
    this.total_downloads = total_downloads;
    this.downloaded_bytes = downloaded_bytes;
    this.total_bytes = total_bytes;
  }
}
export const LauncherProgress$LauncherProgress = (session_id, finished_downloads, total_downloads, downloaded_bytes, total_bytes) =>
  new LauncherProgress(session_id,
  finished_downloads,
  total_downloads,
  downloaded_bytes,
  total_bytes);
export const LauncherProgress$isLauncherProgress = (value) =>
  value instanceof LauncherProgress;
export const LauncherProgress$LauncherProgress$session_id = (value) =>
  value.session_id;
export const LauncherProgress$LauncherProgress$0 = (value) => value.session_id;
export const LauncherProgress$LauncherProgress$finished_downloads = (value) =>
  value.finished_downloads;
export const LauncherProgress$LauncherProgress$1 = (value) =>
  value.finished_downloads;
export const LauncherProgress$LauncherProgress$total_downloads = (value) =>
  value.total_downloads;
export const LauncherProgress$LauncherProgress$2 = (value) =>
  value.total_downloads;
export const LauncherProgress$LauncherProgress$downloaded_bytes = (value) =>
  value.downloaded_bytes;
export const LauncherProgress$LauncherProgress$3 = (value) =>
  value.downloaded_bytes;
export const LauncherProgress$LauncherProgress$total_bytes = (value) =>
  value.total_bytes;
export const LauncherProgress$LauncherProgress$4 = (value) => value.total_bytes;

export class AuthStatus extends $CustomType {
  constructor(signed_in, username) {
    super();
    this.signed_in = signed_in;
    this.username = username;
  }
}
export const AuthStatus$AuthStatus = (signed_in, username) =>
  new AuthStatus(signed_in, username);
export const AuthStatus$isAuthStatus = (value) => value instanceof AuthStatus;
export const AuthStatus$AuthStatus$signed_in = (value) => value.signed_in;
export const AuthStatus$AuthStatus$0 = (value) => value.signed_in;
export const AuthStatus$AuthStatus$username = (value) => value.username;
export const AuthStatus$AuthStatus$1 = (value) => value.username;

export class AuthEvent extends $CustomType {
  constructor(status, username, error) {
    super();
    this.status = status;
    this.username = username;
    this.error = error;
  }
}
export const AuthEvent$AuthEvent = (status, username, error) =>
  new AuthEvent(status, username, error);
export const AuthEvent$isAuthEvent = (value) => value instanceof AuthEvent;
export const AuthEvent$AuthEvent$status = (value) => value.status;
export const AuthEvent$AuthEvent$0 = (value) => value.status;
export const AuthEvent$AuthEvent$username = (value) => value.username;
export const AuthEvent$AuthEvent$1 = (value) => value.username;
export const AuthEvent$AuthEvent$error = (value) => value.error;
export const AuthEvent$AuthEvent$2 = (value) => value.error;

export function action_name(action) {
  if (action instanceof PacksIndex) {
    return "packs-index";
  } else if (action instanceof ValidateAll) {
    return "validate-all";
  } else if (action instanceof ValidateProject) {
    return "validate-project";
  } else if (action instanceof Doctor) {
    return "doctor";
  } else if (action instanceof Lint) {
    return "lint";
  } else if (action instanceof WorkspaceStatus) {
    return "workspace-status";
  } else if (action instanceof WorkspaceSync) {
    return "workspace-sync";
  } else if (action instanceof WorkspaceRefresh) {
    return "workspace-refresh";
  } else if (action instanceof WorkspaceUpdate) {
    let $ = action.check;
    if ($) {
      return "workspace-update-check";
    } else {
      return "workspace-update";
    }
  } else if (action instanceof RefreshSubdir) {
    return "refresh";
  } else if (action instanceof AddMod) {
    return "add-mod";
  } else if (action instanceof RemoveMod) {
    return "remove-mod";
  } else if (action instanceof PinMod) {
    return "pin-mod";
  } else if (action instanceof UnpinMod) {
    return "unpin-mod";
  } else if (action instanceof UpdateMod) {
    return "update-mod";
  } else if (action instanceof UpdateAll) {
    return "update-all";
  } else if (action instanceof Build) {
    return "build";
  } else if (action instanceof Rehash) {
    return "rehash";
  } else if (action instanceof ExportModrinth) {
    return "export-modrinth";
  } else if (action instanceof ExportCurseforge) {
    return "export-curseforge";
  } else if (action instanceof Bump) {
    return "bump";
  } else if (action instanceof FreezeMod) {
    return "freeze-mod";
  } else if (action instanceof UnfreezeMod) {
    return "unfreeze-mod";
  } else if (action instanceof SetSide) {
    return "set-side";
  } else if (action instanceof NixGen) {
    return "nix-gen";
  } else if (action instanceof DocsModlist) {
    return "docs-modlist";
  } else {
    return "docs-pages";
  }
}

export function action_subdir(action) {
  if (action instanceof ValidateProject) {
    let path = action.path;
    return path;
  } else if (action instanceof RefreshSubdir) {
    let path = action.path;
    return path;
  } else if (action instanceof AddMod) {
    let path = action.path;
    return path;
  } else if (action instanceof RemoveMod) {
    let path = action.path;
    return path;
  } else if (action instanceof PinMod) {
    let path = action.path;
    return path;
  } else if (action instanceof UnpinMod) {
    let path = action.path;
    return path;
  } else if (action instanceof UpdateMod) {
    let path = action.path;
    return path;
  } else if (action instanceof UpdateAll) {
    let path = action.path;
    return path;
  } else if (action instanceof Build) {
    let path = action.path;
    return path;
  } else if (action instanceof Rehash) {
    let path = action.path;
    return path;
  } else if (action instanceof ExportModrinth) {
    let path = action.path;
    return path;
  } else if (action instanceof ExportCurseforge) {
    let path = action.path;
    return path;
  } else if (action instanceof Bump) {
    let path = action.path;
    return path;
  } else if (action instanceof FreezeMod) {
    let path = action.path;
    return path;
  } else if (action instanceof UnfreezeMod) {
    let path = action.path;
    return path;
  } else if (action instanceof SetSide) {
    let path = action.path;
    return path;
  } else if (action instanceof NixGen) {
    let path = action.path;
    return path;
  } else if (action instanceof DocsModlist) {
    let path = action.path;
    return path;
  } else {
    return "";
  }
}

export function action_slug(action) {
  if (action instanceof AddMod) {
    let slug = action.slug;
    return slug;
  } else if (action instanceof RemoveMod) {
    let slug = action.slug;
    return slug;
  } else if (action instanceof PinMod) {
    let slug = action.slug;
    return slug;
  } else if (action instanceof UnpinMod) {
    let slug = action.slug;
    return slug;
  } else if (action instanceof UpdateMod) {
    let slug = action.slug;
    return slug;
  } else if (action instanceof FreezeMod) {
    let slug = action.slug;
    return slug;
  } else if (action instanceof UnfreezeMod) {
    let slug = action.slug;
    return slug;
  } else if (action instanceof SetSide) {
    let slug = action.slug;
    return slug;
  } else {
    return "";
  }
}

export function action_dry_run(action) {
  if (action instanceof WorkspaceSync) {
    let dry_run = action.dry_run;
    return dry_run;
  } else {
    return false;
  }
}

export function action_version(action) {
  if (action instanceof Bump) {
    let version = action.version;
    return version;
  } else {
    return "";
  }
}

export function action_configs(action) {
  if (action instanceof Bump) {
    let configs = action.configs;
    return configs;
  } else {
    return false;
  }
}

export function action_side(action) {
  if (action instanceof SetSide) {
    let side = action.side;
    return side;
  } else {
    return "";
  }
}

export function action_refreshes_mods(action) {
  if (action instanceof RefreshSubdir) {
    return true;
  } else if (action instanceof AddMod) {
    return true;
  } else if (action instanceof RemoveMod) {
    return true;
  } else if (action instanceof PinMod) {
    return true;
  } else if (action instanceof UnpinMod) {
    return true;
  } else if (action instanceof UpdateMod) {
    return true;
  } else if (action instanceof UpdateAll) {
    return true;
  } else if (action instanceof Build) {
    return true;
  } else if (action instanceof Rehash) {
    return true;
  } else if (action instanceof FreezeMod) {
    return true;
  } else if (action instanceof UnfreezeMod) {
    return true;
  } else if (action instanceof SetSide) {
    return true;
  } else {
    return false;
  }
}

export function launcher_event_decoder() {
  return $decode.optional_field(
    "session_id",
    "",
    $decode.string,
    (session_id) => {
      return $decode.optional_field(
        "event",
        "",
        $decode.string,
        (kind) => {
          return $decode.optional_field(
            "pid",
            0,
            $decode.int,
            (pid) => {
              return $decode.optional_field(
                "line",
                "",
                $decode.string,
                (line) => {
                  return $decode.optional_field(
                    "code",
                    0,
                    (() => {
                      let _pipe = $decode.optional($decode.int);
                      return $decode.map(
                        _pipe,
                        (_capture) => { return $option.unwrap(_capture, 0); },
                      );
                    })(),
                    (code) => {
                      return $decode.optional_field(
                        "error",
                        "",
                        $decode.string,
                        (error) => {
                          return $decode.success(
                            new LauncherEvent(
                              session_id,
                              kind,
                              pid,
                              line,
                              code,
                              error,
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

export function launcher_progress_decoder() {
  return $decode.optional_field(
    "session_id",
    "",
    $decode.string,
    (session_id) => {
      return $decode.optional_field(
        "finished_downloads",
        0,
        $decode.int,
        (finished_downloads) => {
          return $decode.optional_field(
            "total_downloads",
            0,
            $decode.int,
            (total_downloads) => {
              return $decode.optional_field(
                "downloaded_bytes",
                0,
                $decode.int,
                (downloaded_bytes) => {
                  return $decode.optional_field(
                    "total_bytes",
                    0,
                    $decode.int,
                    (total_bytes) => {
                      return $decode.success(
                        new LauncherProgress(
                          session_id,
                          finished_downloads,
                          total_downloads,
                          downloaded_bytes,
                          total_bytes,
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
}

export function auth_status_decoder() {
  return $decode.optional_field(
    "signed_in",
    false,
    $decode.bool,
    (signed_in) => {
      return $decode.optional_field(
        "username",
        "",
        $decode.string,
        (username) => {
          return $decode.success(new AuthStatus(signed_in, username));
        },
      );
    },
  );
}

export function auth_event_decoder() {
  return $decode.optional_field(
    "status",
    "",
    $decode.string,
    (status) => {
      return $decode.optional_field(
        "username",
        "",
        $decode.string,
        (username) => {
          return $decode.optional_field(
            "error",
            "",
            $decode.string,
            (error) => {
              return $decode.success(new AuthEvent(status, username, error));
            },
          );
        },
      );
    },
  );
}

function prefix(prefix, value) {
  if (value === "") {
    return value;
  } else {
    return prefix + value;
  }
}

export function project_summary(project) {
  let _pipe = toList([
    project.name,
    project.kind,
    prefix("v", project.version),
    prefix("mc", project.minecraft),
    project.loader,
  ]);
  let _pipe$1 = $list.filter(_pipe, (value) => { return value !== ""; });
  return $string.join(_pipe$1, "  ");
}
