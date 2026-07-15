---
title: Editor Setup
---

## VS Code syntax highlighting

Vinyl includes a VS Code-compatible TextMate grammar under:

```text
editors/vscode-vinyl/
```

Package and install it locally:

```bash
cd editors/vscode-vinyl
pnpm install
pnpm package
code --install-extension vinyl-vn-syntax-0.1.0.vsix --force
```

Open the extension directory in VS Code and press `F5` to test it in an Extension Development Host.

## Highlighted syntax

The grammar recognizes:

- comments and quoted strings
- text IDs such as `[intro-hello]`
- variables such as `$affection`
- labels, jumps, menus, conditionals, and `end`
- scene, sprite, and music commands
- fade, dissolve, instant, and typewriter effects
- speaker names and operators

## Recommended authoring loop

```bash
vn check .
vn run .
```

Run `vn check` after structural edits. It provides the authoritative parser and validator result; editor highlighting is not a substitute for validation.

## Formatting and indentation

- Use four spaces for each block level.
- Do not use tabs.
- Keep expression operators separated by spaces.
- Save scripts as UTF-8.
- Use stable text IDs for translated content.

## GitHub preview

GitHub does not load repository-provided TextMate grammars, so `.vn` files may appear as plain text in its code viewer. This does not affect local editor support or the engine.
