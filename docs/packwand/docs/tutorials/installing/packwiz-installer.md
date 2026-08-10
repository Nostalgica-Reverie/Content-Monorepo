# Third-party installation using packwiz-installer

Packwand's desktop app has its own launcher. Its Install and Play actions run
the bundled native `packwand-installer` before starting Minecraft, so Packwand
users do not need to configure MultiMC, Prism, or a Java pre-launch task.

[packwiz-installer](https://github.com/packwiz/packwiz-installer) remains a
Java-based compatibility option when distributing Packwand-compatible packs to
third-party launchers such as MultiMC, Prism, or ATLauncher, or when integrating
a pack into an existing server startup script.

To distribute a modpack, you'll first want to set up a web hosting service (such as Netlify, GitHub Pages, GitLab Pages) so that your pack files are accessible from a HTTP/HTTPS link. For testing, you can use the `packwand serve` command to run a local HTTP server, that serves your pack at `http://localhost:8080/pack.toml` - it'll refresh the index whenever it's queried so you don't need to refresh it manually!

## Creating a MultiMC instance for your modpack

To distribute the modpack as a MultiMC instance:

1. Create a barebones MultiMC instance, with the modloader and Minecraft version you want (memory allocation overrides are also a good idea)
2. Download packwiz-installer-bootstrap from https://github.com/packwiz/packwiz-installer-bootstrap/releases and place it in the instance Minecraft folder

::: info
This is the same folder as options.txt - MultiMC will call it `.minecraft` or `minecraft` depending on your system.
:::

3. Go to Edit Instance -> Settings -> Custom commands, then check the Custom Commands box and paste the following command into the pre-launch command field:
   - `"$INST_JAVA" -jar packwiz-installer-bootstrap.jar https://[your-server]/pack.toml`
     (where `https://[your-server]/pack.toml` is the HTTP URL your `pack.toml` file is hosted at)
4. Use the Export Instance function to export your pack as a `.zip` file (which can be distributed similarly to your pack via a web hosting service)

To install your pack, users just need to add it with Add instance -> Import from zip - then packwiz-installer does the rest, keeping it up to date every time the game is launched!

## Using a modpack with a server

You can use packwiz-installer to download non-client mods (side either `both` or `server`), for example:

    java -jar packwiz-installer-bootstrap.jar -g -s server https://[your-server]/pack.toml

- `-g` flag to disable the GUI
- `-s server` to download only server-side mods.

itzg's [docker-minecraft-server](https://github.com/itzg/docker-minecraft-server) has built in support for packwiz-format packs. You can pass the `PACKWIZ_URL` environment variable pointing to your pack's TOML file, and the container will bootstrap packwiz-installer and install/update the provided pack. See [the documentation](https://docker-minecraft-server.readthedocs.io/en/latest/mods-and-plugins/packwiz/) for more information.

::: tip
For local validation, `packwand test <pack-subdir>` starts an ephemeral pack
server and invokes the native `packwand-installer` directly. It does not require
Java. Set `PACKWAND_INSTALLER_BIN` to test with a specific installer binary.
:::
