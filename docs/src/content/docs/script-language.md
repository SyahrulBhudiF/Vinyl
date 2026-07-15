---
title: Script Language Reference
---

Vinyl scripts are UTF-8, indentation-based `.vn` files. Blocks use four spaces. Comments begin with `#` outside quoted strings.

## Entry point

Every project must contain exactly one `label start`:

```vn
label start:
    "The story begins."
    end
```

Other labels can live in any loaded `.vn` file, including nested script directories. Jumps resolve across file boundaries after the project is combined and validated.

## Dialogue and narration

```vn
eileen "Hello."
"Narration has no speaker."
```

Optional stable text IDs appear before the source text:

```vn
eileen [chapter1-hello] "Hello."
[chapter1-narration] "A quiet morning."
```

Strings support `\n`, `\"`, and `\\` escapes.

### Text effects

```vn
eileen "Slow reveal." with typewriter(speed=30)
eileen "Show immediately." with instant
```

`speed` is characters per second. Typewriter defaults to 30 when omitted.

## Labels and jumps

```vn
label start:
    jump garden

label garden:
    "You enter the garden."
    end
```

All jump targets are validated before the player opens.

## Backgrounds

```vn
scene bg room
scene bg sunset with fade(duration=0.5)
```

A scene replaces the background and clears visible sprites. `duration` is measured in seconds.

## Character sprites

```vn
show eileen happy at left
show eileen surprised at center with dissolve(duration=0.4)
show eileen sad at right
hide eileen
```

Supported rendered positions are `left`, `center`, and `right`. Unknown positions currently render at center. Attributes map to a sprite filename; see [Assets](/Vinyl/assets/).

## Music

```vn
play music "theme.mp3"
stop music
```

Music paths are relative to the configured audio directory. MP3 is the supported MVP audio format.

## Menus

```vn
menu:
    [ask-name] "Ask her name":
        eileen "I'm Eileen."
        jump continue_story
    [leave] "Leave":
        end
```

A menu requires at least one choice, and every choice requires a non-empty indented body.

### Conditional choices

```vn
menu:
    "Open the locked door" if has_key == true:
        jump vault
    "Walk away":
        end
```

Only choices whose conditions evaluate to `true` are shown.

## Variables

Variable names begin with a letter or underscore and contain ASCII letters, digits, or underscores.

```vn
$affection = 3
$affection += 1
$affection -= 1
$has_key = true
$name = "Eileen"
```

Supported values are integers, booleans, and strings. `+=` and `-=` require integers.

## Conditions

```vn
if affection >= 3 and has_key == true:
    "The door opens."
else:
    "Nothing happens."
```

Supported operators, from lower to higher precedence:

| Group | Operators |
|---|---|
| Boolean OR | `or` |
| Boolean AND | `and` |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=` |
| Arithmetic | `+`, `-` |
| Unary | `not` |

Expressions are whitespace-tokenized. Write `affection >= 3`, not `affection>=3`. Parentheses are not currently supported.

## Ending the story

```vn
end
```

`end` enters the player's ended state. It does not jump to a label named `end`; use `jump end_label` when a shared ending block is needed.

## Complete example

```vn
label start:
    scene bg room with fade(duration=0.5)
    play music "theme.mp3"
    show eileen happy at center with dissolve(duration=0.4)
    $affection = 3
    eileen [intro-hello] "Hello." with typewriter(speed=45)

    menu:
        [ask-name] "Ask her name" if affection >= 3:
            eileen [name-answer] "I'm Eileen."
            jump ending
        [leave] "Leave":
            jump ending

label ending:
    stop music
    end
```

For a runnable project using variables, conditions, conditional choices, multiple endings, assets, and two locales, see the [Complete Example Game](/Vinyl/example-game/).
