# Publishing to CurseForge

Exporting a CurseForge pack is as simple as running `packwand curseforge export` - this gives you a `.zip` in your pack folder that you can upload to CurseForge!

Since this pack format doesn't support side-only mods, packwand can't create a pack that differs between server and client. You can use the `--side` flag to specify which mods should be exported - by default it exports a pack for Minecraft clients (containing mods with side `client` or `both`). Mods without the necessary CurseForge metadata (such as those installed from Modrinth) will be placed as JARs into the modpack zip; these must be [approved manually by CurseForge staff](https://support.curseforge.com/en/support/solutions/articles/9000197913-non-curseforge-mods).

Be wary of including files that you don't want (the `packwand` executable, and the modpack zip itself) in the pack! packwand's default ignore rules exclude the executable and `*.zip` files at the pack root.

The CurseForge pack format doesn't really support optional mods. The user won't be prompted about optional mods, but if they default to being disabled they will be disabled in the CurseForge launcher.

See [the corresponding reference page](/wiki/modpack-management/packwand/reference/commands/packwand_curseforge_export) for the full flag list.
