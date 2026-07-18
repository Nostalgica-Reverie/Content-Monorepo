# packwiz-installer
An installer for launching packwiz modpacks with MultiMC. You'll need [the bootstrapper](https://github.com/comp500/packwiz-installer-bootstrap/releases) to actually use this.

## CurseForge API key

The installer includes PackWand's CurseForge client API key, so CurseForge
metadata and CDN downloads work without additional configuration. Values from
`PACKWAND_CURSEFORGE_API_KEY`, `CURSEFORGE_API_KEY`, or `CF_API_KEY` are
accepted as runtime overrides for key rotation.
