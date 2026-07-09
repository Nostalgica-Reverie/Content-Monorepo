<script>
  import { siteConfig } from '$lib/site';
</script>

# Installation

Prebuilt binaries are available from <a href={siteConfig.packwiz.actionsUrl}>GitHub Actions</a>. The UI is awkward, but the general flow is to open the latest successful build and download the artifact zip for your system from the artifacts section. To run the executable, add the folder where you downloaded it to your `PATH` environment variable ([see tutorial for Windows here](https://www.howtogeek.com/118594/how-to-edit-your-system-path-for-easy-command-line-access/)) or move it somewhere already on `PATH`.

If you do not have a GitHub account or cannot download directly from GitHub, you can also use <a href={siteConfig.packwiz.nightlyUrl}>nightly.link</a>.

You can also compile from source:

1. Install Go (1.19 or newer) from https://golang.org/dl/
2. Run `go install github.com/packwiz/packwiz@latest`

Be patient on the first run; Go needs to download and compile dependencies.

## Choosing an install path

- Use the prebuilt archive if you only need the CLI.
- Use `go install` if you already work in Go and want the fastest developer setup.
- Pair packwiz with the [bootstrap](/wiki/modpack-management/packwiz/components/bootstrap) and [installer](/wiki/modpack-management/packwiz/components/installer) when you are validating the player update path.