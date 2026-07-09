# Installation

## Prebuilt binaries

Prebuilt binaries for Linux, Windows, and macOS (amd64 and arm64) are published on the [Forgejo releases page](https://git.nostalgica.net/Lasting-Legacy/Lasting-Legacy-Monorepo/releases). Download the archive for your platform, extract it, and add the folder containing the executable to your `PATH` environment variable ([see tutorial for Windows here](https://www.howtogeek.com/118594/how-to-edit-your-system-path-for-easy-command-line-access/)) or move it to where you want to use it.

Verify the download against `checksums.txt` (SHA-256) attached to the release.

## go install

With Go 1.26 or newer installed, a single command builds and installs the latest packwand from the monorepo:

```sh
go install git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand@latest
```

The binary is placed in `$(go env GOPATH)/bin` — make sure that directory is on your `PATH`.

::: tip
`@latest` resolves through the public Go module proxy, which can lag the tip of `main` by up to ~30 minutes. To fetch the newest commit straight from the repository, bypass the proxy:

```sh
GOPRIVATE=git.nostalgica.net go install git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand@latest
```
:::

## Building from source

1. Install Go (1.26 or newer) from https://golang.org/dl/
2. Clone the monorepo and build:

```sh
git clone https://git.nostalgica.net/Lasting-Legacy/Lasting-Legacy-Monorepo.git
cd Lasting-Legacy-Monorepo/apps/packwand
go build -o packwand .
```

Be patient the first time — Go has to download and compile dependencies as well!

::: tip
Tools in this repository that shell out to packwand respect the `PACKWAND_BIN` environment variable if you want to point them at a specific binary.
:::
