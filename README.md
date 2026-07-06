# Vinyl

Vinyl adalah engine visual novel berbasis Rust, tapi **game developer tidak perlu menulis atau build Rust**.
Target pemakaian utama: download binary `vn_cli`, buat project, tulis file `.vn`, taruh asset, cek, lalu jalankan/export lewat tooling.

## Untuk siapa?

- Writer / narrative designer yang ingin bikin visual novel pakai text file.
- Developer yang ingin runtime VN renderer-independent.
- Project yang butuh parser, validator, save system, localization, dan integrasi renderer seperti Bevy.

## Status

MVP engine dan CLI tooling sedang dibangun. Yang sudah ada:

- Script `.vn` indentation-based.
- Project manifest `vinyl.toml`.
- Asset resolver.
- Parser + validator dengan diagnostics multi-file.
- Menu, jump, condition, assignment, expression.
- Transition/text-effect metadata.
- Runtime VM + deterministic save/rollback.
- Localization pakai Fluent `.ftl`.
- Writer CLI: `new`, `check`, `run`, `dump-ast`, `dump-ir`, `list-assets`, `extract-locales`.
- Astro Starlight docs site di `docs/`.
- CI dan release workflow untuk prebuilt binary.

## Install

### Cara normal untuk game developer

Ambil binary dari GitHub Releases:

- Linux: `vn_cli-linux-x86_64`
- macOS: `vn_cli-macos-aarch64`
- Windows: `vn_cli-windows-x86_64.exe`

Lalu rename ke `vn_cli` dan taruh di PATH.

Contoh Linux/macOS:

```bash
chmod +x vn_cli-linux-x86_64
mv vn_cli-linux-x86_64 vn_cli
./vn_cli --help
```

### Cara developer engine

Kalau kamu mau ngembangin engine ini:

```bash
cargo build
cargo test --workspace
```

## Quickstart

Buat project baru:

```bash
vn_cli new my-game
cd my-game
```

Struktur yang dibuat:

```text
my-game/
├── vinyl.toml
├── script/
│   └── start.vn
├── locale/
│   └── en-US.ftl
└── assets/
    ├── bg/
    ├── sprites/
    │   └── eileen/
    └── audio/
```

Cek project:

```bash
vn_cli check .
```

Jalankan smoke runtime CLI:

```bash
vn_cli run .
```

Pakai locale tertentu:

```bash
vn_cli check . --locale en-US
vn_cli run . --locale en-US
```

## Contoh script `.vn`

`script/start.vn`:

```vn
label start:
    eileen [intro-hello] "Hello."
    menu:
        [intro-continue] "Continue":
            end
```

Dengan localization di `locale/en-US.ftl`:

```ftl
intro-hello = Hello.
intro-continue = Continue
```

Contoh lebih lengkap:

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

Asset path yang dicari:

```text
scene bg room                  -> assets/bg/room.png
show eileen happy at center    -> assets/sprites/eileen/happy.png
play music theme               -> assets/audio/theme.*
```

## File `vinyl.toml`

Contoh manifest:

```toml
[project]
id = "my-game"
title = "My Game"
version = "0.1.0"
default_locale = "en-US"

[paths]
script = "script"
assets = "assets"
locales = "locale"

[assets]
backgrounds = "bg"
sprites = "sprites"
audio = "audio"
```

## Command CLI

```bash
vn_cli new <project>
```

Buat project baru siap tulis.

```bash
vn_cli check <project> [--locale en-US]
```

Parse + validasi script, label, asset, dan locale.

```bash
vn_cli run <project> [--locale en-US]
```

Jalankan runtime deterministic smoke test. Berguna untuk memastikan script bisa dieksekusi.

```bash
vn_cli list-assets <project>
```

Tampilkan semua asset yang direferensikan script.

```bash
vn_cli extract-locales <project>
```

Generate entry Fluent dari text id di script.

```bash
vn_cli dump-ast <project>
vn_cli dump-ir <project>
```

Debug parser/compiler output sebagai JSON.

```bash
vn_cli fmt <project>
```

Saat ini masih parse/check placeholder. Source formatter penuh belum dibuat.

## Dokumentasi website

Docs site ada di `docs/` dan memakai Astro Starlight.

```bash
cd docs
npm install
npm run dev
npm run build
```

## GitHub Actions buat apa?

GitHub Actions adalah automation yang jalan di GitHub tiap push, pull request, atau tag release.

Di repo ini dipakai untuk 2 hal:

### 1. CI quality gate

File: `.github/workflows/ci.yml`

CI menjalankan:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --no-fail-fast --no-default-features
cd docs && npm run build
```

Gunanya:

- Ngecek format kode.
- Ngecek lint Rust.
- Ngejalanin test otomatis.
- Ngecek docs bisa build.
- Mencegah code rusak masuk ke branch utama.

### 2. Release binary otomatis

File: `.github/workflows/release.yml`

Saat kamu push tag seperti:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub akan build `vn_cli` untuk Linux/macOS/Windows dan upload ke GitHub Releases.

Gunanya penting: **game developer bisa download binary langsung, tidak perlu install Rust atau cargo build**.

## Development checks lokal

Sebelum commit besar:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo test --workspace --no-fail-fast --no-default-features
```

## Workspace crates

- `vn_core`: AST, IR, compiler, VM, save model.
- `vn_script`: parser, validator, manifest, asset resolver, localization loader.
- `vn_runtime`: presentation orchestration.
- `vn_bevy`: Bevy renderer integration.
- `vn_cli`: writer/developer CLI.
