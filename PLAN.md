# Vinyl Desktop Player Plan

Track implementation of `vn run` as a real desktop visual-novel player.

## Status

- `[ ]` not started
- `[~]` active
- `[x]` done and verified
- `[!]` blocked; add reason beside item

Update the progress table and task checkbox in the same commit. A phase is complete only when its gate passes.

## Progress

| Phase | Scope | Status | Gate |
|---|---|---:|---|
| P0 | Decisions and terminology | `[x]` | `CONTEXT.md`, ADRs present |
| P1 | Correct story entry point | `[x]` | core/script tests pass |
| P2 | Public `vn` CLI and `smoke` | `[x]` | CLI tests pass |
| P3 | Desktop Bevy bootstrap | `[x]` | real window opens |
| P4 | Asset loading and memory layout | `[x]` | PNG and MP3 decode gates pass |
| P5 | Rendered story UI and input | `[x]` | first playable fixture passes |
| P6 | Correct transitions | `[x]` | fade/dissolve tests pass |
| P7 | Save/load/rollback | `[ ]` | slot round-trip tests pass |
| P8 | Pause and settings | `[ ]` | interaction tests pass |
| P9 | End-to-end visual CI | `[ ]` | Linux golden test passes |
| P10 | Packaging and documentation | `[ ]` | release builds pass |

## Locked Product Contract

- Public executable: `vn`; internal crate remains `vn_cli`.
- `vn run [project]`: rendered interactive desktop game; project defaults to `.`.
- `vn smoke [project]`: deterministic headless VM verification.
- Desktop only: Linux X11/XWayland, Windows, macOS.
- One distributed binary containing CLI and Bevy player.
- Supported MVP assets: PNG images and MP3 music.
- Required entry point: `label start`.
- Default window: resizable 1280×720, logical 16:9 canvas.
- Default player: background, sprites, dialogue, choices, audio, transitions, pause, save/load, settings.
- Twelve manual save slots plus one autosave.
- Rollback persists through save/load, capped at 100 interaction checkpoints.

## Memory Layout Rules

Apply these deliberately; do not rewrite cold/UI code for theoretical savings.

1. **Indexes instead of pointers**
   - Keep bytecode control flow as `OpId` indexes.
   - Store selected/focused UI entries as small indexes, not entity/reference graphs.
   - Save slot identity is `u8` slot index, not path-owning objects.

2. **Booleans out-of-band**
   - Keep dense player runtime flags in one compact flag word/bitset: paused, ended, muted, fullscreen, auto-advance, loading, transition-active.
   - Do not add a dependency solely for flags; a small integer wrapper is enough.

3. **Structure of Arrays with `soa-rs`**
   - Add `soa-rs` with its `serde` feature to `vn_core`.
   - Derive `Soars` for `RollbackCheckpoint` and store rollback history as `Soa<RollbackCheckpoint>`.
   - Fields: VM state and presentation snapshot. Use generated field slices when save serialization, restore, truncation, or inspection only needs one field family.
   - Keep the hard cap at 100 entries; `Soa` is not permission for unbounded history.
   - Add one size/allocation benchmark against the former `Vec<(VmState, PresentationSnapshot)>`; record result below. Keep `soa-rs` because explicitly selected, but avoid spreading it to twelve-slot UI or heterogeneous Bevy state.

4. **Sparse data in hash maps**
   - Continue using maps for script variables and visible sprites because keys are sparse and dynamic.
   - Do not replace fixed save slots or three sprite positions with maps.

5. **Encodings instead of polymorphism**
   - Continue using enums for VM ops, events, player mode, transition kind, and errors.
   - No trait objects, factories, or per-screen interface hierarchy.

### Memory Measurement

| Measurement | Before | After | Delta | Command/notes |
|---|---:|---:|---:|---|
| 100 rollback checkpoints, bytes | TBD | TBD | TBD | benchmark P7.2 |
| `SaveFile` JSON, 100 checkpoints | TBD | TBD | TBD | benchmark P7.2 |
| Player idle RSS | TBD | TBD | TBD | Linux release build |
| Fixture loaded RSS | TBD | TBD | TBD | Linux release build |

## Dependency Order

```text
P1 ─→ P2 ─→ P3 ─→ P4 ─→ P5 ─→ P6
                  └────────────→ P7 ─→ P8
P5 + P6 + P7 + P8 ─────────────→ P9 ─→ P10
```

Only parallelize tasks that do not write the same files.

---

## P0 — Decisions and terminology `[x]`

