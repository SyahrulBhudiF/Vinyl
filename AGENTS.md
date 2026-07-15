# Vinyl

Desktop visual-novel player and writer tooling in Rust. Public executable: `vn`. Workspace crates: `vn_core`, `vn_script`, `vn_runtime`, `vn_bevy`, `vn_cli`.

## Product Contract

- `vn run [project]` validates, then opens the desktop player. Project defaults to `.`.
- `vn smoke [project]` performs deterministic headless verification.
- Exactly one `label start` is required and is always the entry point.
- Logical canvas: 1280×720, resizable 16:9 composition with letterboxing.
- Desktop targets: Linux X11/XWayland, Windows, macOS. Native Wayland and web are deferred.
- Supported MVP assets: PNG backgrounds/sprites and MP3 music.
- Player UI is English; locale affects game script content only.
- Saves: one autosave plus 12 manual slots, atomic writes, per-project OS data directory.
- Rollback persists in saves and is capped at 100 interaction checkpoints.
- Preferences live beside saves, outside save slots. Loading a save must not change preferences.
- Music restarts after load or rollback. Missing audio devices warn and continue silently.
- First advance completes an active transition/typewriter effect; the next advances the story.
- One physical mouse press performs at most one story action. A click revealing a menu cannot select it.

## Architecture

- `vn_core`: AST, IR, compiler, deterministic VM, save schema, rollback. Renderer-independent.
- `vn_script`: manifest/project loading, parser, validation, localization, asset resolution.
- `vn_runtime`: VM-event to presentation-command mapping, presentation snapshot, save storage.
- `vn_bevy`: Bevy window/render/input/audio/player UI and deterministic visual harness.
- `vn_cli`: internal crate producing public binary `vn`; command routing and project scaffolding.

Dependency direction:

```text
vn_core <- vn_script
vn_core <- vn_runtime <- vn_bevy <- vn_cli
              ^            ^
              └─ vn_script ┘
```

Keep VM/save semantics out of Bevy. Keep renderer-specific handles/entities out of `vn_core` and serialized state.

## Build and Test

Use one Cargo build job for expensive desktop builds when system resources matter.

```bash
cargo build
cargo fmt --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --no-fail-fast --no-default-features
```

Desktop/audio gate:

```bash
CARGO_BUILD_JOBS=1 cargo clippy --workspace --all-targets --features audio -- -D warnings
CARGO_BUILD_JOBS=1 cargo test -p vn_bevy --features audio
CARGO_BUILD_JOBS=1 cargo build -p vn_cli --release --features audio
```

Single test:

```bash
cargo test test_name
cargo test -p vn_core test_name
cargo test -p vn_bevy --test presentation_sync test_name --features audio
```

Docs:

```bash
cd docs
pnpm install
pnpm build
```

Linux visual CI:

```bash
CARGO_BUILD_JOBS=1 cargo build -p vn_cli --features audio
WGPU_BACKEND=vulkan LIBGL_ALWAYS_SOFTWARE=1 \
  xvfb-run -a -s '-screen 0 1280x720x24' python3 scripts/visual-ci.py
```

Do not regenerate goldens under a compositor-sized window. Expected captures are exactly 1280×720.

## Rust Style

- Rust 2024 edition; `rustfmt` is authoritative.
- `thiserror` for library errors; `anyhow` for CLI/application errors.
- Use `serde` on persisted/public data that crosses crate or process boundaries.
- Public APIs need concise `///` docs.
- Prefer borrowing over cloning; avoid hidden allocations in interaction loops.
- Use Rust naming conventions and Clippy-clean code.
- Keep imports rustfmt-organized; do not hand-format groups against rustfmt.
- Integration tests belong in `tests/`. Small private implementation tests may stay inline.
- Do not add tests for guarantees already enforced by the type system.
- Non-trivial parser, state-transition, save, input, or security fixes need one focused regression test.

## Implementation Rules

- Fix root causes at the shared flow, not each caller.
- Reuse existing project types/systems before adding helpers or dependencies.
- No compatibility wrappers, legacy shims, speculative abstractions, or convenience re-exports.
- Treat the project as greenfield unless explicitly told otherwise.
- Keep diffs small; deletion is preferred when behavior remains complete.
- Do not overwrite unrelated changes. Parallel work may exist in other files.
- Validate trust boundaries: project files, save JSON, locale files, and assets.
- Preserve atomic save/preference writes: temporary file, flush, `sync_all`, rename.
- Never open the desktop window before project validation succeeds.
- Asset loading must retain strong handles and gate presentation commits on readiness.
- Corrupt assets enter Runtime Error with path/cause; corrupt MP3 must be preflight-decoded.
- Story/menu input stays locked during loading, transitions, typewriter reveal, and overlays.
- GPU-backed player cleanup must be graceful: AppExit or SIGINT, then wait. Never use SIGTERM/SIGKILL as fallback.

## Performance Policy

- Measure before changing data layout.
- `soa-rs` is confined to rollback history. Existing measurement showed no inline-memory or JSON reduction versus equivalent AoS for current checkpoints.
- Do not convert save slots, VM ops, UI nodes, presentation maps, or Bevy entities to SoA without a separate reproducible benchmark.
- Keep rollback capped at 100 even if storage becomes cheaper.
- Prefer on-demand assets over loading an entire project.
- Record benchmark workload, release/debug mode, relevant feature flags, and limitations. Do not present synthetic results as product guarantees.

## Script Language Constraints

- Indentation-based, four spaces; comments use `#` outside strings.
- Core statements: `label`, dialogue/narration, `scene`, `show`, `hide`, `play music`, `stop music`, `menu`, `jump`, assignment, `if`/`else`, `end`.
- Transitions: fade/dissolve duration is in seconds.
- Text effects: instant/typewriter; typewriter speed is characters per second.
- Expressions are whitespace-tokenized; parentheses are currently unsupported.
- Rendered sprite positions: left, center, right.

## Documentation

- User docs live in `docs/src/content/docs/`; sidebar is `docs/astro.config.mjs`.
- README is a concise landing page, install path, quickstart, and project overview—not an internal design diary.
- Write product-facing wording. Explain internal implementation only in architecture/development docs where it helps contributors.
- Keep commands synchronized with the public executable name `vn`, not the crate name `vn_cli`.
- When behavior, controls, CLI, save schema, platform scope, or asset support changes, update the relevant docs in the same change.
- Verify docs with `cd docs && pnpm build`.

## Version Control and Scope

- Current sessions may be on detached HEAD; check `git status --short --branch` before committing.
- Do not restore or recreate intentionally deleted planning artifacts unless asked.
- Do not include unrelated staged files in commits.
- Before completion: `cargo fmt --check`, relevant Clippy/tests, docs build when docs changed, and `git diff --check`.
