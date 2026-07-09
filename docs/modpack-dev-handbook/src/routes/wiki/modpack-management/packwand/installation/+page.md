<script>
  import { siteConfig } from '$lib/site';
</script>

# Installation

## Prebuilt binaries

Prebuilt binaries for Linux, Windows, and macOS (amd64 and arm64) are published on the <a href={siteConfig.packwand.releasesUrl}>Forgejo releases page</a>. Download the archive for your platform, extract it, and add the folder containing the executable to your `PATH` environment variable ([see tutorial for Windows here](https://www.howtogeek.com/118594/how-to-edit-your-system-path-for-easy-command-line-access/)) or move it to where you want to use it.

Verify the download against `checksums.txt` (SHA-256) attached to the release.

## go install

With Go 1.26 or newer installed, a single command builds and installs the latest packwand from the repository:

```sh
go install git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand@latest
```

The binary is placed in `$(go env GOPATH)/bin` - make sure that directory is on your `PATH`.

::: tip
`@latest` resolves through the public Go module proxy, which can lag the tip of `main` by up to ~30 minutes. To fetch the newest commit straight from the repository, bypass the proxy:

```sh
GOPRIVATE=git.nostalgica.net go install git.nostalgica.net/Reverie-Projects/monorepo/apps/packwand@latest
```
:::

## Building from source

1. Install Go (1.26 or newer) from https://golang.org/dl/
2. Clone the repository and build:

```sh
git clone https://git.nostalgica.net/Reverie-Projects/monorepo.git
cd monorepo/apps/packwand
go build -o packwand .
```

Be patient the first time - Go has to download and compile dependencies as well.

## Which install path should you choose?

- Use the release archive if you just want a stable binary on your workstation.
- Use `go install` if you already have Go installed and want the CLI on your developer machine quickly.
- Build from source when you need to modify packwand itself, test a branch, or produce binaries in CI.

::: tip
Tools in this repository that shell out to packwand respect the `PACKWAND_BIN` environment variable if you want to point them at a specific binary.
:::