- [x] Define Run, Smoke, Vinyl CLI, Start Label, Default Player UI, Save Slot, Fade, Dissolve in `CONTEXT.md`.
- [x] Record one-binary distribution in `docs/adr/0001-ship-cli-and-player-as-one-binary.md`.
- [x] Record rollback persistence in `docs/adr/0002-save-files-preserve-rollback-history.md`.
- [x] Record preferences outside save slots in `docs/adr/0003-player-preferences-live-outside-save-slots.md`.

**Gate:** terminology and irreversible decisions are documented.

## P1 — Correct story entry point `[x]`

### P1.1 Validation

- [x] Require exactly one `label start` across all script files.
- [x] Emit source-aware diagnostic when missing or duplicated.
- [x] Keep existing missing-label and asset diagnostics aggregated.

### P1.2 Compiler/VM

- [x] Resolve `Program.labels["start"]` to an `OpId`.
- [x] Construct VM at that index instead of implicit `pc = 0`.
- [x] Ensure restored saves retain their saved `pc`.
- [x] Keep jumps and menu targets index-based.

### P1.3 Tests

- [x] Missing `start` fails validation.
- [x] Duplicate `start` fails validation.
- [x] Multi-file project begins at `start` regardless of file discovery order.
- [x] Smoke and player use the same entry-point path.

**Gate:**

```bash
cargo test -p vn_core
cargo test -p vn_script
```

## P2 — Public `vn` CLI and `smoke` `[x]`

### P2.1 Binary and commands

- [x] Add `[[bin]] name = "vn"` while package remains `vn_cli`.
- [x] Rename current headless `run()` implementation to `smoke()`.
- [x] Add real `run()` command boundary; initially calls the P3 player bootstrap.
- [x] Default every project path argument to `.`.
- [x] Keep `--locale`; default to manifest `default_locale`.
- [x] Make `vn --help` distinguish rendered Run from headless Smoke.
- [x] Do not ship a `vn_cli` compatibility executable.

### P2.2 Behavior

- [x] `vn run` validates before creating a window.
- [x] `vn smoke` starts at `label start`, chooses first menu option, verifies save/load/rollback, prints events, exits.
- [x] Invalid locale or project exits non-zero with full diagnostics.

### P2.3 Tests

- [x] `vn --help` snapshot/contains assertions.
- [x] Commands work with omitted project path from fixture working directory.
- [x] `vn smoke` preserves deterministic event output.
- [x] `vn run` rejects invalid project before player startup.

**Gate:**

```bash
cargo test -p vn_cli
```

## P3 — Desktop Bevy bootstrap `[x]`

### P3.1 Features

- [x] Enable Bevy window/winit, X11, 2D renderer, UI renderer, text, and PNG features needed by the distributed binary. Audio/MP3 remains independently opt-in because ALSA development files are unavailable locally.
- [x] Keep headless library tests able to create `App::new()` without a window.
- [x] Make `vn_cli` depend on the desktop player entry point from `vn_bevy`.

### P3.2 Reusable player API

- [x] Add one `vn_bevy` player entry function/config type accepting loaded root/manifest, program, translations, project identity, script hash, and engine version.
- [x] Keep CLI parsing and diagnostics out of `vn_bevy`.
- [x] Use an enum for player mode: boot, loading, playing, paused, save, load, settings, runtime error, ended.
- [x] Store dense runtime flags out-of-band in one compact flags value.

### P3.3 Window

- [x] Open resizable 1280×720 window.
- [x] Preserve 16:9 logical composition with letterboxing.
- [x] Spawn camera and visible loading surface.
- [x] Auto-start after initial content is ready; no branded splash.
- [x] At `end`, retain final frame and disable story advance.

**Gate:** manually and automatically confirm a visible window opens and remains responsive.

## P4 — Asset loading and memory layout `[x]`

### P4.1 Paths

- [x] Separate filesystem validation paths from Bevy asset paths.
- [x] Configure project/asset root once; never produce duplicated `assets/assets/...` paths.
- [x] Resolve background to PNG, sprite to PNG, music to MP3 according to manifest.

### P4.2 Loading

- [x] Load on demand and wait before committing its presentation event.
- [x] Show loading overlay only after roughly 150 ms.
- [x] Reuse Bevy's asset cache; do not build a second cache.
- [x] Missing-after-validation or corrupt PNG/MP3 enters runtime error mode with path and cause.
- [x] Corrupt MP3 is preflight-decoded before Bevy playback to avoid Bevy 0.17's decoder panic.
- [x] Missing audio device logs warning and continues silently.

### P4.3 Bundled font

