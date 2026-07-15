# Vinyl

Vinyl is a Rust visual novel engine for people writing games.
Download `vn`, make a project, write `.vn` scripts, add assets, then check and run it.

## Who is this for?

- Writers and narrative designers who want to create visual novels with text files.
- Game developers who want a renderer-independent VN runtime.
- Engine developers who need parser, validation, save, localization, and renderer integration primitives.

## Status

MVP engine and CLI tooling are in progress. Currently included:

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

### Normal install for game developers

Download a binary from the latest GitHub Release:

<https://github.com/SyahrulBhudiF/Vinyl/releases/latest>

Pick the file for your OS:

- Linux: `vn-linux-x86_64`
- macOS: `vn-macos-aarch64`
- Windows: `vn-windows-x86_64.exe`

Rename it to `vn` and place it somewhere in your PATH.

Linux/macOS example:

```bash
chmod +x vn-linux-x86_64
mv vn-linux-x86_64 vn
./vn --help
```

Linux uses X11 or XWayland and requires ALSA, udev, X11, cursor, input, and RandR runtime libraries. Native Wayland and web builds are deferred.

Windows example:

1. Download `vn-windows-x86_64.exe`.
2. Rename it to `vn.exe`.
3. Move it to a folder, for example `C:\Tools\Vinyl\`.
4. Add that folder to `Path`:
   - Start menu → search “Environment Variables”.
   - Open “Edit the system environment variables”.
   - Click “Environment Variables…”.
   - Under user variables, select `Path` → “Edit”.
   - Add `C:\Tools\Vinyl\`.
5. Open a new PowerShell window:

```powershell
vn --help
```

### Engine development install

Only needed if you want to work on the engine itself:

```bash
cargo build
cargo test --workspace
```

## Quickstart

Create a new project:

```bash
vn new my-game
cd my-game
```

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

## Documentation site

The docs site lives in `docs/` and uses Astro Starlight.

```bash
cd docs
npm install
npm run dev
npm run build
```

## What are GitHub Actions for?

GitHub Actions are automation jobs that run on GitHub when code is pushed, a pull request is opened, or a release tag is created.

This repo uses them for two things:

### 1. CI quality gate

File: `.github/workflows/ci.yml`

CI runs:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --no-fail-fast --no-default-features
cd docs && npm run build
```

Why it exists:

- Check code formatting.
- Run Rust lints.
- Run automated tests.
- Verify the docs site builds.
- Prevent broken code from landing unnoticed.

### 2. Automatic release binaries

File: `.github/workflows/release.yml`

When a version tag is pushed:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub builds `vn` for Linux, macOS, and Windows, then uploads the binaries to GitHub Releases.

This is important because **game developers can download the CLI directly without installing Rust or running `cargo build`**.

## Local development checks

Before a large commit:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --no-fail-fast --no-default-features
```

## Workspace crates

- `vn_core`: AST, IR, compiler, VM, save model.
- `vn_script`: parser, validator, manifest, asset resolver, localization loader.
- `vn_runtime`: presentation orchestration.
- `vn_bevy`: Bevy renderer integration.
- `vn_cli`: internal crate that builds the public `vn` player/CLI.
