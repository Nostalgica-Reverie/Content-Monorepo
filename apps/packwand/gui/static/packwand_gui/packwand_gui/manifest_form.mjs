import * as $json from "../../gleam_json/gleam/json.mjs";
import * as $dict from "../../gleam_stdlib/gleam/dict.mjs";
import * as $decode from "../../gleam_stdlib/gleam/dynamic/decode.mjs";
import * as $int from "../../gleam_stdlib/gleam/int.mjs";
import * as $list from "../../gleam_stdlib/gleam/list.mjs";
import * as $option from "../../gleam_stdlib/gleam/option.mjs";
import { None, Some } from "../../gleam_stdlib/gleam/option.mjs";
import * as $string from "../../gleam_stdlib/gleam/string.mjs";
import { Ok, Error, toList, Empty as $Empty, CustomType as $CustomType, isEqual } from "../gleam.mjs";
import { prettyJson as pretty_json } from "../packwand_gui/ffi.mjs";

export class RoleNone extends $CustomType {}
export const RoleKind$RoleNone = () => new RoleNone();
export const RoleKind$isRoleNone = (value) => value instanceof RoleNone;

export class RoleBase extends $CustomType {}
export const RoleKind$RoleBase = () => new RoleBase();
export const RoleKind$isRoleBase = (value) => value instanceof RoleBase;

export class RoleConsumer extends $CustomType {}
export const RoleKind$RoleConsumer = () => new RoleConsumer();
export const RoleKind$isRoleConsumer = (value) => value instanceof RoleConsumer;

export class Mapping extends $CustomType {
  constructor(source, target) {
    super();
    this.source = source;
    this.target = target;
  }
}
export const Mapping$Mapping = (source, target) => new Mapping(source, target);
export const Mapping$isMapping = (value) => value instanceof Mapping;
export const Mapping$Mapping$source = (value) => value.source;
export const Mapping$Mapping$0 = (value) => value.source;
export const Mapping$Mapping$target = (value) => value.target;
export const Mapping$Mapping$1 = (value) => value.target;

export class FormVariant extends $CustomType {
  constructor(mc_version, id, name, version, release_type, loader) {
    super();
    this.mc_version = mc_version;
    this.id = id;
    this.name = name;
    this.version = version;
    this.release_type = release_type;
    this.loader = loader;
  }
}
export const FormVariant$FormVariant = (mc_version, id, name, version, release_type, loader) =>
  new FormVariant(mc_version, id, name, version, release_type, loader);
export const FormVariant$isFormVariant = (value) =>
  value instanceof FormVariant;
export const FormVariant$FormVariant$mc_version = (value) => value.mc_version;
export const FormVariant$FormVariant$0 = (value) => value.mc_version;
export const FormVariant$FormVariant$id = (value) => value.id;
export const FormVariant$FormVariant$1 = (value) => value.id;
export const FormVariant$FormVariant$name = (value) => value.name;
export const FormVariant$FormVariant$2 = (value) => value.name;
export const FormVariant$FormVariant$version = (value) => value.version;
export const FormVariant$FormVariant$3 = (value) => value.version;
export const FormVariant$FormVariant$release_type = (value) =>
  value.release_type;
export const FormVariant$FormVariant$4 = (value) => value.release_type;
export const FormVariant$FormVariant$loader = (value) => value.loader;
export const FormVariant$FormVariant$5 = (value) => value.loader;

export class Automation extends $CustomType {
  constructor(auto_update, server_promo, sync_exclude, freeze) {
    super();
    this.auto_update = auto_update;
    this.server_promo = server_promo;
    this.sync_exclude = sync_exclude;
    this.freeze = freeze;
  }
}
export const Automation$Automation = (auto_update, server_promo, sync_exclude, freeze) =>
  new Automation(auto_update, server_promo, sync_exclude, freeze);
export const Automation$isAutomation = (value) => value instanceof Automation;
export const Automation$Automation$auto_update = (value) => value.auto_update;
export const Automation$Automation$0 = (value) => value.auto_update;
export const Automation$Automation$server_promo = (value) => value.server_promo;
export const Automation$Automation$1 = (value) => value.server_promo;
export const Automation$Automation$sync_exclude = (value) => value.sync_exclude;
export const Automation$Automation$2 = (value) => value.sync_exclude;
export const Automation$Automation$freeze = (value) => value.freeze;
export const Automation$Automation$3 = (value) => value.freeze;

export class ManifestForm extends $CustomType {
  constructor(schema, id, name, kind, loader, release_type, version, mc_version, use_variants, variants, modrinth_id, curseforge_id, github_id, gitea_id, gitlab_id, lifecycle, role_kind, role_pack, role_mappings, shared_assets, automation) {
    super();
    this.schema = schema;
    this.id = id;
    this.name = name;
    this.kind = kind;
    this.loader = loader;
    this.release_type = release_type;
    this.version = version;
    this.mc_version = mc_version;
    this.use_variants = use_variants;
    this.variants = variants;
    this.modrinth_id = modrinth_id;
    this.curseforge_id = curseforge_id;
    this.github_id = github_id;
    this.gitea_id = gitea_id;
    this.gitlab_id = gitlab_id;
    this.lifecycle = lifecycle;
    this.role_kind = role_kind;
    this.role_pack = role_pack;
    this.role_mappings = role_mappings;
    this.shared_assets = shared_assets;
    this.automation = automation;
  }
}
export const ManifestForm$ManifestForm = (schema, id, name, kind, loader, release_type, version, mc_version, use_variants, variants, modrinth_id, curseforge_id, github_id, gitea_id, gitlab_id, lifecycle, role_kind, role_pack, role_mappings, shared_assets, automation) =>
  new ManifestForm(schema,
  id,
  name,
  kind,
  loader,
  release_type,
  version,
  mc_version,
  use_variants,
  variants,
  modrinth_id,
  curseforge_id,
  github_id,
  gitea_id,
  gitlab_id,
  lifecycle,
  role_kind,
  role_pack,
  role_mappings,
  shared_assets,
  automation);
