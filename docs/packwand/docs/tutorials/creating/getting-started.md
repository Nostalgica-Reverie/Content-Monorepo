# Getting started

To use the packwand CLI, first you'll want to create a folder to develop your modpack in. This should not be the same as your `.minecraft` or a MultiMC instance folder; this folder holds metadata and files for your modpack so it can be managed by packwand. Then open your command line (Command Prompt/Terminal) and use the `cd` command to move into the folder you created.

## Creating a new modpack

To create the files for your new modpack, just run `packwand init` in the folder you created! It'll ask you for a few details, then create a `pack.toml` and `index.toml` based on your answers.

`pack.toml` is the main file of your modpack and defines several crucial details; including the name of your modpack, the version of Minecraft and the version of the mod loader you're using. Optionally, you can include a version (required for exporting to Modrinth packs) and a description for your modpack.

`index.toml` is the index of your modpack which lists the files in your modpack with their hashes (for integrity checking). You're unlikely to need to touch this yourself, but you'll need to run the `packwand refresh` command when you manually add, remove or edit files in the pack.

## Importing an existing modpack

Have an existing CurseForge modpack? You can use the `packwand curseforge import` command with the path to the modpack `.zip` file, which will import all the mods and files from the pack into your current directory. If this isn't your own modpack, please make sure you have permission (or a license) to redistribute the modpack you import!

::: warning
If you have existing files in your modpack, importing will overwrite them. It's a good idea to use version control systems (such as Git) with packwand!
:::

## Cheat Sheet

You'll get more information in the tutorials following this one (and the reference pages), but this is a quick summary of the most useful commands:

- `packwand init` creates a modpack in the current folder
- `packwand curseforge import [zip path]` imports a CurseForge modpack
- `packwand refresh` updates the modpack index
- `packwand curseforge install [mod]` installs a mod from CurseForge
- `packwand modrinth install [mod]` installs a mod from Modrinth
- `packwand update [mod]` updates a mod
- `packwand update --all` updates all the mods in the modpack
- `packwand curseforge export` exports the modpack in the format supported by the CurseForge Launcher
- `packwand modrinth export` exports the modpack in the format supported by Modrinth
- `packwand curseforge detect` to detect files that are available on CurseForge and make them downloaded from there
- `packwand workspace status` shows every pack in a multi-pack repository
- Use the `--help` flag for more information about any command!
