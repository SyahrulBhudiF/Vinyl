---
title: Architecture
---

Vinyl separates story semantics from rendering. The VM and save model do not depend on Bevy, allowing deterministic headless execution and renderer-independent integration.

## Workspace

```text
.vn files + vinyl.toml
          │
          ▼
      vn_script ── parse, load, validate, localize, resolve assets
          │ AST
          ▼
       vn_core ─── compile IR, execute VM, own save/rollback state
          │ VmEvent
          ▼
     vn_runtime ── convert events into presentation commands/snapshots
          │ PresentationCommand
          ▼
       vn_bevy ─── window, assets, rendering, input, audio, player UI
          │
          ▼
        vn_cli ─── public `vn` executable and command routing
```

## `vn_core`

Owns data and behavior that must remain deterministic and renderer-independent:

- AST and expressions
- compiled IR and label targets
- VM state and execution
- dialogue history
- presentation snapshots used by saves
- rollback checkpoints
- save schema and compatibility checks

The VM emits typed `VmEvent` values such as Dialogue, Scene, Show, Menu, and PlayMusic. It does not create windows, load textures, or play audio.

## `vn_script`

Owns author-facing project input:

- recursive `.vn` file loading in stable path order
- `vinyl.toml` parsing and defaults
- recoverable parsing with source diagnostics
- label, asset, and locale validation
- BLAKE3 script identity
- Fluent catalog loading and message extraction
- manifest-aware asset path resolution

Validation runs before desktop startup so authoring errors do not become renderer failures.

## `vn_runtime`

Bridges VM semantics to presentation without depending on Bevy:

- converts `VmEvent` into `PresentationCommand`
- applies commands to serializable `PresentationSnapshot`
- resolves renderer-independent asset paths
- reads and atomically writes saves and preferences
- classifies slots as empty, compatible, incompatible, or corrupt

## `vn_bevy`

Owns desktop presentation:

- Bevy plugin and ordered ECS systems
- 1280×720 camera and letterboxed window composition
- asynchronous asset loading and runtime-error UI
- PNG background/sprite materialization
- MP3 validation and playback
- dialogue, menu, pause, save/load, and settings UI
- transition and typewriter timers
- keyboard and mouse input rules
- screenshot capture and deterministic visual-test driver

`VnBevyPlugin` processes rollback/input, queued commands, deferred ECS changes, presentation entities, render entities, sizing, transitions, and text reveal in a fixed chain.

## `vn_cli`

Builds the public `vn` executable. It owns command-line parsing, project scaffolding, validation reporting, debug dumps, headless smoke execution, and desktop-player startup. The internal crate name is not part of the user-facing command.

## Story execution flow

1. `vn run` loads the manifest, scripts, and locale catalogs.
2. The validator checks entry point, labels, assets, and localized text IDs.
3. The compiler converts AST statements into indexed VM operations.
4. The VM starts at the compiled `start` label and emits one interaction or presentation event at a time.
5. Runtime maps each event into presentation commands.
6. Bevy waits for required assets, commits render state, and updates UI.
7. Input either completes the active effect or requests the next VM event.
8. Stable dialogue/menu boundaries become rollback checkpoints and autosave opportunities.

## State boundaries

| State | Owner |
|---|---|
| Script source and manifest | Project files |
| AST, IR, VM, variables, history | `vn_core` |
| Serializable visible story state | `PresentationSnapshot` |
| Command queue and asset loading | `vn_runtime` / `vn_bevy` resources |
| Render entities and UI nodes | Bevy ECS |
| Preferences and saves | OS data directory |

## Feature boundaries

- Default CLI feature: desktop player.
- `desktop`: Bevy UI, windowing, and X11 support.
- `audio`: desktop plus Bevy audio, MP3 decoding, and rodio preflight validation.
- No-default-feature builds retain parser, VM, runtime, tests, and non-rendering command behavior.
