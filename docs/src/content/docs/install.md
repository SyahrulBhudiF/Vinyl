---
title: Install Vinyl
---

Vinyl is a renderer-independent visual novel engine for Rust.

## Install flow

Download a prebuilt `vn_cli` binary for your OS from Releases, then put it on your PATH.

```bash
vn_cli new my-game
cd my-game
vn_cli check .
vn_cli run .
```

## Commands writers use

```bash
vn_cli new my-game
vn_cli check my-game
vn_cli run my-game
vn_cli extract-locales my-game
vn_cli list-assets my-game
```

Rust is only needed if you are hacking on the engine itself.
