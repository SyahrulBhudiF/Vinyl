---
title: Project Layout and Manifest
---

## Generated structure

`vn new my-game` creates:

```text
my-game/
├── vinyl.toml
├── script/
│   └── start.vn
├── locale/
│   └── en-US.ftl
└── assets/
    ├── bg/
    ├── sprites/
    │   └── eileen/
    └── audio/
```

All `.vn` files under the configured script directory are loaded recursively. Files are sorted by path before parsing, then combined into one project script. Labels and jumps may cross file boundaries, but the complete project must still contain exactly one `label start`. Execution begins there, not in the first file by path.

## `vinyl.toml`

```toml
[project]
id = "my-game"
title = "My Game"
version = "0.1.0"
default_locale = "en-US"

[paths]
script = "script"
assets = "assets"
locales = "locale"

[assets]
backgrounds = "bg"
sprites = "sprites"
audio = "audio"
```

### `[project]`

| Field | Purpose |
|---|---|
| `id` | Stable project identifier used for save-directory ownership and compatibility. |
| `title` | Human-readable game title. |
| `version` | Project version stored in saves and checked when loading. |
| `default_locale` | Locale selected when `--locale` is omitted. |

Do not casually change `id` or `version` after publishing saves: incompatible slots remain visible but cannot be loaded.

### `[paths]`

Paths are relative to the project root.

| Field | Default |
|---|---|
| `script` | `script` |
| `assets` | `assets` |
| `locales` | `locale` |

### `[assets]`

These directories are relative to `paths.assets`.

| Field | Default |
|---|---|
| `backgrounds` | `bg` |
| `sprites` | `sprites` |
| `audio` | `audio` |

## Optional manifest

If `vinyl.toml` is absent, Vinyl uses the project directory name as its ID and title plus the default paths above. Published projects should include an explicit manifest so save identity and content roots remain stable.

## Script identity

Vinyl hashes every loaded script path and its content. Saves are compatible only when their game ID, project version, script hash, and save schema match the running project.

See [`examples/branching-mystery/`](https://github.com/SyahrulBhudiF/Vinyl/tree/main/examples/branching-mystery) or the [Complete Example Game](/Vinyl/example-game/) for a tested project containing scripts, assets, and two locales.
