# Vinyl

Vinyl is a visual novel engine written in Rust, but **game developers do not need to write or build Rust**.
The intended workflow is simple: download the prebuilt `vn_cli` binary, create a project, write `.vn` scripts, add assets, validate, and run/export through the tooling.

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
- Writer CLI: `new`, `check`, `run`, `dump-ast`, `dump-ir`, `list-assets`, `extract-locales`.
- Astro Starlight documentation site in `docs/`.
- CI and release workflows for prebuilt binaries.

## Install

### Normal install for game developers

Download a binary from GitHub Releases:

- Linux: `vn_cli-linux-x86_64`
- macOS: `vn_cli-macos-aarch64`
- Windows: `vn_cli-windows-x86_64.exe`

Rename it to `vn_cli` and place it somewhere in your PATH.

Linux/macOS example:

```bash
chmod +x vn_cli-linux-x86_64
mv vn_cli-linux-x86_64 vn_cli
./vn_cli --help
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
vn_cli new my-game
cd my-game
```

Generated structure:

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

Validate the project:

```bash
vn_cli check .
```

Run a deterministic CLI smoke test:

```bash
vn_cli run .
```

Use a specific locale:

```bash
vn_cli check . --locale en-US
vn_cli run . --locale en-US
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
vn_cli new <project>
```

Create a writer-ready project.

```bash
vn_cli check <project> [--locale en-US]
```

Parse and validate scripts, labels, assets, and locale entries.

```bash
vn_cli run <project> [--locale en-US]
```

Run the project through the deterministic CLI runtime. Useful for smoke testing script execution.

```bash
vn_cli list-assets <project>
```

Print all asset paths referenced by scripts.

```bash
vn_cli extract-locales <project>
```

Generate Fluent entries from script text IDs.

```bash
vn_cli dump-ast <project>
vn_cli dump-ir <project>
```

Print parser/compiler output as JSON for debugging.

```bash
vn_cli fmt <project>
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

GitHub builds `vn_cli` for Linux, macOS, and Windows, then uploads the binaries to GitHub Releases.

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
- `vn_cli`: writer/developer CLI.
