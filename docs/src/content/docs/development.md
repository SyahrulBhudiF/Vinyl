---
title: Development, Testing, and Releases
---

## Prerequisites

- Stable Rust toolchain
- Linux desktop builds: ALSA and udev development packages
- Docs: Node.js 24 and pnpm
- Visual CI: Vulkan software driver, Xvfb, and Pillow

## Workspace commands

```bash
cargo build
cargo fmt --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --no-fail-fast --no-default-features
```

Audio/desktop gate:

```bash
cargo clippy --workspace --all-targets --features audio -- -D warnings
cargo test -p vn_bevy --features audio
cargo build -p vn_cli --release --features audio
```

Run one test:

```bash
cargo test test_name
cargo test -p vn_core test_name
cargo test -p vn_bevy --test presentation_sync test_name --features audio
```

## Docs

```bash
cd docs
pnpm install
pnpm dev
pnpm build
```

The Starlight site is deployed to GitHub Pages from `main`.

## Test layers

### Unit and integration tests

Tests cover parser recovery, validation, localization, compiler/VM behavior, save identity, rollback cap, atomic storage, presentation commands, Bevy input, transitions, audio validation, and screenshot dimensions.

### Headless smoke

```bash
vn smoke fixtures/mvp
```

This checks deterministic story execution, save serialization, compatibility, choice selection, and rollback without GPU startup.

### Linux visual regression

```bash
cargo build -p vn_cli --features audio
xvfb-run -a -s '-screen 0 1280x720x24' python3 scripts/visual-ci.py
```

The harness uses a fixed 1280×720 X display and Vulkan software rendering. It captures menu and post-choice frames, compares them with Linux golden images using a small pixel tolerance, and verifies an active music entity plus a loadable save screenshot. Failed CI uploads `target/visual-ci`.

Update goldens only after reviewing the visual change:

```bash
python3 scripts/visual-ci.py --update
```

Run it under the same Xvfb environment used for comparison.

## CI workflows

| Workflow | Purpose |
|---|---|
| `ci.yml` | Formatting, no-default-feature Clippy/tests, docs build, Pages deployment. |
| `visual.yml` | Linux 1280×720 software-rendered golden test with audio. |
| `release.yml` | Tagged Linux, macOS, and Windows release builds; Linux DEB/RPM packaging. |

## Release process

```bash
git tag v0.1.0
git push origin v0.1.0
```

A `v*` tag builds:

```text
vn-linux-x86_64
vn-macos-aarch64
vn-windows-x86_64.exe
vinyl_<version>_amd64.deb
vinyl-<version>-1.*.x86_64.rpm
```

## Platform verification

Linux golden images are intentionally Linux-only because text rasterization and window composition vary by platform. macOS and Windows release jobs build the complete audio-enabled desktop executable without cross-platform pixel comparison.
