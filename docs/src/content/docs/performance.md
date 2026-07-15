---
title: Performance and Measurement
---

This page records measurement methods and observed results. It does not claim a general performance level. Results vary with hardware, operating system, compiler, build profile, feature flags, story shape, assets, GPU driver, and window backend.

## Measurement scope

Vinyl uses different checks for different layers:

- `vn smoke` exercises deterministic, headless story execution.
- Rust tests cover parser, VM, rollback, persistence, runtime, and presentation behavior.
- Linux visual CI checks rendered output at 1280×720 under Xvfb and software Vulkan.
- Targeted release-mode probes are used when comparing implementation changes.

These are not interchangeable. Headless VM timing does not measure frame rate, startup, asset loading, audio, or GPU behavior. Linux software rendering does not predict native Windows or macOS rendering.

## Reproduce the checks

Headless fixture:

```bash
cargo build -p vn_cli --release --features audio
/usr/bin/time -v target/release/vn smoke fixtures/mvp
```

Linux visual check:

```bash
cargo build -p vn_cli --features audio
WGPU_BACKEND=vulkan LIBGL_ALWAYS_SOFTWARE=1 \
  xvfb-run -a -s '-screen 0 1280x720x24' python3 scripts/visual-ci.py
```

Rollback layout fixture:

```bash
cargo test -p vn_core --test rollback_layout -- --nocapture
```

When publishing a result, include the commit, Rust version, OS, CPU, build profile, feature flags, fixture, exact command, and run count.

## Results from the current refactor

A temporary release-mode probe executed 1,000 dialogue interactions while retaining the configured maximum of 100 rollback checkpoints. It was run five times on the same development machine against committed `663e843` and the current refactor.

| Revision | Median VM time | Five runs | Serialized rollback data |
|---|---:|---|---:|
| `663e843` | 28.699 ms | 28.396–28.711 ms | 3,710,951 bytes |
| Current refactor | 29.413 ms | 28.557–32.504 ms | 3,710,951 bytes |

The current median is about 2.5% higher, but the distributions overlap and one current run was an outlier. This does not establish a regression or improvement. The refactor primarily changes player structure and state bookkeeping rather than the VM or rollback representation.

For `vn smoke fixtures/mvp`, `/usr/bin/time` reported less than 0.01 seconds for both revisions. Maximum resident set size was 23,468 KiB for `663e843` and 24,112 KiB for the current tree on one run each. Timer resolution and one-sample memory results are insufficient for comparative claims.

The temporary VM probe was removed after measurement. The committed `rollback_layout` test remains reproducible, but it compares storage layout for one synthetic checkpoint shape; it is not a general speed benchmark.

## Rollback trade-off

The player retains at most 100 interaction checkpoints. Each checkpoint contains VM and presentation state needed for restoration. The count is bounded; checkpoint size still grows with variables and accumulated story history.

Rollback uses `soa-rs`. For the fixture in `rollback_layout`, Struct-of-Arrays and equivalent Array-of-Structs representations have equal inline container storage and serialized size. That result only describes this fixture. It does not prove that either representation is universally faster or smaller.

## Runtime behavior relevant to measurement

- Assets are requested on demand.
- The loading overlay appears after about 150 ms.
- MP3 files are preflight-decoded before Bevy playback.
- Transitions and typewriter effects are timer-driven.
- Focused desktop playback uses continuous Bevy updates.
- The logical canvas is 1280×720; actual render cost depends on output size and backend.

## Reporting policy

- Report measurements, commands, environment, and limitations together.
- Prefer medians and distributions over single runs.
- Compare identical fixtures and build settings.
- Keep VM, serialization, startup, memory, loading, and rendering measurements separate.
- Do not call a change faster or smaller unless the difference is repeatable and exceeds run-to-run variance.
