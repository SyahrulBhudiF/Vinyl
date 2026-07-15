---
title: Vinyl
---

Vinyl is a desktop visual-novel player and writer tool. Games are projects containing a manifest, indentation-based `.vn` scripts, PNG images, MP3 music, and optional Fluent translations.

## Start here

1. [Install Vinyl](/Vinyl/install/) for Linux, macOS, or Windows.
2. [Create your first game](/Vinyl/quickstart/).
3. Learn the [script language](/Vinyl/script-language/).
4. Explore the tested [complete example game](/Vinyl/example-game/).
5. Add [assets](/Vinyl/assets/) and [localization](/Vinyl/localization/).

## Player features

- 1280×720 logical canvas with resizable 16:9 presentation.
- Dialogue, choices, typewriter text, fade, and dissolve transitions.
- PNG backgrounds and sprites; MP3 music.
- Pause, text speed, auto-advance, volume, mute, and fullscreen settings.
- Twelve manual save slots, one autosave, screenshots, and up to 100 rollback checkpoints.
- Deterministic headless smoke testing and Linux visual regression testing.

## Documentation

### Game authoring

- [Quickstart](/Vinyl/quickstart/)
- [Project layout and manifest](/Vinyl/project-layout/)
- [Script language reference](/Vinyl/script-language/)
- [Complete example game](/Vinyl/example-game/)
- [Assets](/Vinyl/assets/)
- [Localization](/Vinyl/localization/)
- [Player controls and settings](/Vinyl/player/)
- [CLI reference](/Vinyl/cli/)
- [Troubleshooting](/Vinyl/troubleshooting/)

### Engine development

- [Architecture](/Vinyl/architecture/)
- [Save and rollback model](/Vinyl/saves/)
- [Testing and release process](/Vinyl/development/)
- [Performance](/Vinyl/performance/)

## Current platform scope

Release targets are Linux x86-64 through X11/XWayland, macOS Apple Silicon, and Windows x86-64. Native Wayland and web builds are not currently supported. The default player interface is English; game content can use any configured locale supported by its bundled text and fonts.
