---
title: Install Vinyl
---

Vinyl ships as one executable named `vn`. It contains both the desktop player and project-authoring commands. Download it from the latest GitHub release; Rust is not required for game authors.

## Linux

### Requirements

- x86-64 Linux
- X11 or XWayland
- ALSA, udev, X11, Xcursor, Xi, and Xrandr runtime libraries

Ubuntu or Debian:

```bash
sudo apt install libasound2 libudev1 libx11-6 libxcursor1 libxi6 libxrandr2
chmod +x vn-linux-x86_64
sudo mv vn-linux-x86_64 /usr/local/bin/vn
vn --help
```

Release pages may also provide DEB and RPM packages. Native Wayland is not currently supported; use an X11 session or XWayland.

## macOS

The current release target is Apple Silicon:

```bash
chmod +x vn-macos-aarch64
sudo mv vn-macos-aarch64 /usr/local/bin/vn
vn --help
```

If macOS blocks the downloaded binary, allow it in **System Settings → Privacy & Security**.

## Windows

In PowerShell:

```powershell
New-Item -ItemType Directory -Force C:\Tools\Vinyl
Move-Item .\vn-windows-x86_64.exe C:\Tools\Vinyl\vn.exe
C:\Tools\Vinyl\vn.exe --help
```

Optionally add `C:\Tools\Vinyl` to the user `Path`, then open a new terminal and run `vn --help`.

## Verify the installation

```bash
vn --help
vn new hello-vinyl
cd hello-vinyl
vn check .
vn run .
```

`vn check` must print `ok`. `vn run` validates the project before opening a window.

## Engine development installation

Only engine contributors need Rust and the docs toolchain:

```bash
cargo build
cargo test --workspace
cd docs
pnpm install
pnpm build
```
