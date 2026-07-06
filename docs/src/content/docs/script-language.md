---
title: Script Language
---

Vinyl scripts are indentation-based and writer-friendly.

```vn
label start:
    scene bg room with fade(duration=0.4)
    show eileen happy at center with dissolve(duration=0.25)
    eileen [intro.hello] "Hello." with typewriter(speed=35)
    $affection += 1
    menu:
        [intro.ask] "Ask" if affection >= 3:
            jump ask
        [intro.leave] "Leave":
            end
```

Core statements: `label`, `scene`, `show`, `hide`, dialogue, `menu`, `jump`, `$var = value`, `if`/`else`, `end`.
