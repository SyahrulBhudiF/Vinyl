---
title: Saves, Autosave, and Rollback
---

## Slots

Each project has:

- one autosave
- twelve manual slots

Manual saves include local timestamp metadata and a 320×180 PNG screenshot. Saving over an occupied manual slot requires confirmation.

## Autosave

Vinyl autosaves at stable dialogue and menu boundaries. It does not save during an incomplete asset load, transition, or typewriter effect. Successful autosave state is also used to decide whether Quit needs confirmation.

## Rollback

Rollback restores one interaction boundary, including:

- VM program counter and variables
- visible choices
- dialogue history
- background and visible sprites
- active dialogue or menu
- active music identity

Up to 100 checkpoints are retained. Rollback history is serialized into saves, so loading a slot preserves its recent navigation history. Music restarts from the beginning after load or rollback.

## Save compatibility

A save can load only when these values match:

- save schema version
- project ID
- project version
- script hash

Incompatible and corrupt slots remain visible so the player can explain their state and allow overwrite, but they cannot be loaded.

## Atomic writes

Save and preference files are written to a temporary file, flushed, synchronized, and renamed. The previous slot is not replaced until serialization completes successfully.

## Storage locations

Files live under the operating system's per-user data directory:

| Platform | Base path |
|---|---|
| Linux | `$XDG_DATA_HOME/vinyl/<project-id>/saves`, or `~/.local/share/vinyl/<project-id>/saves` |
| macOS | `~/Library/Application Support/vinyl/<project-id>/saves` |
| Windows | `%APPDATA%\vinyl\<project-id>\saves` |

Files include:

```text
autosave.json
slot-01.json
...
slot-12.json
preferences.json
```

Preferences are stored beside save slots but outside the save data model. Loading a story slot never changes text speed, auto-advance, volume, mute, fullscreen, or locale preference.