- [x] Add one licensed Latin font covering English, Indonesian, and common European accents.
- [x] Bundle/load it independently of system fonts.
- [x] Record license beside the asset.

### P4.4 Asset tests

- [x] Real fixture PNG reaches loaded state and has non-zero dimensions.
- [x] Real fixture MP3 reaches decoded source state.
- [x] Corrupt PNG and MP3 use explicit runtime-error paths.
- [x] Project-relative path test prevents duplicated asset roots.

**Gate:** asset decode tests pass without relying on a physical audio device.

## P5 — Rendered story UI and input `[x]`

### P5.1 Visual composition

- [x] Background scale-to-cover and center-crop logical canvas.
- [x] Sprite preserves aspect ratio, shrinks above 90% viewport height, anchors left/center/right, and stands above dialogue area.
- [x] Replace default/empty image handles with explicit loading/error states.

### P5.2 Dialogue

- [x] Render bottom dialogue panel, optional speaker name, wrapped body text.
- [x] Implement typewriter using configured chars/second.
- [x] Skip-first: first advance completes transition/typewriter; next advance continues story.
- [x] Space, Enter, and left-click advance dialogue.

### P5.3 Menu

- [x] Render real choice buttons.
- [x] Mouse hover/click.
- [x] Up/down focus, Enter select, keys 1–9 shortcuts.
- [x] Consume UI clicks so they cannot also advance the story.
- [x] Lock menu during loading/transition.

### P5.4 Auto-start/driving

- [x] Drain scene/show/hide/music events until dialogue/menu/end.
- [x] Never show a ready blank frame between boot and first interaction.
- [x] Keep presentation snapshot and visible entities synchronized.

**Gate:** fixture opens with real background, sprite, dialogue, music, and clickable/keyboard menu.

## P6 — Correct transitions `[x]`

### P6.1 Fade

- [x] Old visual fades to blank.
- [x] New visual fades in after blank boundary.
- [x] Keep old visual alive until its phase completes.

### P6.2 Dissolve

- [x] Load incoming visual before transition starts.
- [x] Crossfade old and new concurrently.
- [x] Sprite replacement keeps outgoing entity until crossfade completes.

### P6.3 Input and tests

- [x] First advance completes active transition immediately.
- [x] Next advance continues story.
- [x] Test midpoint and completion alpha/lifecycle for fade and dissolve separately.

**Gate:** fade and dissolve cannot pass using the same lifecycle assertions.

## P7 — Save/load/rollback `[ ]`

### P7.1 Save schema

- [ ] Increment `CURRENT_SAVE_VERSION`.
- [ ] Introduce public serializable `RollbackCheckpoint`.
- [ ] Add `soa-rs = { version = "1", features = ["serde"] }` to `vn_core`.
- [ ] Derive `Soars` for `RollbackCheckpoint`; use `Soa<RollbackCheckpoint>` for VM rollback storage and serialized saves.
- [ ] Cap rollback storage at 100 checkpoints by dropping the oldest.
- [ ] Restore rollback SoA in `Vm::from_parts`/replacement constructor.
- [ ] Remove `Preferences` from `SaveFile`.
- [ ] Active music state restores track identity, restarting playback from zero.

### P7.2 Memory proof

- [ ] Benchmark 100 representative checkpoints using old AoS tuple vector and new `Soa`.
- [ ] Record allocation/size results in the Memory Measurement table.
- [ ] Verify field-slice access is used where only VM states or presentations are needed.
- [ ] Do not convert save slots, UI nodes, VM ops, or Bevy entities to `Soa` without separate evidence.

### P7.3 Storage

- [ ] Resolve per-project OS data directory.
- [ ] Store `autosave.json` and `slot-01.json` through `slot-12.json`.
- [ ] Write temporary file, flush, then atomic rename.
- [ ] Treat incompatible files as visible but unloadable; allow overwrite.

### P7.4 Screenshot and slot metadata

- [ ] Capture scene without pause/save overlay.
- [ ] Resize thumbnail to 320×180 PNG.
- [ ] Show slot number, thumbnail, local timestamp, speaker, dialogue excerpt, compatibility state.
- [ ] Confirm overwrite of occupied compatible slot.
- [ ] Load compatible slot directly.

### P7.5 Autosave and rollback UI

- [ ] Autosave after stable dialogue/menu interaction boundary.
- [ ] Never autosave while loading, transitioning, paused, or errored.
- [ ] PageUp and mouse-wheel-up rollback one boundary.
- [ ] Pause menu exposes Rollback action.
- [ ] Rollback restores visual/audio/dialogue/menu state together.

