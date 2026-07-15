---
title: Player Controls and Settings
---

## Window and presentation

Vinyl renders a 1280×720 logical canvas. The desktop window is resizable; composition remains 16:9 with letterboxing when the window has another aspect ratio. Alt+Enter toggles borderless fullscreen.

## Story controls

| Input | Action |
|---|---|
| Space, Enter, left click | Complete the current transition or typewriter effect; press again to advance. |
| Arrow keys | Move menu focus. |
| Number keys | Choose the corresponding visible menu option. |
| Left click on choice | Select that choice. |
| Page Up or mouse wheel up | Roll back one interaction checkpoint. |
| Escape | Pause, resume, or return from Save/Load/Settings. |
| F5 | Open Save. |
| F9 | Open Load. |
| Alt+Enter | Toggle borderless fullscreen. |

One physical mouse press performs at most one story action. A click that reveals a menu cannot also choose an option from that newly opened menu.

## Pause menu

The pause menu provides:

- Resume
- Save
- Load
- Settings
- Rollback
- Quit

Quit asks for a second confirmation only when progress is newer than the last successful autosave.

## Settings

### Text speed

- Slow: 15 characters per second
- Normal: 30 characters per second
- Fast: 60 characters per second
- Instant

The setting applies immediately to the current typewriter reveal.

### Auto-advance

Auto-advance waits for transitions and typewriter text to finish, then uses a minimum delay plus text-length reading time. It stops at choices, pause screens, loading, runtime errors, and the end of the story.

### Audio

Music volume changes in 10% increments. Mute and volume apply immediately. If the operating system has no audio output device, Vinyl continues silently.

### Fullscreen

Settings and Alt+Enter both control borderless fullscreen. The value persists per project.

## Player states

The player moves through Boot, Loading, Playing, Paused, Save, Load, Settings, Runtime Error, and Ended states. Story input is disabled while loading assets, showing overlays, or reporting runtime errors.
