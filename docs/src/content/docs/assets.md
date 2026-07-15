---
title: Assets
---

## Supported formats

| Asset | Format |
|---|---|
| Background | PNG |
| Character sprite | PNG |
| Music | MP3 |
| Player font | Bundled Noto Sans Latin font |

CJK and custom game fonts, sound effects, and voice tracks are not currently part of the desktop MVP.

## Backgrounds

```vn
scene bg room
```

With the default manifest, this resolves to:

```text
assets/bg/room.png
```

Spaces create nested directories:

```vn
scene bg school hallway
```

```text
assets/bg/school/hallway.png
```

Backgrounds cover the 1280×720 logical canvas while preserving aspect ratio. Excess edges are center-cropped.

## Character sprites

```vn
show eileen happy at center
```

Resolves to:

```text
assets/sprites/eileen/happy.png
```

Multiple attributes are joined with underscores:

```vn
show eileen school happy at left
```

```text
assets/sprites/eileen/school_happy.png
```

A sprite without attributes uses `default.png`:

```vn
show eileen at center
```

```text
assets/sprites/eileen/default.png
```

Sprites preserve aspect ratio and are limited to 90% of the logical canvas height.

## Music

```vn
play music "theme.mp3"
```

Resolves to:

```text
assets/audio/theme.mp3
```

Nested paths are supported:

```vn
play music "bgm/chapter1.mp3"
```

Music loops during playback. Loading or rolling back restarts the active track from the beginning. If no audio device exists, Vinyl warns and continues silently. Corrupt MP3 files are rejected before playback and shown as runtime errors rather than crashing the player.

## Custom asset roots

```toml
[paths]
assets = "game-assets"

[assets]
backgrounds = "backgrounds"
sprites = "characters"
audio = "music"
```

All script references then resolve through those directories.

## Validation and inspection

```bash
vn check .
vn list-assets .
```

`vn check` rejects missing referenced files. `vn list-assets` prints the resolved filesystem paths, which is useful when a filename or manifest path is unclear.