export const ManifestForm$isManifestForm = (value) =>
  value instanceof ManifestForm;
export const ManifestForm$ManifestForm$schema = (value) => value.schema;
export const ManifestForm$ManifestForm$0 = (value) => value.schema;
export const ManifestForm$ManifestForm$id = (value) => value.id;
export const ManifestForm$ManifestForm$1 = (value) => value.id;
export const ManifestForm$ManifestForm$name = (value) => value.name;
export const ManifestForm$ManifestForm$2 = (value) => value.name;
export const ManifestForm$ManifestForm$kind = (value) => value.kind;
export const ManifestForm$ManifestForm$3 = (value) => value.kind;
export const ManifestForm$ManifestForm$loader = (value) => value.loader;
export const ManifestForm$ManifestForm$4 = (value) => value.loader;
export const ManifestForm$ManifestForm$release_type = (value) =>
  value.release_type;
export const ManifestForm$ManifestForm$5 = (value) => value.release_type;
export const ManifestForm$ManifestForm$version = (value) => value.version;
export const ManifestForm$ManifestForm$6 = (value) => value.version;
export const ManifestForm$ManifestForm$mc_version = (value) => value.mc_version;
export const ManifestForm$ManifestForm$7 = (value) => value.mc_version;
export const ManifestForm$ManifestForm$use_variants = (value) =>
  value.use_variants;
export const ManifestForm$ManifestForm$8 = (value) => value.use_variants;
export const ManifestForm$ManifestForm$variants = (value) => value.variants;
export const ManifestForm$ManifestForm$9 = (value) => value.variants;
export const ManifestForm$ManifestForm$modrinth_id = (value) =>
  value.modrinth_id;
export const ManifestForm$ManifestForm$10 = (value) => value.modrinth_id;
export const ManifestForm$ManifestForm$curseforge_id = (value) =>
  value.curseforge_id;
export const ManifestForm$ManifestForm$11 = (value) => value.curseforge_id;
export const ManifestForm$ManifestForm$github_id = (value) => value.github_id;
export const ManifestForm$ManifestForm$12 = (value) => value.github_id;
export const ManifestForm$ManifestForm$gitea_id = (value) => value.gitea_id;
export const ManifestForm$ManifestForm$13 = (value) => value.gitea_id;
export const ManifestForm$ManifestForm$gitlab_id = (value) => value.gitlab_id;
export const ManifestForm$ManifestForm$14 = (value) => value.gitlab_id;
export const ManifestForm$ManifestForm$lifecycle = (value) => value.lifecycle;
export const ManifestForm$ManifestForm$15 = (value) => value.lifecycle;
export const ManifestForm$ManifestForm$role_kind = (value) => value.role_kind;
export const ManifestForm$ManifestForm$16 = (value) => value.role_kind;
export const ManifestForm$ManifestForm$role_pack = (value) => value.role_pack;
export const ManifestForm$ManifestForm$17 = (value) => value.role_pack;
export const ManifestForm$ManifestForm$role_mappings = (value) =>
  value.role_mappings;
export const ManifestForm$ManifestForm$18 = (value) => value.role_mappings;
export const ManifestForm$ManifestForm$shared_assets = (value) =>
  value.shared_assets;
export const ManifestForm$ManifestForm$19 = (value) => value.shared_assets;
export const ManifestForm$ManifestForm$automation = (value) => value.automation;
export const ManifestForm$ManifestForm$20 = (value) => value.automation;

export class VMcVersion extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const VariantField$VMcVersion = ($0) => new VMcVersion($0);
export const VariantField$isVMcVersion = (value) => value instanceof VMcVersion;
export const VariantField$VMcVersion$0 = (value) => value[0];

export class VId extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const VariantField$VId = ($0) => new VId($0);
export const VariantField$isVId = (value) => value instanceof VId;
export const VariantField$VId$0 = (value) => value[0];

export class VName extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const VariantField$VName = ($0) => new VName($0);
export const VariantField$isVName = (value) => value instanceof VName;
export const VariantField$VName$0 = (value) => value[0];

export class VVersion extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const VariantField$VVersion = ($0) => new VVersion($0);
export const VariantField$isVVersion = (value) => value instanceof VVersion;
export const VariantField$VVersion$0 = (value) => value[0];

export class VReleaseType extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const VariantField$VReleaseType = ($0) => new VReleaseType($0);
export const VariantField$isVReleaseType = (value) =>
  value instanceof VReleaseType;
export const VariantField$VReleaseType$0 = (value) => value[0];

export class VLoader extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const VariantField$VLoader = ($0) => new VLoader($0);
export const VariantField$isVLoader = (value) => value instanceof VLoader;
export const VariantField$VLoader$0 = (value) => value[0];

export class FId extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FId = ($0) => new FId($0);
export const Field$isFId = (value) => value instanceof FId;
export const Field$FId$0 = (value) => value[0];

