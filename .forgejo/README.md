# Content Monorepo Forgejo Actions
These are all of the workflows used in the Lasting Legacy Content Monorepo.

## Current Functions
- Auto Publish
- Auto Update*
- Auto Refresh*
- Auto Build
- Datapack Validator
- JSON Linter
- Resource Pack Validator
- Resource Pack Compressor**

*for modpacks only

**on publish and build only

## Using Auto Publish (for devs)
Actions currently use a tag-based release system. Once a tag is created with a certain prefix, it will automatically publish the associated project to Modrinth, Curseforge, and Forgejo.

### Current tags
- RC (Re-Console)
- SL (Simply Legacy)
- 2K (2000's Edition)
- LPC (LCE Panorama Collection)
- LSP (Legacy Skin Packs)
- MSP (Modern Skin Packs)
- O4J (Ore4J)
- FL (Faithful Legacy)
- TUT (Tutorial World Add-on)
- LM (Legacy Mechanics)
- LMA (Legacy Mechanics Add-on)
- LN (Legacy Nether)
- LNE (Legacy Nether Extended)
- HTP (Modern How To Play)
- LT (Legacy Titles)
- VL (Vanilla Live)
- TU0 (Old UI for Legacy4J)

### Creating a release
To create a release using auto-publish, simply create a Forgejo release. So it knows what project you're publishing for, refer to the above tag key. To make a version, all tags must be TAG-x.y.z. ie, Re-Console 26.03.4, would be RC-26.03.4. For an example of SemVer, LN-3.0.0 would also work. Any version number that is at least 3 numbers long will function.

## Incremental Builds
Only the pack modified within a commit will be built. So if you modified something in, lets say Simply Legacy, your commit would only build Simply Legacy, and not Re-Console or 2000's Edition.

# License
Workflows are licensed under the GNU AFFERO GENERAL PUBLIC LICENSE, unless explicitly stated otherwise.

