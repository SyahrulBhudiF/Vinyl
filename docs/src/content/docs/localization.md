---
title: Localization
---

Vinyl keeps readable source text in scripts and uses stable text IDs to select translations.

## Add text IDs

```vn
eileen [intro-hello] "Hello."
menu:
    [intro-ask] "Ask":
        end
```

IDs must be unique across all loaded scripts. Hyphenated IDs work directly with Fluent and are recommended.

## Create locale files

Each file is named `<locale>.ftl` under the configured locale directory:

```text
locale/en-US.ftl
locale/id-ID.ftl
```

```ftl
intro-hello = Hello.
intro-ask = Ask
```

Indonesian example:

```ftl
intro-hello = Halo.
intro-ask = Tanya
```

## Extract source entries

```bash
vn extract-locales . > locale/template.ftl
```

The command prints one Fluent entry for every script text ID, using source text as the initial value. Review the output before replacing an existing translation file.

## Validate a locale

```bash
vn check . --locale id-ID
```

Validation reports missing locale entries and duplicate text IDs. Without `--locale`, Vinyl validates the loaded locale catalogs and runs with `project.default_locale`.

## Run a locale

```bash
vn run . --locale id-ID
vn smoke . --locale id-ID
```

Missing translations fall back to source script text at runtime. The default player interface—Pause, Save, Load, Settings, and related labels—remains English; locale selection affects game script content only.

## Current Fluent support

Vinyl currently reads flat Fluent message values. Plain text is supported. Fluent attributes, selectors, and placeable expressions are not evaluated; keep locale values simple.
