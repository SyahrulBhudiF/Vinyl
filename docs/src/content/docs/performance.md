---
title: Performance
---

Vinyl separates story execution from desktop presentation. The deterministic VM advances when the player reaches a dialogue line, choice, visual command, audio command, or ending. Bevy handles the continuously rendered window, transitions, text reveal, input, and audio.

## What affects a game

Different parts of a project affect different resources:

- Script size and expression count affect parsing, validation, compilation, and VM work.
- Dialogue history, variables, visible sprites, and rollback checkpoints affect save size and memory use.
- Image dimensions and the number of visible sprites affect texture memory and rendering.
- Audio length and encoding affect file size and decoding.
- GPU driver, window size, and rendering backend affect desktop frame time.

A headless story check cannot predict rendering or asset-loading performance. Measure those separately when they matter to a project.

## Runtime behavior

- Assets are requested when presentation commands need them; the full project is not preloaded.
- Pending assets retain strong Bevy handles until loading completes.
- A loading overlay appears after about 150 ms, avoiding a flash for quick loads.
- PNG backgrounds and sprites preserve their intended composition on the 1280×720 logical canvas.
- MP3 files are preflight-decoded before playback so corrupt audio becomes a runtime error with a useful path and cause.
- Transitions and typewriter effects are timer-driven.
- Story and menu input remain locked while loading or an active effect must finish.

## Rollback and saves

Vinyl retains at most 100 interaction checkpoints. A checkpoint contains the VM and presentation state needed to restore the story. The checkpoint count is bounded, but each checkpoint can become larger as a game accumulates variables and dialogue history.

Rollback history uses `soa-rs`. The repository contains a layout regression test for the current checkpoint shape:

```bash
cargo test -p vn_core --test rollback_layout -- --nocapture
```

That test compares inline storage and serialized size with an equivalent Array-of-Structs representation. It describes its fixture only; it is not a general speed or memory benchmark.

## Multi-file parser stress test

The repository includes two parser checks for larger projects:

- an ordinary test that recursively loads, hashes, validates, compiles, and executes 128 `.vn` files;
- an ignored release-mode performance probe using 128 files, 4,352 parsed statements, and 4,224 compiled operations.

Run the performance probe with:

```bash
cargo test -p vn_script --release --test parser_performance -- \
  --ignored --nocapture
```

On an AMD Ryzen 7 5800H running Linux with Rust 1.93.1, five release-mode runs produced these observations:

| Stage | Median | Observed range |
|---|---:|---:|
| Recursive file loading, hashing, and parsing | 3.451 ms | 3.347–4.279 ms |
| Validation | 0.062 ms | 0.044–0.092 ms |
| Compilation | 1.249 ms | 1.119–1.476 ms |
| Combined stages | 4.794 ms | 4.512–5.830 ms |

The probe creates temporary files immediately before each measurement. Its loading stage therefore includes filesystem access, source hashing, and parsing. It does not measure desktop startup, asset decoding, rendering, audio, VM playback, or arbitrary user projects. Results are observations for this synthetic workload and machine, not performance guarantees.

The probe is ignored during normal test runs so CI correctness checks do not depend on timing. Its assertions still verify the expected file, statement, operation, and label counts.

## Measure a project

Use a release build when measuring normal command execution:

```bash
cargo build -p vn_cli --release --features audio
/usr/bin/time -v target/release/vn smoke examples/branching-mystery
```

`vn smoke` measures deterministic project loading, validation, compilation, VM execution, save serialization and restoration at the first menu, and rollback. It does not create a window or measure GPU and audio playback.

For Linux visual verification:

```bash
cargo build -p vn_cli --features audio
WGPU_BACKEND=vulkan LIBGL_ALWAYS_SOFTWARE=1 \
  xvfb-run -a -s '-screen 0 1280x720x24' python3 scripts/visual-ci.py
```

This checks deterministic captured output under Xvfb and software Vulkan. It is a correctness check, not a native-GPU frame-rate benchmark.

## Record useful results

When sharing measurements, include:

- Vinyl commit or release version;
- Rust version and build profile;
- enabled feature flags;
- operating system, CPU, memory, and GPU;
- exact project or fixture;
- exact command;
- number of runs and median or distribution.

Keep startup, VM execution, save serialization, resident memory, asset loading, and rendering measurements separate. A result is meaningful only for the workload and environment that produced it.