export class FName extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FName = ($0) => new FName($0);
export const Field$isFName = (value) => value instanceof FName;
export const Field$FName$0 = (value) => value[0];

export class FKind extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FKind = ($0) => new FKind($0);
export const Field$isFKind = (value) => value instanceof FKind;
export const Field$FKind$0 = (value) => value[0];

export class FLoader extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FLoader = ($0) => new FLoader($0);
export const Field$isFLoader = (value) => value instanceof FLoader;
export const Field$FLoader$0 = (value) => value[0];

export class FReleaseType extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FReleaseType = ($0) => new FReleaseType($0);
export const Field$isFReleaseType = (value) => value instanceof FReleaseType;
export const Field$FReleaseType$0 = (value) => value[0];

export class FVersion extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FVersion = ($0) => new FVersion($0);
export const Field$isFVersion = (value) => value instanceof FVersion;
export const Field$FVersion$0 = (value) => value[0];

export class FMcVersion extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FMcVersion = ($0) => new FMcVersion($0);
export const Field$isFMcVersion = (value) => value instanceof FMcVersion;
export const Field$FMcVersion$0 = (value) => value[0];

export class FUseVariants extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FUseVariants = ($0) => new FUseVariants($0);
export const Field$isFUseVariants = (value) => value instanceof FUseVariants;
export const Field$FUseVariants$0 = (value) => value[0];

export class FVariantAdd extends $CustomType {}
export const Field$FVariantAdd = () => new FVariantAdd();
export const Field$isFVariantAdd = (value) => value instanceof FVariantAdd;

export class FVariantRemove extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FVariantRemove = ($0) => new FVariantRemove($0);
export const Field$isFVariantRemove = (value) =>
  value instanceof FVariantRemove;
export const Field$FVariantRemove$0 = (value) => value[0];

export class FVariant extends $CustomType {
  constructor($0, $1) {
    super();
    this[0] = $0;
    this[1] = $1;
  }
}
export const Field$FVariant = ($0, $1) => new FVariant($0, $1);
export const Field$isFVariant = (value) => value instanceof FVariant;
export const Field$FVariant$0 = (value) => value[0];
export const Field$FVariant$1 = (value) => value[1];

export class FModrinthId extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FModrinthId = ($0) => new FModrinthId($0);
export const Field$isFModrinthId = (value) => value instanceof FModrinthId;
export const Field$FModrinthId$0 = (value) => value[0];

export class FCurseforgeId extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FCurseforgeId = ($0) => new FCurseforgeId($0);
export const Field$isFCurseforgeId = (value) => value instanceof FCurseforgeId;
export const Field$FCurseforgeId$0 = (value) => value[0];

export class FGithubId extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FGithubId = ($0) => new FGithubId($0);
export const Field$isFGithubId = (value) => value instanceof FGithubId;
export const Field$FGithubId$0 = (value) => value[0];

export class FGiteaId extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FGiteaId = ($0) => new FGiteaId($0);
export const Field$isFGiteaId = (value) => value instanceof FGiteaId;
export const Field$FGiteaId$0 = (value) => value[0];

export class FGitlabId extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FGitlabId = ($0) => new FGitlabId($0);
export const Field$isFGitlabId = (value) => value instanceof FGitlabId;
export const Field$FGitlabId$0 = (value) => value[0];

export class FLifecycle extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FLifecycle = ($0) => new FLifecycle($0);
export const Field$isFLifecycle = (value) => value instanceof FLifecycle;
export const Field$FLifecycle$0 = (value) => value[0];

export class FRoleKind extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FRoleKind = ($0) => new FRoleKind($0);
export const Field$isFRoleKind = (value) => value instanceof FRoleKind;
export const Field$FRoleKind$0 = (value) => value[0];

export class FRolePack extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FRolePack = ($0) => new FRolePack($0);
export const Field$isFRolePack = (value) => value instanceof FRolePack;
export const Field$FRolePack$0 = (value) => value[0];

export class FMappingAdd extends $CustomType {}
export const Field$FMappingAdd = () => new FMappingAdd();
export const Field$isFMappingAdd = (value) => value instanceof FMappingAdd;

export class FMappingRemove extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FMappingRemove = ($0) => new FMappingRemove($0);
export const Field$isFMappingRemove = (value) =>
  value instanceof FMappingRemove;
export const Field$FMappingRemove$0 = (value) => value[0];

export class FMappingSource extends $CustomType {
  constructor($0, $1) {
    super();
    this[0] = $0;
    this[1] = $1;
  }
}
export const Field$FMappingSource = ($0, $1) => new FMappingSource($0, $1);
export const Field$isFMappingSource = (value) =>
  value instanceof FMappingSource;
export const Field$FMappingSource$0 = (value) => value[0];
export const Field$FMappingSource$1 = (value) => value[1];

export class FMappingTarget extends $CustomType {
  constructor($0, $1) {
    super();
    this[0] = $0;
    this[1] = $1;
  }
}
export const Field$FMappingTarget = ($0, $1) => new FMappingTarget($0, $1);
export const Field$isFMappingTarget = (value) =>
  value instanceof FMappingTarget;
export const Field$FMappingTarget$0 = (value) => value[0];
export const Field$FMappingTarget$1 = (value) => value[1];

