---
title: Assets
---

Default resolution:

```text
scene bg room               -> assets/bg/room.png
show eileen happy at left   -> assets/sprites/eileen/happy.png
play music "bgm/theme.ogg"  -> assets/audio/bgm/theme.ogg
```

Use `vn list-assets <project>` to print resolved asset paths referenced by scripts.

Asset roots are configured in `vinyl.toml`.
