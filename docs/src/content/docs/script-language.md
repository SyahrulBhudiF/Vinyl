---
title: Script Language Reference
---

Vinyl stories are written in UTF-8 `.vn` text files. The language is line-based and indentation-based: each line is one statement, and indentation defines which statements belong to a label, condition, or menu choice.

This page describes the complete MVP syntax, how statements behave, and common mistakes.

## Essential rules

1. Use **four spaces** for every indentation level. Do not use tabs.
2. Every project needs exactly one `label start:`. This is always the entry point.
3. Statements inside a block must be indented one level below its header.
4. Names are case-sensitive: `garden`, `Garden`, and `GARDEN` are different names.
5. Expressions require spaces around operators: use `score >= 3`, not `score>=3`.
6. Comments begin with `#` outside strings.
7. Run `vn check .` after editing. Vinyl reports the source file, line, column, and problem.

Smallest valid script:

```vn
label start:
    "Hello, world."
    end
```

## File and block structure

A project may contain one or many `.vn` files under its configured script directory. Vinyl loads them recursively, sorts them by path, and combines them into one story before validating labels and jumps.

```text
script/
├── start.vn
├── endings.vn
└── chapters/
    └── chapter_one.vn
```

Labels and jumps may cross file boundaries:

```vn
# script/start.vn
label start:
    jump chapter_one
```

```vn
# script/chapters/chapter_one.vn
label chapter_one:
    "Chapter one begins."
    jump good_ending
```

```vn
# script/endings.vn
label good_ending:
    "The end."
    end
```

File order does not select the entry point. Execution always starts at `label start`.

### Indentation

Use exactly four additional spaces for nested content:

```vn
label start:
    if has_key == true:
        menu:
            "Open the door":
                "The door opens."
                end
    else:
        "You need a key."
        end
```

Blank lines are ignored. Tabs and inconsistent indentation are not supported.

### Comments

```vn
# Full-line comment
label start:
    $score = 1  # End-of-line comment
    "A # inside a string is text, not a comment."
    end
```

## Labels, flow, and endings

### `label`

A label names a destination in the story:

```vn
label garden:
    "You enter the garden."
```

Label names must begin with an ASCII letter or `_`, followed by ASCII letters, digits, or `_`.

Valid names:

```text
start
chapter_1
_good_ending
```

Invalid names include `chapter-one`, `1chapter`, and names containing spaces. Labels must be unique across every loaded `.vn` file.

### `jump`

`jump` immediately continues execution at another label:

```vn
jump garden
```

The target may be in the same file or another loaded file. `vn check` reports missing targets before opening the player.

### Normal fallthrough

Without `jump` or `end`, execution continues to the next statement in the combined story. Use explicit jumps at chapter and ending boundaries so behavior does not accidentally depend on file path order.

### `end`

`end` marks the story as finished:

```vn
label ending:
    "Thanks for playing."
    end
```

It is a command, not a label name. To share an ending, jump to a normal label:

```vn
jump common_ending

label common_ending:
    "The end."
    end
```

## Dialogue and narration

### Character dialogue

Place a speaker name before the quoted text:

```vn
eileen "Hello."
guard "The gate is closed."
```

The speaker name follows the same naming rules as labels. It is displayed as written; characters do not need a separate declaration.

### Narration

Omit the speaker:

```vn
"Rain strikes the window."
```

### Escapes

Quoted strings support:

| Escape | Result |
|---|---|
| `\n` | New line |
| `\"` | Literal double quote |
| `\\` | Literal backslash |

```vn
eileen "First line.\nSecond line."
"She says, \"Wait.\""
```

A physical script line cannot contain an unescaped closing quote. Use `\n` for multiline displayed text.

### Stable text IDs

An optional text ID may appear immediately before dialogue, narration, or menu text:

```vn
eileen [intro-hello] "Hello."
[intro-weather] "Rain strikes the window."
```

Text IDs connect source text to Fluent locale entries:

```ftl
intro-hello = Hello.
intro-weather = Rain strikes the window.
```

IDs must be non-empty and unique across all loaded scripts. Hyphenated Fluent-style IDs are recommended. Source text remains the runtime fallback when a translation is unavailable.

See [Localization](/Vinyl/localization/) for locale files and extraction commands.

## Text effects

Text effects follow dialogue or narration after `with`.

### Instant text

```vn
eileen "Shown immediately." with instant
```

`instant` is also the default when no effect is specified.

### Typewriter text

```vn
eileen "Revealed gradually." with typewriter
eileen "Revealed faster." with typewriter(speed=45)
```

`speed` is an integer number of characters per second. The default is `30`.

The first advance input while typewriter text is active completes the line. The next advance continues the story.