export class FSharedAssets extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FSharedAssets = ($0) => new FSharedAssets($0);
export const Field$isFSharedAssets = (value) => value instanceof FSharedAssets;
export const Field$FSharedAssets$0 = (value) => value[0];

export class FAutoUpdate extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FAutoUpdate = ($0) => new FAutoUpdate($0);
export const Field$isFAutoUpdate = (value) => value instanceof FAutoUpdate;
export const Field$FAutoUpdate$0 = (value) => value[0];

export class FServerPromo extends $CustomType {
  constructor($0) {
    super();
    this[0] = $0;
  }
}
export const Field$FServerPromo = ($0) => new FServerPromo($0);
export const Field$isFServerPromo = (value) => value instanceof FServerPromo;
export const Field$FServerPromo$0 = (value) => value[0];

class RoleData extends $CustomType {
  constructor(kind, pack, mappings) {
    super();
    this.kind = kind;
    this.pack = pack;
    this.mappings = mappings;
  }
}

export class IssueError extends $CustomType {}
export const Severity$IssueError = () => new IssueError();
export const Severity$isIssueError = (value) => value instanceof IssueError;

export class IssueWarning extends $CustomType {}
export const Severity$IssueWarning = () => new IssueWarning();
export const Severity$isIssueWarning = (value) => value instanceof IssueWarning;

export class Issue extends $CustomType {
  constructor(field, severity, message) {
    super();
    this.field = field;
    this.severity = severity;
    this.message = message;
  }
}
export const Issue$Issue = (field, severity, message) =>
  new Issue(field, severity, message);
export const Issue$isIssue = (value) => value instanceof Issue;
export const Issue$Issue$field = (value) => value.field;
export const Issue$Issue$0 = (value) => value.field;
export const Issue$Issue$severity = (value) => value.severity;
export const Issue$Issue$1 = (value) => value.severity;
export const Issue$Issue$message = (value) => value.message;
export const Issue$Issue$2 = (value) => value.message;

function empty_automation() {
  return new Automation(new None(), new None(), toList([]), toList([]));
}

function set_automation_bool(automation, raw, set) {
  let _block;
  if (raw === "true") {
    _block = new Some(true);
  } else if (raw === "false") {
    _block = new Some(false);
  } else {
    _block = new None();
  }
  let value = _block;
  let settings = $option.unwrap(automation, empty_automation());
  let updated = set(settings, value);
  let $ = updated.auto_update;
  if ($ instanceof None) {
    let $1 = updated.server_promo;
    if ($1 instanceof None) {
      let $2 = updated.sync_exclude;
      if ($2 instanceof $Empty) {
        let $3 = updated.freeze;
        if ($3 instanceof $Empty) {
          return new None();
        } else {
          return new Some(updated);
        }
      } else {
        return new Some(updated);
      }
    } else {
      return new Some(updated);
    }
  } else {
    return new Some(updated);
  }
}

function update_at(items, index, updater) {
  return $list.index_map(
    items,
    (item, i) => {
      let $ = i === index;
      if ($) {
        return updater(item);
      } else {
        return item;
      }
    },
  );
}

function remove_at(items, index) {
  let _pipe = items;
  let _pipe$1 = $list.index_map(_pipe, (item, i) => { return [i, item]; });
  let _pipe$2 = $list.filter(_pipe$1, (pair) => { return pair[0] !== index; });
  return $list.map(_pipe$2, (pair) => { return pair[1]; });
}

function apply_variant(variant, field) {
  if (field instanceof VMcVersion) {
    let v = field[0];
    return new FormVariant(
      v,
      variant.id,
      variant.name,
      variant.version,
      variant.release_type,
      variant.loader,
    );
  } else if (field instanceof VId) {
    let v = field[0];
    return new FormVariant(
      variant.mc_version,
      v,
      variant.name,
      variant.version,
      variant.release_type,
      variant.loader,
    );
  } else if (field instanceof VName) {
    let v = field[0];
    return new FormVariant(
      variant.mc_version,
      variant.id,
      v,
      variant.version,
      variant.release_type,
      variant.loader,
    );
  } else if (field instanceof VVersion) {
    let v = field[0];
    return new FormVariant(
      variant.mc_version,
      variant.id,
      variant.name,
      v,
      variant.release_type,
      variant.loader,
    );
  } else if (field instanceof VReleaseType) {
    let v = field[0];
    return new FormVariant(
      variant.mc_version,
      variant.id,
      variant.name,
      variant.version,
      v,
      variant.loader,
    );
  } else {
    let v = field[0];
    return new FormVariant(
      variant.mc_version,
      variant.id,
      variant.name,
      variant.version,
      variant.release_type,
      v,
    );
  }
}

function empty_variant() {
  return new FormVariant("", "", "", "", "", "");
}

