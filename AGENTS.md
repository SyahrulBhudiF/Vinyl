# Vinyl — Visual Novel Engine

Renderer-independent VN engine in Rust. Workspace with 5 crates: `vn_core` (VM/IR/AST), `vn_script` (parser/validator), `vn_runtime` (orchestration), `vn_bevy` (Bevy renderer), `vn_cli` (CLI tool).

## Build & Test

```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run single test
cargo test test_name

# Run tests in specific crate
cargo test -p vn_core
cargo test -p vn_script
cargo test -p vn_runtime

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt --check
```

## Code Style

- **Errors**: `thiserror` for library errors, `anyhow` for CLI/binary
- **Serialization**: `serde` derives on all public types
- **Naming**: Rust conventions (snake_case functions, PascalCase types)
- **Docs**: `///` doc comments on public items
- **Imports**: Group std → external crates → internal crate modules
- **Tests**: In `tests/` directories, not inline `#[cfg(test)]` modules

## Architecture

- `vn_core`: AST, IR, compiler, VM (no external deps except serde/thiserror)
- `vn_script`: Parser, validator, project loader (depends on vn_core)
- `vn_runtime`: Presentation orchestration (depends on vn_core)
- `vn_bevy`: Bevy integration (depends on vn_core, vn_runtime)
- `vn_cli`: CLI binary (depends on all)

## Script Language

`.vn` files: indent-based, `label name:`, `scene bg`, `show tag attr at pos`, `speaker "text"`, `menu:`, `jump label`, `$var = value`, `if cond:`.
