---
title: Complete Example Game
---

The repository includes **The Locked Room**, a runnable multi-file project combining the main Vinyl authoring features in one small story.

```text
examples/branching-mystery/
├── vinyl.toml
├── script/
│   ├── start.vn
│   ├── endings.vn
│   └── chapters/
│       └── cabinet.vn
├── locale/
│   ├── en-US.ftl
│   └── id-ID.ftl
└── assets/
    ├── bg/room.png
    ├── sprites/eileen/happy.png
    └── audio/theme.mp3
```

Vinyl loads every `.vn` file recursively under the configured script directory. Files may jump to labels declared in another file; execution still begins at the project's single `label start`.

The example demonstrates:

- cross-file labels and jumps;
- nested script directories;
- dialogue, narration, and text IDs;
- integer and boolean variables;
- assignment with `=` and `+=`;
- `if`/`else` branches;
- conditional menu choices;
- multiple endings;
- fade, dissolve, and typewriter effects;
- background, sprite, and music assets;
- English and Indonesian locale files.

The CLI integration suite validates and smoke-tests the complete project in both bundled locales.

## Run the example

From the repository root:

```bash
cargo run -p vn_cli --features audio -- check examples/branching-mystery
cargo run -p vn_cli --features audio -- smoke examples/branching-mystery
cargo run -p vn_cli --features audio -- run examples/branching-mystery
```

Run the Indonesian version:

```bash
cargo run -p vn_cli --features audio -- run \
  examples/branching-mystery --locale id-ID
```

If `vn` is already installed, replace `cargo run -p vn_cli --features audio --` with `vn`.

## `script/start.vn`

```vn
label start:
    scene bg room with fade(duration=0.5)
    show eileen happy at center with dissolve(duration=0.4)
    play music "theme.mp3"
    $affection = 1
    $has_key = false
    eileen [intro-welcome] "You made it before the storm." with typewriter(speed=38)

    menu:
        [intro-search] "Search the room":
            $has_key = true
            $affection += 2
            eileen [intro-key] "That key opens the cabinet."
            jump cabinet
        [intro-ask] "Ask Eileen for help":
            $affection += 1
            eileen [intro-trust] "I was hoping you would trust me."
            jump cabinet
        [intro-leave] "Leave before the storm":
            jump early_exit
```

The `cabinet` and `early_exit` labels are declared in other files.

## `script/chapters/cabinet.vn`

```vn
label cabinet:
    if has_key == true and affection >= 3:
        eileen [cabinet-ready] "We have the key, and I know the combination."
    else:
        eileen [cabinet-closed] "The cabinet is still locked."

    menu:
        [cabinet-open] "Open the cabinet" if has_key == true:
            "Inside is a letter addressed to both of you."
            jump ending
        [cabinet-listen] "Listen to Eileen":
            $affection += 2
            eileen [cabinet-story] "My family hid the letter here years ago."
            jump ending
        [cabinet-back] "Step away":
            jump early_exit
```

## `script/endings.vn`

```vn
label ending:
    if affection >= 3:
        hide eileen
        "You leave with the truth and her trust."
    else:
        "You found the truth, but not her trust."
    stop music
    end

label early_exit:
    hide eileen
    stop music
    "You leave the mystery for another night."
    end
```

## What the tests verify

The parser and project-loader tests cover:

- deeply nested `if` and `menu` blocks;
- cross-file jumps through nested directories;
- stable loading and hashing of 128 script files;
- project execution beginning at `label start`, independent of file order.

A separate ignored release-mode probe measures recursive loading, hashing, parsing, validation, and compilation for 128 files and 4,352 statements. See [Performance](/Vinyl/performance/) for the command, observed results, and limitations.

`vn smoke` additionally advances to the first menu, serializes and restores story state, chooses the first available option, and checks rollback to the menu. The desktop player remains available for manually exploring every branch.
