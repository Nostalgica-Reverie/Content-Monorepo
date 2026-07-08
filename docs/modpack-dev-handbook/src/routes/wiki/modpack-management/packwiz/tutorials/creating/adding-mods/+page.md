# Adding mods and resource packs
To add external files to your modpack, such as mods and resource packs, you'll need `.pw.toml` metadata files to define how to download them. The `modrinth install` and `curseforge install` commands can automatically create these for you with all the necessary metadata!

## CurseForge and Modrinth
Mods and resource packs from CurseForge and Modrinth can be easily added with the `modrinth install` and `curseforge install` commands. They can also be updated with the `packwiz update` command; pass `--all` to update all your mods at once. Mods can be passed in multiple forms to these commands:

- `packwiz curseforge install indium` (by slug)
- `packwiz curseforge install --category texture-packs unity` (by slug; category and game can be specified with the corresponding flags)
- `packwiz curseforge install https://www.curseforge.com/minecraft/mc-mods/indium` (by mod page URL)
- `packwiz curseforge install https://www.curseforge.com/minecraft/mc-mods/indium/files/3535202` (by file page URL)
- `packwiz curseforge install Indium` (by search)
- `packwiz curseforge install --addon-id 459496 --file-id 3535202` (if all else fails)
- `packwiz modrinth install indium` (by slug)
- `packwiz modrinth install https://modrinth.com/mod/indium` (by mod page URL)
- `packwiz modrinth install https://modrinth.com/mod/indium/version/mfNlBb6U` (by file page URL)
- `packwiz modrinth install Fabric Rendering Sodium` (by search)
- `packwiz modrinth install Orvt0mRa` (by ID)

Dependencies are automatically picked up for you - if you don't have them already, you'll be prompted whether you want to install them. packwiz also checks if your mods are being installed for the wrong version; but you can tell it to allow more versions using the `acceptable-game-versions` field in `pack.toml`. Just add the following to the bottom of `pack.toml`, replacing the versions listed here with those you want to allow:

```toml
[options]
acceptable-game-versions = ["1.16", "1.16.1", "1.16.2", "1.16.3", "1.16.4"]
```

!!! tip

	Several aliases exist for the `curseforge` and `modrinth` commands to speed up your workflow. Try `packwiz cf add` or `packwiz mr add`!

## Internal files (config files, scripts, etc.)
Configuration files for your modpack can simply be placed in a config folder (in the same place as the mods folder) and they'll be copied to the config folder when installing the modpack. This works for any file (including quests/scripts) - place it in the modpack and it'll be installed into the corresponding location in the game folder. Make sure you run `packwiz refresh` so that the index is up to date!

This works for mods that aren't available elsewhere online too (e.g. custom mods or forks); just drop them in the `mods` folder alongside the `.pw.toml` files. This isn't ideal for Git as it's not great at handling large binary files; you could use Git LFS or you may prefer to upload them elsewhere manually and reference them from the pack - see the section below.

!!! tip

	If you don't want to include files in the modpack, you can add them to a file called `.packwizignore` in your modpack directory. This uses the [same format as gitignore](https://git-scm.com/docs/gitignore); see the [example pack](https://github.com/packwiz/packwiz-example-pack) for an example!

## Other external files
If you have external files/mods that aren't from CurseForge or Modrinth, you'll need to create the `.pw.toml` files manually. See the following for an example of how you could lay it out:

```toml
name = "Flamingo"
filename = "flamingo.jar"
side = "both"

[download]
url = "https://example.com/flamingo.jar"

# A number of tools can generate the hash for you, including 7-zip and sha256sum
# packwiz supports a number of hashes, including sha256, sha512, sha1 and md5
hash-format = "sha256"
hash = "b22d1d8fe5752533954028172c9bf3ac01b57f40c82946a3e7b1eaff389e2b87"
```

You can even create them for files that aren't mods (such as resource packs) - just make sure to use the `.pw.toml` extension and run `packwiz refresh`, so that packwiz knows that the file contains metadata.