export function apply(form, field) {
  if (field instanceof FId) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      v,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FName) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      v,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FKind) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      v,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FLoader) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      v,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FReleaseType) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      v,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FVersion) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      v,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FMcVersion) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      v,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FUseVariants) {
    let v = field[0];
    let $ = form.variants;
    if (v && $ instanceof $Empty) {
      return new ManifestForm(
        form.schema,
        form.id,
        form.name,
        form.kind,
        form.loader,
        form.release_type,
        form.version,
        form.mc_version,
        true,
        toList([empty_variant()]),
        form.modrinth_id,
        form.curseforge_id,
        form.github_id,
        form.gitea_id,
        form.gitlab_id,
        form.lifecycle,
        form.role_kind,
        form.role_pack,
        form.role_mappings,
        form.shared_assets,
        form.automation,
      );
    } else {
      return new ManifestForm(
        form.schema,
        form.id,
        form.name,
        form.kind,
        form.loader,
        form.release_type,
        form.version,
        form.mc_version,
        v,
        form.variants,
        form.modrinth_id,
        form.curseforge_id,
        form.github_id,
        form.gitea_id,
        form.gitlab_id,
        form.lifecycle,
        form.role_kind,
        form.role_pack,
        form.role_mappings,
        form.shared_assets,
        form.automation,
      );
    }
  } else if (field instanceof FVariantAdd) {
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      $list.append(form.variants, toList([empty_variant()])),
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FVariantRemove) {
    let index = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      remove_at(form.variants, index),
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FVariant) {
    let index = field[0];
    let vf = field[1];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      update_at(
        form.variants,
        index,
        (_capture) => { return apply_variant(_capture, vf); },
      ),
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FModrinthId) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      v,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FCurseforgeId) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      v,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FGithubId) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      v,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FGiteaId) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      v,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FGitlabId) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      v,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FLifecycle) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      v,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FRoleKind) {
    let v = field[0];
    if (v === "base") {
      return new ManifestForm(
        form.schema,
        form.id,
        form.name,
        form.kind,
        form.loader,
        form.release_type,
        form.version,
        form.mc_version,
        form.use_variants,
        form.variants,
        form.modrinth_id,
        form.curseforge_id,
        form.github_id,
        form.gitea_id,
        form.gitlab_id,
        form.lifecycle,
        new RoleBase(),
        form.role_pack,
        form.role_mappings,
        form.shared_assets,
        form.automation,
      );
    } else if (v === "consumer") {
      let $ = form.role_mappings;
      if ($ instanceof $Empty) {
        return new ManifestForm(
          form.schema,
          form.id,
          form.name,
          form.kind,
          form.loader,
          form.release_type,
          form.version,
          form.mc_version,
          form.use_variants,
          form.variants,
          form.modrinth_id,
          form.curseforge_id,
          form.github_id,
          form.gitea_id,
          form.gitlab_id,
          form.lifecycle,
          new RoleConsumer(),
          form.role_pack,
          toList([new Mapping("", "")]),
          form.shared_assets,
          form.automation,
        );
      } else {
        return new ManifestForm(
          form.schema,
          form.id,
          form.name,
          form.kind,
          form.loader,
          form.release_type,
          form.version,
          form.mc_version,
          form.use_variants,
          form.variants,
          form.modrinth_id,
          form.curseforge_id,
          form.github_id,
          form.gitea_id,
          form.gitlab_id,
          form.lifecycle,
          new RoleConsumer(),
          form.role_pack,
          form.role_mappings,
          form.shared_assets,
          form.automation,
        );
      }
    } else {
      return new ManifestForm(
        form.schema,
        form.id,
        form.name,
        form.kind,
        form.loader,
        form.release_type,
        form.version,
        form.mc_version,
        form.use_variants,
        form.variants,
        form.modrinth_id,
        form.curseforge_id,
        form.github_id,
        form.gitea_id,
        form.gitlab_id,
        form.lifecycle,
        new RoleNone(),
        form.role_pack,
        form.role_mappings,
        form.shared_assets,
        form.automation,
      );
    }
  } else if (field instanceof FRolePack) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      v,
      form.role_mappings,
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FMappingAdd) {
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      $list.append(form.role_mappings, toList([new Mapping("", "")])),
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FMappingRemove) {
    let index = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      remove_at(form.role_mappings, index),
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FMappingSource) {
    let index = field[0];
    let v = field[1];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      update_at(
        form.role_mappings,
        index,
        (m) => { return new Mapping(v, m.target); },
      ),
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FMappingTarget) {
    let index = field[0];
    let v = field[1];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      update_at(
        form.role_mappings,
        index,
        (m) => { return new Mapping(m.source, v); },
      ),
      form.shared_assets,
      form.automation,
    );
  } else if (field instanceof FSharedAssets) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      v,
      form.automation,
    );
  } else if (field instanceof FAutoUpdate) {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      set_automation_bool(
        form.automation,
        v,
        (settings, value) => {
          return new Automation(
            value,
            settings.server_promo,
            settings.sync_exclude,
            settings.freeze,
          );
        },
      ),
    );
  } else {
    let v = field[0];
    return new ManifestForm(
      form.schema,
      form.id,
      form.name,
      form.kind,
      form.loader,
      form.release_type,
      form.version,
      form.mc_version,
      form.use_variants,
      form.variants,
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
      form.lifecycle,
      form.role_kind,
      form.role_pack,
      form.role_mappings,
      form.shared_assets,
      set_automation_bool(
        form.automation,
        v,
        (settings, value) => {
          return new Automation(
            settings.auto_update,
            value,
            settings.sync_exclude,
            settings.freeze,
          );
        },
      ),
    );
  }
}

function describe_parse_error(error) {
  if (error instanceof $json.UnexpectedEndOfInput) {
    return "Unexpected end of JSON input.";
  } else if (error instanceof $json.UnexpectedByte) {
    let byte = error[0];
    return ("Unexpected byte " + byte) + " in JSON.";
  } else if (error instanceof $json.UnexpectedSequence) {
    let seq = error[0];
    return ("Unexpected sequence " + seq) + " in JSON.";
  } else {
    return "The manifest JSON has an unexpected shape.";
  }
}

