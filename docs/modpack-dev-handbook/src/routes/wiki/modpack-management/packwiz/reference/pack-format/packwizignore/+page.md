# .packwizignore

`.packwizignore` works like `.gitignore`, but for `packwiz refresh` and export generation.

Use it to exclude files that should exist in your working directory without being indexed into the pack manifest.

## Common entries

```text
.git/**
.gitattributes
.gitignore
*.mrpack
*.zip
```

## When to use it

Add entries when a file is part of your local workflow but should not be treated as pack content.

Typical examples include Git metadata, exported archives, local notes, or temporary tooling output. If a file should never be downloaded by players or mirrored into the pack index, `.packwizignore` is the right place to exclude it.