### P7.6 Tests

- [ ] Save round-trip restores VM, presentation, and all rollback checkpoints.
- [ ] 101st checkpoint evicts first.
- [ ] Incompatible saves remain listed but cannot load.
- [ ] Interrupted/temp write does not destroy previous slot.
- [ ] Screenshot excludes overlays.

**Gate:** save/load/rollback integration suite passes and memory measurements are recorded.

## P8 — Pause and settings `[ ]`

### P8.1 Pause overlay

- [ ] Escape toggles pause/resume.
- [ ] Actions: Resume, Save, Load, Settings, Rollback, Quit.
- [ ] F5 opens Save; F9 opens Load.
- [ ] Quit confirms only when progress is newer than last successful autosave.

### P8.2 Preferences

- [ ] Persist per project outside save slots: text speed, auto-advance, music volume, mute, fullscreen.
- [ ] Text speed choices: Slow, Normal, Fast, Instant.
- [ ] One Music Volume control plus Mute; no speculative SFX/voice mixer.
- [ ] Alt+Enter toggles fullscreen and persists it.
- [ ] Loading a slot never changes preferences.

### P8.3 Auto-advance

- [ ] Start timer after typewriter completes.
- [ ] Use minimum 1.5 seconds plus simple length-based reading time.
- [ ] Manual input remains available.
- [ ] Stop on menu, pause, loading, transition, error, or end.

**Gate:** keyboard/mouse interaction tests cover pause, settings persistence, and auto-advance stops.

## P9 — End-to-end visual CI `[ ]`

### P9.1 Linux deterministic environment

- [ ] Run window under virtual X display with software rendering.
- [ ] Fix window/logical resolution at 1280×720.
- [ ] Use bundled font and fixture assets.
- [ ] Wait for stable interaction frame, then capture screenshot.
- [ ] Compare against golden image with documented small pixel tolerance.

### P9.2 Assertions

- [ ] Background pixels visible.
- [ ] Character sprite visible.
- [ ] Dialogue speaker/text visible.
- [ ] Menu choices visible.
- [ ] MP3 decoded and playback entity active even if no device exists.
- [ ] Input selects choice and next frame changes.
- [ ] Save overlay creates a loadable slot with screenshot.

### P9.3 Other platforms

- [ ] Windows build/startup/decode smoke.
- [ ] macOS build/startup/decode smoke.
- [ ] No cross-platform golden comparison.

**Gate:** Linux golden E2E passes in CI and platform release builds start successfully.

## P10 — Packaging and documentation `[ ]`

### P10.1 Release

- [ ] Build `target/release/vn`.
- [ ] Publish `vn-linux-x86_64`, `vn-macos-aarch64`, `vn-windows-x86_64.exe`.
- [ ] Install `/usr/bin/vn` in DEB/RPM.
- [ ] Declare Linux X11/audio runtime dependencies.
- [ ] Verify XWayland launch path.

### P10.2 Docs

- [ ] Replace public `vn_cli` command references with `vn`.
- [ ] Document `run` versus `smoke` versus `check`.
- [ ] Document controls, save location, slots/autosave, rollback, settings, PNG/MP3 support, required `label start`, and desktop-only status.
- [ ] State that Default Player UI labels are English.
- [ ] Update examples, quickstart, install page, release instructions, and project template.

### P10.3 Final quality gate

- [ ] Formatting.
- [ ] Clippy, workspace/all targets/no default features where applicable.
- [ ] Full workspace tests.
- [ ] Linux visual E2E.
- [ ] Three-platform release build.

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --no-fail-fast --no-default-features
```

## Scope Deferred

- Browser/WASM player.
- Native Wayland backend requirement.
- JPEG/WebP/OGG/WAV support.
- Theme/custom layout system.
- Custom project fonts and CJK bundled fonts.
- Dialogue backlog screen.
- SFX/voice channels and mixer.
- Unlimited save slots/history.
- Exact audio playback-position restoration.

## Execution Steps

1. Complete P1 and mark its tests/gate.
2. Complete P2; preserve old runtime behavior under `vn smoke`.
3. Complete P3–P4; prove a real window and real asset decode before UI work.
4. Complete P5–P6; prove the fixture is playable and transitions are semantically distinct.
5. Complete P7; integrate `soa-rs`, persistence, slot UI, autosave, and rollback measurements.
6. Complete P8; add pause/settings without changing save-slot story state.
7. Complete P9; make rendered pixels—not ECS markers—the regression boundary.
8. Complete P10; rename artifacts/docs and pass all release gates.
