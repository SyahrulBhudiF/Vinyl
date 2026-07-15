---
title: Troubleshooting
---

## `vn` is not found

Confirm the binary directory is on `PATH`, then open a new terminal:

```bash
vn --help
```

Windows users can run `C:\Tools\Vinyl\vn.exe --help` directly to distinguish a PATH problem from a binary problem.

## The project does not start

Run validation first:

```bash
vn check .
```

Common causes:

- missing or duplicate `label start`
- wrong indentation
- missing jump target
- missing PNG or MP3 file
- duplicate text ID
- locale entry missing for a selected locale

`vn run` performs the same validation before opening a window.

## Linux window does not open

Vinyl currently requires X11 or XWayland. In a native Wayland session, ensure XWayland is installed and enabled. Also install the runtime packages listed on the [Install](/Vinyl/install/) page.

## No music plays

Check the resolved path:

```bash
vn list-assets .
```

Music must be a valid MP3. Missing audio hardware or an unavailable output device produces a warning and the game continues silently.

## Runtime error overlay

A corrupt or missing image/audio asset can enter Runtime Error mode after startup. The overlay and terminal output include the asset path and cause. Correct the file, run `vn check .`, and restart the player.

## A save cannot load

The slot may be incompatible because its save schema, project ID, project version, or script hash differs from the current game. Incompatible and corrupt slots remain visible but are intentionally blocked from loading. They can be overwritten.

## Text is missing glyphs

The bundled player font covers Latin text. CJK and custom project fonts are not currently supported. The locale system can load translated strings, but rendering still depends on available bundled glyphs.

## macOS blocks the binary

Open **System Settings → Privacy & Security** and allow the downloaded executable. The current release artifact targets Apple Silicon.

## Visual test fails locally

Use the CI-equivalent environment:

```bash
export WGPU_BACKEND=vulkan
export LIBGL_ALWAYS_SOFTWARE=1
xvfb-run -a -s '-screen 0 1280x720x24' python3 scripts/visual-ci.py
```

Do not regenerate golden images under a tiled desktop compositor; the expected capture size is exactly 1280×720.