function freeze_decoder() {
  let _pipe = $decode.dict($decode.string, $decode.list($decode.string));
  return $decode.map(
    _pipe,
    (entries) => {
      let _pipe$1 = entries;
      let _pipe$2 = $dict.to_list(_pipe$1);
      return $list.sort(
        _pipe$2,
        (a, b) => { return $string.compare(a[0], b[0]); },
      );
    },
  );
}

function automation_decoder() {
  return $decode.optional_field(
    "auto_update",
    new None(),
    $decode.map($decode.bool, (var0) => { return new Some(var0); }),
    (auto_update) => {
      return $decode.optional_field(
        "server_promo",
        new None(),
        $decode.map($decode.bool, (var0) => { return new Some(var0); }),
        (server_promo) => {
          return $decode.optional_field(
            "sync_exclude",
            toList([]),
            $decode.list($decode.string),
            (sync_exclude) => {
              return $decode.optional_field(
                "freeze",
                toList([]),
                freeze_decoder(),
                (freeze) => {
                  return $decode.success(
                    new Automation(
                      auto_update,
                      server_promo,
                      sync_exclude,
                      freeze,
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
}

function mapping_decoder() {
  return $decode.optional_field(
    "source",
    "",
    $decode.string,
    (source) => {
      return $decode.optional_field(
        "target",
        "",
        $decode.string,
        (target) => { return $decode.success(new Mapping(source, target)); },
      );
    },
  );
}

function role_decoder() {
  let _block;
  let _pipe = $decode.string;
  _block = $decode.map(
    _pipe,
    (value) => {
      if (value === "base") {
        return new RoleData(new RoleBase(), "", toList([]));
      } else {
        return new RoleData(new RoleNone(), "", toList([]));
      }
    },
  );
  let as_string = _block;
  let as_consumer = $decode.subfield(
    toList(["performance_base", "pack"]),
    $decode.string,
    (pack) => {
      return $decode.subfield(
        toList(["performance_base", "mappings"]),
        $decode.list(mapping_decoder()),
        (mappings) => {
          return $decode.success(
            new RoleData(new RoleConsumer(), pack, mappings),
          );
        },
      );
    },
  );
  return $decode.one_of(as_string, toList([as_consumer]));
}

function form_variant_decoder() {
  return $decode.optional_field(
    "mc_version",
    "",
    $decode.string,
    (mc_version) => {
      return $decode.optional_field(
        "id",
        "",
        $decode.string,
        (id) => {
          return $decode.optional_field(
            "name",
            "",
            $decode.string,
            (name) => {
              return $decode.optional_field(
                "version",
                "",
                $decode.string,
                (version) => {
                  return $decode.optional_field(
                    "release_type",
                    "",
                    $decode.string,
                    (release_type) => {
                      return $decode.optional_field(
                        "loader",
                        "",
                        $decode.string,
                        (loader) => {
                          return $decode.success(
                            new FormVariant(
                              mc_version,
                              id,
                              name,
                              version,
                              release_type,
                              loader,
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

function form_decoder() {
  return $decode.optional_field(
    "$schema",
    "",
    $decode.string,
    (schema) => {
      return $decode.optional_field(
        "id",
        "",
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
                            "version",
                            "",
                            $decode.string,
                            (version) => {
                              return $decode.optional_field(
                                "mc_version",
                                "",
                                $decode.string,
                                (mc_version) => {
                                  return $decode.optional_field(
                                    "variants",
                                    toList([]),
                                    $decode.list(form_variant_decoder()),
                                    (variants) => {
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
                                                            "lifecycle",
                                                            "",
                                                            $decode.string,
                                                            (lifecycle) => {
                                                              return $decode.optional_field(
                                                                "role",
                                                                new RoleData(
                                                                  new RoleNone(),
                                                                  "",
                                                                  toList([]),
                                                                ),
                                                                role_decoder(),
                                                                (role) => {
                                                                  return $decode.optional_field(
                                                                    "shared_assets",
                                                                    "",
                                                                    $decode.string,
                                                                    (
                                                                        shared_assets
                                                                      ) => {
                                                                      return $decode.optional_field(
                                                                        "automation",
                                                                        new None(),
                                                                        $decode.map(
                                                                          automation_decoder(),
                                                                          (var0) => {
                                                                            return new Some(
                                                                              var0,
                                                                            );
                                                                          },
                                                                        ),
                                                                        (
                                                                            automation
                                                                          ) => {
                                                                          return $decode.success(
                                                                            new ManifestForm(
                                                                              schema,
                                                                              id,
                                                                              name,
                                                                              kind,
                                                                              loader,
                                                                              release_type,
                                                                              version,
                                                                              mc_version,
                                                                              !isEqual(
                                                                                variants,
                                                                                toList([])
                                                                              ),
                                                                              variants,
                                                                              modrinth_id,
                                                                              curseforge_id,
                                                                              github_id,
                                                                              gitea_id,
                                                                              gitlab_id,
                                                                              lifecycle,
                                                                              role.kind,
                                                                              role.pack,
                                                                              role.mappings,
                                                                              shared_assets,
                                                                              automation,
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
}

export function parse(raw) {
  let $ = $json.parse(raw, form_decoder());
  if ($ instanceof Ok) {
    return $;
  } else {
    let error = $[0];
    return new Error(describe_parse_error(error));
  }
}

function automation_json(settings) {
  let bool_field = (key, value) => {
    if (value instanceof Some) {
      let b = value[0];
      return toList([[key, $json.bool(b)]]);
    } else {
      return toList([]);
    }
  };
  let _block;
  let $ = settings.sync_exclude;
  if ($ instanceof $Empty) {
    _block = $;
  } else {
    let values = $;
    _block = toList([["sync_exclude", $json.array(values, $json.string)]]);
  }
  let sync_exclude = _block;
  let _block$1;
  let $1 = settings.freeze;
  if ($1 instanceof $Empty) {
    _block$1 = $1;
  } else {
    let entries = $1;
    _block$1 = toList([
      [
        "freeze",
        $json.object(
          $list.map(
            entries,
            (entry) => {
              return [entry[0], $json.array(entry[1], $json.string)];
            },
          ),
        ),
      ],
    ]);
  }
  let freeze = _block$1;
  return $json.object(
    $list.flatten(
      toList([
        bool_field("auto_update", settings.auto_update),
        bool_field("server_promo", settings.server_promo),
        sync_exclude,
        freeze,
      ]),
    ),
  );
}

/**
 * Serialize the form back to manifest JSON (2-space indented).
 */
export function serialize(form) {
  let optional_string = (key, value) => {
    let $ = $string.trim(value);
    if ($ === "") {
      return toList([]);
    } else {
      return toList([[key, $json.string(value)]]);
    }
  };
  let _block;
  let $ = form.use_variants;
  if ($) {
    _block = toList([
      [
        "variants",
        $json.array(
          form.variants,
          (v) => {
            return $json.object(
              (() => {
                let _pipe = toList([["mc_version", $json.string(v.mc_version)]]);
                let _pipe$1 = $list.append(_pipe, optional_string("id", v.id));
                let _pipe$2 = $list.append(
                  _pipe$1,
                  optional_string("name", v.name),
                );
                let _pipe$3 = $list.append(
                  _pipe$2,
                  optional_string("version", v.version),
                );
                let _pipe$4 = $list.append(
                  _pipe$3,
                  optional_string("release_type", v.release_type),
                );
                return $list.append(
                  _pipe$4,
                  optional_string("loader", v.loader),
                );
              })(),
            );
          },
        ),
      ],
    ]);
  } else {
    _block = toList([["mc_version", $json.string(form.mc_version)]]);
  }
  let shape = _block;
  let _block$1;
  let $1 = form.role_kind;
  if ($1 instanceof RoleNone) {
    _block$1 = $json.string("none");
  } else if ($1 instanceof RoleBase) {
    _block$1 = $json.string("base");
  } else {
    _block$1 = $json.object(
      toList([
        [
          "performance_base",
          $json.object(
            toList([
              ["pack", $json.string(form.role_pack)],
              [
                "mappings",
                $json.array(
                  form.role_mappings,
                  (m) => {
                    return $json.object(
                      toList([
                        ["source", $json.string(m.source)],
                        ["target", $json.string(m.target)],
                      ]),
                    );
                  },
                ),
              ],
            ]),
          ),
        ],
      ]),
    );
  }
  let role = _block$1;
  let platform_ids = $list.flatten(
    toList([
      optional_string("modrinth_id", form.modrinth_id),
      optional_string("curseforge_id", form.curseforge_id),
      optional_string("github_id", form.github_id),
      optional_string("gitea_id", form.gitea_id),
      optional_string("gitlab_id", form.gitlab_id),
    ]),
  );
  let _block$2;
  if (platform_ids instanceof $Empty) {
    _block$2 = toList([["modrinth_id", $json.string("")]]);
  } else {
    _block$2 = platform_ids;
  }
  let platform_ids$1 = _block$2;
  let _block$3;
  let $2 = form.automation;
  if ($2 instanceof Some) {
    let settings = $2[0];
    _block$3 = toList([["automation", automation_json(settings)]]);
  } else {
    _block$3 = toList([]);
  }
  let automation = _block$3;
  let pairs = $list.flatten(
    toList([
      optional_string("$schema", form.schema),
      toList([
        ["id", $json.string(form.id)],
        ["name", $json.string(form.name)],
        ["type", $json.string(form.kind)],
      ]),
      optional_string("loader", form.loader),
      optional_string("version", form.version),
      shape,
      toList([["release_type", $json.string(form.release_type)], ["role", role]]),
      optional_string("lifecycle", form.lifecycle),
      platform_ids$1,
      optional_string("shared_assets", form.shared_assets),
      automation,
    ]),
  );
  let _pipe = $json.object(pairs);
  let _pipe$1 = $json.to_string(_pipe);
  return pretty_json(_pipe$1);
}

function platform_suffix(value) {
  let $ = $string.ends_with(value, "-mr");
  let $1 = $string.ends_with(value, "-cf");
  if ($) {
    return "mr";
  } else if ($1) {
    return "cf";
  } else {
    return "";
  }
}

function int_to_string(value) {
  return $int.to_string(value);
}

function validate_mapping(mapping, index) {
  let label = ("mapping[" + int_to_string(index)) + "]";
  let prefix = "Mapping " + int_to_string(index + 1);
  let source_suffix = platform_suffix(mapping.source);
  let target_suffix = platform_suffix(mapping.target);
  return $list.flatten(
    toList([
      (() => {
        if (source_suffix === "") {
          return toList([
            new Issue(
              label,
              new IssueError(),
              prefix + ": source must end in -mr or -cf.",
            ),
          ]);
        } else {
          return toList([]);
        }
      })(),
      (() => {
        if (target_suffix === "") {
          return toList([
            new Issue(
              label,
              new IssueError(),
              prefix + ": target must end in -mr or -cf.",
            ),
          ]);
        } else {
          return toList([]);
        }
      })(),
      (() => {
        let $ = ((source_suffix !== "") && (target_suffix !== "")) && (source_suffix !== target_suffix);
        if ($) {
          return toList([
            new Issue(
              label,
              new IssueError(),
              prefix + ": source and target must share a platform suffix (MR/CF must never cross).",
            ),
          ]);
        } else {
          return toList([]);
        }
      })(),
    ]),
  );
}

function all_variants_have_loaders(form) {
  return (form.use_variants && (!isEqual(form.variants, toList([])))) && $list.all(
    form.variants,
    (v) => { return $string.trim(v.loader) !== ""; },
  );
}

export function validate(form) {
  let required = (field, value, label) => {
    let $ = $string.trim(value);
    if ($ === "") {
      return toList([
        new Issue(field, new IssueError(), label + " is required."),
      ]);
    } else {
      return toList([]);
    }
  };
  let identity = $list.flatten(
    toList([
      required("id", form.id, "Pack ID"),
      required("name", form.name, "Name"),
      required("type", form.kind, "Type"),
      required("release_type", form.release_type, "Release type"),
      required("version", form.version, "Version"),
    ]),
  );
  let _block;
  let $ = ((form.kind === "modpack") && ($string.trim(form.loader) === "")) && !all_variants_have_loaders(
    form,
  );
  if ($) {
    _block = toList([
      new Issue(
        "loader",
        new IssueError(),
        "Modpacks must declare a loader (pack-level or on every variant).",
      ),
    ]);
  } else {
    _block = toList([]);
  }
  let loader = _block;
  let _block$1;
  let $1 = form.use_variants;
  if ($1) {
    let $2 = form.variants;
    if ($2 instanceof $Empty) {
      _block$1 = toList([
        new Issue("variants", new IssueError(), "Add at least one variant."),
      ]);
    } else {
      let variants = $2;
      let _pipe = variants;
      let _pipe$1 = $list.index_map(
        _pipe,
        (variant, index) => {
          let label = ("variants[" + int_to_string(index)) + "]";
          let $3 = $string.trim(variant.mc_version);
          if ($3 === "") {
            return toList([
              new Issue(
                label,
                new IssueError(),
                ("Variant " + int_to_string(index + 1)) + " needs mc_version.",
              ),
            ]);
          } else {
            return toList([]);
          }
        },
      );
      _block$1 = $list.flatten(_pipe$1);
    }
  } else {
    _block$1 = required("mc_version", form.mc_version, "Minecraft version");
  }
  let shape = _block$1;
  let _block$2;
  let $2 = (() => {
    let _pipe = toList([
      form.modrinth_id,
      form.curseforge_id,
      form.github_id,
      form.gitea_id,
      form.gitlab_id,
    ]);
    return $list.any(_pipe, (value) => { return $string.trim(value) !== ""; });
  })();
  if ($2) {
    _block$2 = toList([]);
  } else {
    let _block$3;
    let $3 = form.kind;
    if ($3 === "resourcepack") {
      _block$3 = new IssueWarning();
    } else {
      _block$3 = new IssueError();
    }
    let severity = _block$3;
    _block$2 = toList([
      new Issue(
        "platforms",
        severity,
        "Set at least one platform id (Modrinth, CurseForge, GitHub, Gitea, or GitLab).",
      ),
    ]);
  }
  let platforms = _block$2;
  let _block$3;
  let $3 = form.role_kind;
  if ($3 instanceof RoleConsumer) {
    _block$3 = $list.flatten(
      toList([
        required("role_pack", form.role_pack, "Performance base pack"),
        (() => {
          let $4 = form.role_mappings;
          if ($4 instanceof $Empty) {
            return toList([
              new Issue(
                "role_mappings",
                new IssueError(),
                "Add at least one base mapping.",
              ),
            ]);
          } else {
            let mappings = $4;
            let _pipe = mappings;
            let _pipe$1 = $list.index_map(
              _pipe,
              (mapping, index) => { return validate_mapping(mapping, index); },
            );
            return $list.flatten(_pipe$1);
          }
        })(),
      ]),
    );
  } else {
    _block$3 = toList([]);
  }
  let role = _block$3;
  let _block$4;
  let $4 = form.lifecycle;
  if ($4 === "") {
    _block$4 = toList([]);
  } else if ($4 === "active") {
    _block$4 = toList([]);
  } else if ($4 === "maintenance") {
    _block$4 = toList([]);
  } else if ($4 === "archived") {
    _block$4 = toList([]);
  } else if ($4 === "eol") {
    _block$4 = toList([]);
  } else {
    let other = $4;
    _block$4 = toList([
      new Issue(
        "lifecycle",
        new IssueError(),
        ("Invalid lifecycle '" + other) + "' (active, maintenance, archived, eol).",
      ),
    ]);
  }
  let lifecycle = _block$4;
  return $list.flatten(
    toList([identity, loader, shape, platforms, role, lifecycle]),
  );
}

export function errors(issues) {
  return $list.filter(
    issues,
    (issue) => { return issue.severity instanceof IssueError; },
  );
}

export function field_issues(issues, field) {
  return $list.filter(issues, (issue) => { return issue.field === field; });
}
