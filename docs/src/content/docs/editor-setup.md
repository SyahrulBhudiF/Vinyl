---
title: Editor Setup
---

Vinyl includes a small VS Code-compatible syntax extension for `.vn` scripts.

## VS Code highlighting

The extension lives in the repository:

```text
editors/vscode-vinyl/
```

Package and install it locally:

```bash
cd editors/vscode-vinyl
npx @vscode/vsce package
code --install-extension vinyl-vn-syntax-0.1.0.vsix
```

After installing, files ending in `.vn` are recognized as **Vinyl VN**.

## What gets highlighted?

```vn
label start:
    scene bg room with fade(duration=0.5)
    show eileen happy at center with dissolve(duration=0.4)
    $affection = 3
    eileen [intro-hello] "Hello." with typewriter(speed=45)

    menu:
        [ask-name] "Ask her name" if affection >= 3:
            eileen [name-answer] "I'm Eileen."
            jump end
        [leave] "Leave":
            jump end

label end:
    end
```

The grammar highlights:

- comments
- strings
- text IDs like `[intro-hello]`
- variables like `$affection`
- labels, menus, jumps, conditionals, and `end`
- scene/show/audio commands
- transition and text effects
- speaker names

## GitHub preview

GitHub does not load custom TextMate grammars from a repository. The `.vn` syntax highlighting is for editors, not GitHub's code viewer.
