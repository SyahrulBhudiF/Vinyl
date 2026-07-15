---
title: Install Vinyl
---

Vinyl ships as one desktop executable, `vn`, containing the player and writer tools.

## Install

Download the release for your OS and put it on your `PATH`:

- Linux: `vn-linux-x86_64`
- macOS: `vn-macos-aarch64`
- Windows: `vn-windows-x86_64.exe`

Linux requires X11 or XWayland plus the system ALSA, udev, X11, cursor, input, and RandR libraries. Native Wayland and web builds are not currently supported.

```bash
vn new my-game
cd my-game
vn check .
vn run .
```

## Main commands

- `vn check [project]`: parse and validate without opening a window.
- `vn run [project]`: validate, then launch the desktop player.
- `vn smoke [project]`: deterministic headless VM verification.
- `vn extract-locales [project]`: print Fluent entries for script text IDs.
- `vn list-assets [project]`: print referenced asset paths.

The project argument defaults to `.`. A project must contain exactly one `label start`.

## Player controls

- Space, Enter, or left click: complete the current effect, then advance.
- Arrow keys, number keys, or mouse: select menu choices.
- Page Up or mouse wheel up: rollback one interaction.
- Escape: pause or return from an overlay.
- F5 / F9: open Save / Load.
- Alt+Enter: toggle borderless fullscreen.

Each project has 12 manual slots and one autosave. Saves, rollback history, screenshots, and `preferences.json` live in the OS data directory for that project. Preferences are separate from save slots.

The MVP supports PNG backgrounds/sprites and MP3 music. If no audio device exists, playback is disabled with a warning and the story continues. Music restarts after load or rollback. Player UI labels are English; locale selection affects game script content only.

Rust is only needed when developing the engine itself.
