# Publishing to Modrinth

Exporting a Modrinth pack is as simple as running `packwiz modrinth export` - this gives you a `.mrpack` in your pack folder that you can upload to Modrinth!

Unlike CurseForge, this pack format does support side-only mods. When exporting, packwiz will export a pack with side information provided from Modrinth or as specified in the mod's `mod.pw.toml` file. The official Modrinth launcher will automatically filter out serverside mods, and the pack can be used on the server using tools like [mrpack-install](https://support.modrinth.com/en/articles/8802250-modpacks-on-modrinth) or [packwiz-installer](/wiki/modpack-management/packwiz/tutorials/hosting/../../tutorials/installing/packwiz-installer). Mods without the necessary Modrinth metadata (such as those installed from CurseForge) will be placed as JARs into the modpack zip; make sure that you have the licenses for these mods [as it is your responsibility as a pack creator to](https://support.modrinth.com/en/articles/8797527-obtaining-modpack-permissions).

Be wary of including files that you don't want (the `packwiz` executable, and the modpack zip itself) in the pack!

Keep in mind that since the official Modrinth app doesn't support optional mods, the user won't be prompted for optional mods. The official launcher will automatically install all optional mods (even if they default to being disabled!). If you'd like to be able to use optional mods, you use Prism Launcher's "Import Instance" section to install the exported .mrpack file. Note that if you use Prism Launcher's "Modrinth" section to install the pack, you will not be prompted for optional mods.
