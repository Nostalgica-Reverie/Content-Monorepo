# Packeater

Packeater is Packwand's aggressive Minecraft resource-pack and datapack
optimizer. It is forked directly from PackSquash, with all 2,180 upstream
commits retained before Packeater development begins.

## Folder markers

Put a `packeater.json` file in every folder that should become an optimized ZIP.
The marker can be empty (`{}`): aggressive compression and lossy processing are
enabled by default.

```json
{
  "$schema": "path/to/packeater.schema.json",
  "version": 1,
  "output": "../dist/my-pack.zip",
  "compression": {
    "recompressCompressedFiles": true,
    "deduplicateFiles": true,
    "zipIterations": 30,
    "imageIterations": 15,
    "nbtIterations": 20
  },
  "lossy": {
    "png": true,
    "pngPalette": "eight_bit",
    "pngDithering": 0.8,
    "downsizeSingleColorImages": true,
    "audio": true,
    "audioQuality": 0,
    "audioSampleRate": 32000,
    "audioChannels": null
  }
}
```

`output` is resolved relative to the marked folder. If omitted, Packeater
writes `<folder-name>.zip` beside that folder. Packwand always supplies its
artifact path explicitly, so repository builds remain deterministic.

The default profile makes these intentional tradeoffs:

- compressed assets are recompressed and identical files may be deduplicated;
- PNGs use an eight-bit palette, 0.8 dithering, and safe single-color downsizing;
- audio is transcoded to Vorbis at quality 0 and downsampled to 32 kHz;
- channel layout is preserved by default because forced mono can alter
  Minecraft positional audio;
- ZIP, PNG, and NBT compression use 30, 15, and 20 guided iterations.

Set `lossy.png` or `lossy.audio` to `false` to disable that lossy family. Use
`pngPalette: "lossless"` for an explicit lossless PNG path. The JSON schema is
available at [`packeater.schema.json`](packeater.schema.json).

## CLI

```sh
# Discover every marker recursively from the current directory.
packeater

# Discover beneath a particular repository or folder.
packeater --discover resourcepacks

# Build one marked folder to an explicit destination.
packeater resourcepacks/example/packeater.json --output artifacts/example.zip

# Review selection without writing archives.
packeater --discover . --dry-run
```

Existing PackSquash TOML files remain accepted:

```sh
packeater packsquash.toml
```

Legacy TOML input retains its explicitly configured PackSquash behavior. Use a
`packeater.json` marker to receive Packeater's aggressive defaults.

## Packwand integration

`packwand build` and the native publish engine detect markers below datapack and
resource-pack projects. Marked variants are each built as independent
artifacts; unmarked content continues to use the deterministic plain ZIP path.

Packwand resolves the executable from `PACKEATER_BIN`, beside the running
Packwand executable, from the in-tree debug/release build, and finally from
`PATH`. A missing executable is a hard error for a marked folder, preventing CI
from silently publishing a much larger, unoptimized archive.

## Monorepo history maintenance

Packeater lives directly at `apps/packwandrs/packeater`; it is not a nested Git
repository or submodule. The monorepo commit that introduces it must retain the
PackSquash upstream head as a second parent so all upstream commits stay
reachable. During initial integration that parent is available as
`refs/packeater/upstream`.

Future upstream updates should be imported with the same subtree prefix and a
real merge parent. Do not squash or replace Packeater with a source snapshot,
because either approach discards the attribution and ancestry this fork is
intended to preserve.