## Backgrounds and transitions

### `scene`

```vn
scene bg room
scene bg room with fade(duration=0.5)
```

`scene` replaces the current background and clears visible character sprites. With the default asset layout, `scene bg room` resolves to:

```text
assets/bg/room.png
```

Additional words create nested paths. For example, `scene bg school hallway` resolves to `assets/bg/school/hallway.png`.

Supported transitions:

```vn
scene bg room with fade(duration=0.5)
scene bg garden with dissolve(duration=1.0)
```

`duration` is measured in seconds. If omitted, the transition duration is zero, so specify it when visible motion is intended.

## Character sprites

### `show`

```vn
show eileen at center
show eileen happy at left
show eileen school happy at right
```

The first name is the character tag. Remaining names before `at` are sprite attributes.

With the default asset layout:

| Statement | Resolved file |
|---|---|
| `show eileen at center` | `assets/sprites/eileen/default.png` |
| `show eileen happy at center` | `assets/sprites/eileen/happy.png` |
| `show eileen school happy at center` | `assets/sprites/eileen/school_happy.png` |

Supported positions are `left`, `center`, and `right`. If `at` is omitted, the position defaults to `center`.

A transition may be added:

```vn
show eileen happy at center with dissolve(duration=0.4)
```

Showing the same character tag again replaces that character's visible sprite.

### `hide`

```vn
hide eileen
```

`hide` removes the visible sprite associated with that character tag.

See [Assets](/Vinyl/assets/) for formats, paths, sizing, and validation.

## Music

```vn
play music "theme.mp3"
play music "chapter1/storm.mp3"
stop music
```

Music paths are relative to the configured audio directory. MP3 is the supported MVP format. Starting another track replaces the current track. Music loops until `stop music`, another track starts, or the story ends.

Loading a save or rolling back restarts the active track from its beginning. A missing audio device produces a warning and silent playback rather than stopping the story.

## Variables and values

Variables store story state and persist in saves and rollback checkpoints. Assign a variable by prefixing its name with `$`:

```vn
$affection = 0
$has_key = false
$player_name = "Eileen"
```

The `$` prefix is used only on the left side of assignment. Refer to the variable without `$` in expressions:

```vn
$affection = 3
if affection >= 3:
    "She trusts you."
```

Variable names follow the same naming rules as labels.

Supported value types:

| Type | Examples |
|---|---|
| Integer | `0`, `12`, `-4` |
| Boolean | `true`, `false` |
| String | `"Eileen"`, `"chapter_one"` |

Variables are created by assignment; there is no separate declaration statement. Read a variable only after a path has assigned it, or the player reports an unknown-variable runtime error.

### Assignment operators

```vn
$score = 10
$score += 2
$score -= 1
```

- `=` replaces the value and may use any supported type.
- `+=` and `-=` perform integer arithmetic and require an existing integer variable.

Assignments may use expressions:

```vn
$score = score + 2
$remaining = total - used
```

## Expressions and conditions

Expressions are used by `if`, conditional menu choices, and assignments.

```vn
if affection >= 3 and has_key == true:
    "The door opens."
else:
    "Nothing happens."
```

### Operators and precedence

From lowest to highest precedence:

| Precedence | Operators | Result or requirement |
|---|---|---|
| 1 | `or` | Boolean operands, Boolean result |
| 2 | `and` | Boolean operands, Boolean result |
| 3 | `==`, `!=` | Compare values of the same intended type |
| 3 | `<`, `<=`, `>`, `>=` | Integer operands |
| 4 | `+`, `-` | Integer operands |
| 5 | `not` | Boolean operand |

Examples:

```vn
if has_key == true:
    "You have the key."

if score >= 10 and not game_over:
    "Bonus unlocked."

$total = base + bonus - penalty
```

Expressions are whitespace-tokenized. Put spaces around every binary operator:

```vn
# Correct
if affection >= 3 and has_key == true:

# Incorrect
if affection>=3 and has_key==true:
```

Parentheses are not supported. Rewrite complex logic as nested `if` blocks or intermediate Boolean variables:

```vn
$trusted = affection >= 3
if trusted == true:
    if has_key == true:
        "The door opens."
```

String literals in expressions currently need to be a single whitespace-free token, such as `"Eileen"` or `"chapter_one"`. Dialogue and menu strings may contain spaces normally.

Type errors are runtime errors. For example, `score + true`, `not 3`, or comparing an integer with `<` against a string is invalid.

## Conditional branches

```vn
if affection >= 3:
    "She trusts you."
else:
    "She remains cautious."
```

The condition must evaluate to a Boolean. `else:` is optional:

```vn
if has_key == true:
    "The key turns in the lock."
```

Nested branches are supported:

```vn
if has_key == true:
    if affection >= 3:
        jump best_ending
    else:
        jump neutral_ending
else:
    jump locked_ending
```

After the selected branch finishes, execution continues after the complete `if`/`else` block unless a `jump` or `end` changes the flow.

## Menus and choices

A menu pauses story execution and shows its visible choices:

```vn
menu:
    "Ask her name":
        eileen "I'm Eileen."
    "Leave":
        "You walk away."
        end
```

Rules:

- `menu:` ends with a colon.
- Each choice is indented four spaces beneath `menu:`.
- Each choice text ends with a colon after its optional condition.
- Every choice needs a non-empty body indented another four spaces.
- A menu needs at least one declared choice.

After a chosen body finishes, execution continues after the whole menu. A `jump` is only needed when the choice should go somewhere else.

### Choice text IDs

```vn
menu:
    [intro-ask] "Ask her name":
        eileen [intro-answer] "I'm Eileen."
```

Choice IDs follow the same uniqueness and localization rules as dialogue IDs.

### Conditional choices

Add `if <expression>` after the closing quote and before the colon:

```vn
menu:
    "Open the locked door" if has_key == true:
        jump vault
    "Ask for help" if affection >= 2:
        jump help
    "Walk away":
        end
```

Only choices whose conditions evaluate to `true` are shown. Conditions are evaluated when the menu appears. Ensure every reachable menu has at least one visible choice; a menu whose conditions all evaluate to `false` cannot be selected.

A click that reveals a menu cannot select a choice in the same physical mouse press.

## Statement summary

| Statement | Purpose |
|---|---|
| `label name:` | Define a story destination |
| `jump name` | Continue at a label |
| `"Text"` | Show narration |
| `speaker "Text"` | Show character dialogue |
| `scene bg path` | Replace background and clear sprites |
| `show tag attrs at position` | Show or replace a character sprite |
| `hide tag` | Hide a character sprite |
| `play music "file.mp3"` | Start looping music |
| `stop music` | Stop music |
| `$name = expression` | Set a variable |
| `$name += expression` | Add integers |
| `$name -= expression` | Subtract integers |
| `if condition:` / `else:` | Choose a conditional branch |
| `menu:` | Present player choices |
| `end` | Finish the story |

## Common errors

### Wrong indentation

```vn
# Wrong: dialogue is not inside the label
label start:
"Hello."
```

```vn
# Correct
label start:
    "Hello."
```

### Missing colon

```vn
# Wrong
label start
if has_key == true

# Correct
label start:
if has_key == true:
```

### Missing spaces in expressions

```vn
# Wrong
if score>=3:

# Correct
if score >= 3:
```

### Using `$` when reading a variable

```vn
# Wrong
if $has_key == true:

# Correct
if has_key == true:
```

### Empty menu choice

```vn
# Wrong
menu:
    "Continue":
```

```vn
# Correct
menu:
    "Continue":
        jump next_scene
```

### Duplicate project-wide names

Labels and text IDs are global across every loaded script file. Do not reuse them in another chapter.

## Validation workflow

Check the project before running it:

```bash
vn check .
```

Validation reports syntax errors, duplicate or missing labels, duplicate text IDs, missing locale entries, and missing referenced assets. The desktop window is not opened when validation fails.

Useful debugging commands:

```bash
vn dump-ast .
vn dump-ir .
vn extract-locales .
vn list-assets .
vn smoke .
```

- `dump-ast` shows parsed statements.
- `dump-ir` shows compiled runtime operations.
- `extract-locales` prints Fluent entries for text IDs.
- `list-assets` shows resolved asset paths.
- `smoke` performs deterministic headless story verification.

## Complete example

```vn
label start:
    scene bg room with fade(duration=0.5)
    show eileen happy at center with dissolve(duration=0.4)
    play music "theme.mp3"
    $affection = 1
    $has_key = false
    eileen [intro-hello] "You made it." with typewriter(speed=40)

    menu:
        [intro-search] "Search the room":
            $has_key = true
            $affection += 2
            jump cabinet
        [intro-leave] "Leave":
            jump early_exit

label cabinet:
    if has_key == true and affection >= 3:
        eileen [cabinet-open] "The cabinet is open."
        jump good_ending
    else:
        eileen [cabinet-locked] "It is still locked."
        jump early_exit

label good_ending:
    hide eileen
    stop music
    "You found the truth."
    end

label early_exit:
    stop music
    "You leave the mystery behind."
    end
```

For a runnable multi-file project with assets and English/Indonesian localization, see the [Complete Example Game](/Vinyl/example-game/).
