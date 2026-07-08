# manifest.json

The packwand manifest is the root metadata file for a pack directory. It stores the pack's identity, loader/version matrix, publishing identifiers, role, lifecycle, and automation settings.

packwand reads and writes `manifest.json` in each pack directory. Commands such as `packwand new`, `packwand validate`, `packwand publish`, `packwand automation`, and the workspace operations all treat it as the pack's source of truth.

## Required fields

- `id` Unique pack identifier, usually the directory name
- `name` Human-readable pack name
- `type` Pack kind, such as `modpack`, `datapack`, or `resourcepack`
- `role` Pack role, usually `none`, `base`, or a consumer/base mapping object

## Common fields

- `loader` Primary loader for the pack
- `mc_version` Primary Minecraft version for the pack
- `variants` Optional variant list for multi-version packs
- `version` Pack release version
- `release_type` Release channel label used by publish workflows
- `description` Short pack description
- `$schema` Optional schema URL for editor tooling
- `modrinth_id`, `curseforge_id`, `github_id`, `gitea_id`, `gitlab_id` External publishing identifiers
- `shared_assets` Shared asset path used by base/consumer pack layouts
- `lifecycle` Pack maintenance state: `active`, `maintenance`, `archived`, or `eol`

## Variants

Each entry in `variants` is an object with:

- `id` Optional variant identifier
- `name` Optional display name
- `mc_version` Minecraft version for that variant
- `loader` Optional loader override for that variant
- `version` Optional variant-specific pack version

## Role

`role` is deliberately flexible so the pack can describe both simple and workspace-aware setups.

- `"none"` is the default for ordinary packs
- `"base"` marks a performance base pack
- `{ "performance_base": { "pack": "...", "mappings": [...] } }` marks a consumer pack that syncs content from a base pack

## Automation

`automation` is optional. It controls unattended update and release behavior.

- `auto_update` enables or disables automatic update flows
- `server_promo` marks a pack for server promotion workflows
- `sync_exclude` lists paths to skip during workspace sync
- `freeze` maps subdirs to frozen mod slugs that should not update
- `full_auto.enabled` opts into the end-to-end `packwand automation run` pipeline
- `full_auto.tests` is an optional list of shell commands run before the manifest version bump

## Example

```json
{
  "$schema": "./manifest.schema.json",
  "id": "re-console-main",
  "name": "Re-Console",
  "type": "modpack",
  "loader": "fabric",
  "mc_version": "26.1.2",
  "version": "26.07",
  "release_type": "release",
  "description": "Re-Console modpack",
  "modrinth_id": "...",
  "curseforge_id": "...",
  "role": "none",
  "lifecycle": "active",
  "automation": {
    "auto_update": true,
    "full_auto": {
      "enabled": false,
      "tests": []
    }
  }
}
```
