---
title: Performance
---

## Runtime profile

Vinyl is interaction-driven: the story VM performs small bursts of deterministic work when dialogue advances or a choice is selected, while Bevy handles continuous window rendering, transitions, text reveal, and audio playback.

## Rollback storage

The player retains at most 100 interaction checkpoints. Rollback state uses a Struct-of-Arrays container, confined to checkpoint history. A layout comparison of 100 representative checkpoints found equal inline storage and equal serialized size versus the equivalent Array-of-Structs representation:

| Measurement | AoS | SoA |
|---|---:|---:|
| Inline checkpoint storage | 27,200 bytes | 27,200 bytes |
| Container allocations | 1 | 1 |
| Serialized checkpoint data | 22,371 bytes | 22,371 bytes |

This measurement prevents unproven SoA conversions elsewhere. Save slots, VM operations, Bevy entities, and UI state remain in their natural structures.

## Current practical baseline

On the MVP fixture, the release `vn smoke` command completes below timer resolution on the development machine and peaks around 24 MB resident memory. The release desktop binary is approximately 84 MB before stripping because it includes the Bevy renderer, window backend, bundled font, image stack, and audio decoders.

These figures are local observations, not platform guarantees. GPU driver, window system, asset dimensions, story size, and build settings affect real results.

## Asset behavior

- Assets load on demand rather than preloading the whole project.
- A strong handle keeps each pending Bevy asset request alive.
- The loading overlay appears only after roughly 150 ms to avoid flashing during fast loads.
- Background and sprite textures preserve aspect ratio.
- Corrupt MP3 files are decoded once in preflight before Bevy playback.

## Rendering behavior

- The logical canvas remains 1280×720 regardless of window size.
- Transition and typewriter work is timer-based.
- Player updates run continuously while focused so animations do not wait for an external window event.
- Story and menu input are disabled during loading and active effects, avoiding duplicate work and inconsistent state.

## Optimization policy

Optimize only after a reproducible workload identifies a bottleneck. Keep renderer-independent data simple, prefer existing Bevy ECS storage for render state, and do not spread specialized layouts beyond the measured rollback use case without separate evidence.
