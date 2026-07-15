---
title: CLI Reference
---

All project arguments default to the current directory (`.`).

## `vn new`

```bash
vn new [project]
```

Creates a manifest, starter script, English locale file, and asset directories. Existing files with the generated names are overwritten, so use a new or intentionally replaceable directory.

## `vn check`

```bash
vn check [project] [--locale en-US]
```

Parses every `.vn` file and validates:

- exactly one `label start`
- duplicate and missing labels
- script syntax and indentation
- referenced assets
- duplicate text IDs
- locale entries

Validation completes before `vn run` opens a window.

## `vn run`

```bash
vn run [project] [--locale en-US]
```

Validates the project and launches the rendered desktop player.

## `vn smoke`

```bash
vn smoke [project] [--locale en-US]
```

Runs deterministic headless verification without opening a window. It advances the fixture story, checks save serialization and compatibility, selects a choice, and verifies rollback. Use it in scripts or CI when rendering is unnecessary.

## `vn list-assets`

```bash
vn list-assets [project]
```

Prints every referenced background, sprite, and audio path after applying `vinyl.toml` path rules.

## `vn extract-locales`

```bash
vn extract-locales [project]
```

Prints Fluent entries for all script text IDs. Redirect output to create a translation template:

```bash
vn extract-locales . > locale/template.ftl
```

## `vn dump-ast`

```bash
vn dump-ast [project]
```

Prints the parsed abstract syntax tree as formatted JSON.

## `vn dump-ir`

```bash
vn dump-ir [project]
```

Prints compiled VM operations and resolved label targets as formatted JSON.

## `vn fmt`

```bash
vn fmt [project]
```

Currently parses the project and reports `ok`; it does not rewrite source files yet.

## Exit behavior

Commands return a non-zero status on load, parse, validation, serialization, or player startup failures. Diagnostics include file, line, and column where available.
