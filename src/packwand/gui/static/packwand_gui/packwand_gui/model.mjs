import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import {
  toList,
  Empty as $Empty,
  prepend as listPrepend,
  CustomType as $CustomType,
} from "../gleam.mjs";

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
  constructor(slug, name, filename, side, pin, platform) {
    super();
    this.slug = slug;
    this.name = name;
    this.filename = filename;
    this.side = side;
    this.pin = pin;
    this.platform = platform;
  }
}
export const ModEntry$ModEntry = (slug, name, filename, side, pin, platform) =>
  new ModEntry(slug, name, filename, side, pin, platform);
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

export class PacksIndex extends $CustomType {}
export const Action$PacksIndex = () => new PacksIndex();
export const Action$isPacksIndex = (value) => value instanceof PacksIndex;

export class ValidateAll extends $CustomType {}
export const Action$ValidateAll = () => new ValidateAll();
export const Action$isValidateAll = (value) => value instanceof ValidateAll;

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

export function action_name(action) {
  if (action instanceof PacksIndex) {
    return "packs-index";
  } else if (action instanceof ValidateAll) {
    return "validate-all";
  } else if (action instanceof WorkspaceStatus) {
    return "workspace-status";
  } else if (action instanceof WorkspaceSync) {
    return "workspace-sync";
  } else if (action instanceof WorkspaceRefresh) {
    return "workspace-refresh";
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
  } else if (action instanceof ExportModrinth) {
    return "export-modrinth";
  } else {
    return "export-curseforge";
  }
}

export function action_subdir(action) {
  if (action instanceof RefreshSubdir) {
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
  } else if (action instanceof ExportModrinth) {
    let path = action.path;
    return path;
  } else if (action instanceof ExportCurseforge) {
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
  } else {
    return false;
  }
}

function list_filter_non_empty(values) {
  if (values instanceof $Empty) {
    return values;
  } else {
    let first = values.head;
    let rest = values.tail;
    let filtered = list_filter_non_empty(rest);
    if (first === "") {
      return filtered;
    } else {
      return listPrepend(first, filtered);
    }
  }
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
  let _pipe$1 = list_filter_non_empty(_pipe);
  return $string.join(_pipe$1, "  ");
}
