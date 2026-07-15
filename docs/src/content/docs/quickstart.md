---
title: Quickstart
---

## Create a project

```bash
vn new my-game
cd my-game
```

The generated project already contains `label start`, one dialogue line, one choice, and an English locale file.

## Write the first scene

Replace `script/start.vn` with:

```vn
label start:
    scene bg room with fade(duration=0.5)
    show eileen happy at center with dissolve(duration=0.4)
    eileen [intro-hello] "Welcome to my game." with typewriter(speed=35)

    menu:
        [intro-stay] "Stay":
            eileen [intro-stayed] "Let's begin."
            end
        [intro-leave] "Leave":
            "You leave the room."
            end
```

## Add assets

Add these files:

```text
assets/bg/room.png
assets/sprites/eileen/happy.png
```

PNG is the supported image format. Images do not need to be exactly 1280×720: backgrounds cover the logical canvas and sprites preserve their aspect ratio.

## Add translations

Update `locale/en-US.ftl`:

```ftl
intro-hello = Welcome to my game.
intro-stay = Stay
intro-stayed = Let's begin.
intro-leave = Leave
```

Text IDs are optional, but recommended for any project that may be translated.

## Validate and run

```bash
vn check .
vn run .
```

Validation catches missing labels, duplicate labels or text IDs, missing assets, missing locale entries, and syntax errors before a window opens.

## Useful next commands

```bash
vn list-assets .       # show resolved asset files
vn extract-locales .   # generate Fluent entries from text IDs
vn smoke .             # deterministic headless story check
```

Continue with [Project Layout](/Vinyl/project-layout/), the [Script Language Reference](/Vinyl/script-language/), or the tested [Complete Example Game](/Vinyl/example-game/).
