# Additional options
Additional options can be configured in the `[options]` section of `pack.toml`, as follows:

- `acceptable-game-versions` A list of additional Minecraft versions to accept when installing or updating mods (see [Adding mods](/wiki/modpack-management/packwiz/reference/../tutorials/creating/adding-mods))
- `meta-folder` The folder in which new metadata files will be added, defaulting to a folder based on the category (mods, resourcepacks, etc; if the category is unknown the current directory is used)
	- `mods-folder` is now deprecated; aliassed to `meta-folder`
- `meta-folder-base` The base folder from which meta-folder will be resolved, defaulting to the current directory (so you can put all mods/etc in a subfolder while still using the default behaviour)
- `no-internal-hashes` If this is set to true, packwiz will not generate hashes of local files, to prevent merge conflicts and inconsistent hashes when using git/etc.
	- `packwiz refresh --build` can be used in this mode to generate internal hashes for distributing the pack with [packwiz-installer]
- `datapack-folder` The folder in which datapacks are to be added; specific to the datapack loader mod you use, and must be set to add datapacks (that are not bundled as mods)