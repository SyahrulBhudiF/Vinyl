# Vinyl

Vinyl is a Rust visual novel engine for people writing games.
Download `vn`, make a project, write `.vn` scripts, add assets, then check and run it.

## Who is this for?

- Writers and narrative designers who want to create visual novels with text files.
- Game developers who want a renderer-independent VN runtime.
- Engine developers who need parser, validation, save, localization, and renderer integration primitives.

## Status

Vinyl currently provides a desktop MVP for Linux, macOS, and Windows:

- Indentation-based `.vn` script language.
- Project manifest via `vinyl.toml`.
- Asset resolver.
- Parser and validator with multi-file diagnostics.
- Menu, jump, condition, assignment, and expressions.
- Transition and text-effect metadata.
- Runtime VM with deterministic save and rollback.
- Localization with Fluent `.ftl` files.
- Desktop player plus writer CLI: `new`, `check`, `run`, `smoke`, `dump-ast`, `dump-ir`, `list-assets`, `extract-locales`.
- Astro Starlight documentation site in `docs/`.
- CI and release workflows for prebuilt binaries.

## Install

Download the binary for your OS from the [latest release](https://github.com/SyahrulBhudiF/Vinyl/releases/latest). Rust is not required.

### Linux

Vinyl uses X11 or XWayland. Install the common runtime libraries, then install the binary:

```bash
# Ubuntu/Debian
sudo apt install libasound2 libudev1 libx11-6 libxcursor1 libxi6 libxkbcommon-x11-0 libxrandr2

chmod +x vn-linux-x86_64
sudo mv vn-linux-x86_64 /usr/local/bin/vn
vn --help
```

Native Wayland is not currently supported.

### macOS

```bash
chmod +x vn-macos-aarch64
sudo mv vn-macos-aarch64 /usr/local/bin/vn
vn --help
```

If macOS blocks an unsigned download, allow it in **System Settings → Privacy & Security**. The current release target is Apple Silicon.

### Windows

In PowerShell, rename the download and move it somewhere permanent:

```powershell
New-Item -ItemType Directory -Force C:\Tools\Vinyl
Move-Item .\vn-windows-x86_64.exe C:\Tools\Vinyl\vn.exe
C:\Tools\Vinyl\vn.exe --help
```

Optionally add `C:\Tools\Vinyl` to your user `Path`, then use `vn` from any new terminal.

### Engine development install

Only needed if you want to work on the engine itself:

```bash
cargo build
cargo test --workspace
```

## Quickstart

These commands are the same in Bash, macOS Terminal, and PowerShell:

```bash
vn new my-game
cd my-game
vn check .
vn run .
```

`vn check` reports script, label, locale, and asset errors before the player opens. Every project must contain exactly one `label start`.

`vn new` creates this structure:

```text
my-game/
├── vinyl.toml          # project config
├── script/
│   └── start.vn        # story script
├── locale/
│   └── en-US.ftl       # translated text for en-US
└── assets/
    ├── bg/             # backgrounds
    ├── sprites/        # character sprites
    │   └── eileen/
    └── audio/          # music and sound effects
```

`locale/en-US.ftl` is a Fluent translation file. You do not download it separately; it is created by `vn new`. If you add text IDs in `.vn` files, run this to fill missing locale entries:

```bash
vn extract-locales .
```

Validate the project:

```bash
vn check .
```

Launch the desktop player:

```bash
vn run .
```

Run deterministic headless verification:

```bash
vn smoke .
```

Use a specific locale:

```bash
vn check . --locale en-US
vn run . --locale en-US
```

## Editor syntax highlighting

Vinyl includes a small VS Code-compatible syntax extension for `.vn` files:

```text
editors/vscode-vinyl/
```

Install it locally in VS Code:

```bash
cd editors/vscode-vinyl
pnpm install
pnpm package
code --install-extension vinyl-vn-syntax-0.1.0.vsix --force
```

For development, open `editors/vscode-vinyl/` in VS Code and press `F5`.

GitHub does not support custom TextMate grammars from a repository. `.vn` highlighting is handled by the editor extension, not by GitHub Linguist.

## Example `.vn` script

`script/start.vn`:

```vn
label start:
    eileen [intro-hello] "Hello."
    menu:
        [intro-continue] "Continue":
            end
```

Localization in `locale/en-US.ftl`:

```ftl
intro-hello = Hello.
intro-continue = Continue
```

Larger example:

```vn
label start:
    scene bg room with fade(duration=0.5)
    show eileen happy at center with dissolve(duration=0.4)
    $affection = 3
    eileen [intro-hello] "Hello." with typewriter(speed=45)

    menu:
        [ask-name] "Ask her name" if affection >= 3:
            eileen [name-answer] "I'm Eileen."
            jump end
        [leave] "Leave":
            jump end

label end:
    end
```

Player controls: Space/Enter/left-click advances; arrow/number keys or mouse select choices; Page Up or wheel-up rolls back; Escape pauses; F5/F9 opens Save/Load; Alt+Enter toggles fullscreen. There are 12 manual slots plus one autosave. Per-project saves and `preferences.json` use the OS data directory; preferences are outside save slots. Music restarts after load/rollback. Missing audio devices warn and continue silently. Player UI labels are English; locale affects script content only. MVP assets are PNG images and MP3 music.

Asset paths are resolved like this:

```text
scene bg room                  -> assets/bg/room.png
show eileen happy at center    -> assets/sprites/eileen/happy.png
play music theme               -> assets/audio/theme.*
```

## `vinyl.toml`

Example manifest:

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

## CLI commands

```bash
vn new <project>
```

Create a writer-ready project.

```bash
vn check <project> [--locale en-US]
```

Parse and validate scripts, labels, assets, and locale entries.

```bash
vn run <project> [--locale en-US]
```

Validate and launch the rendered desktop player. Use `vn smoke <project>` for deterministic headless VM verification.

```bash
vn list-assets <project>
```

Print all asset paths referenced by scripts.

```bash
vn extract-locales <project>
```

Generate Fluent entries from script text IDs.

```bash
vn dump-ast <project>
vn dump-ir <project>
```

Print parser/compiler output as JSON for debugging.

```bash
vn fmt <project>
```

Currently a parse/check placeholder. Full source rewriting is not implemented yet.

## Documentation

The complete guide covers installation, authoring, CLI usage, player controls, saves, architecture, testing, releases, performance, and troubleshooting:

<https://syahrulbhudif.github.io/Vinyl/>

Build it locally with:

```bash
cd docs
pnpm install
pnpm build
```

## Workspace

- `vn_core`: AST, IR, compiler, deterministic VM, save model, rollback.
- `vn_script`: project loader, parser, validator, manifest, localization, assets.
- `vn_runtime`: renderer-independent presentation commands and save storage.
- `vn_bevy`: desktop rendering, input, audio, player UI, visual testing.
- `vn_cli`: internal crate that builds the public `vn` executable.

See the architecture and development documentation for dependency boundaries, quality gates, and release packaging.
