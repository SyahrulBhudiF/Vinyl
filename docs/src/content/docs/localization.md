---
title: Localization
---

Scripts keep source text plus stable IDs:

```vn
eileen [intro.hello] "Hello."
menu:
    [intro.ask] "Ask":
        end
```

Locale files use Fluent syntax:

```ftl
intro-hello = Hello.
intro-ask = Ask
```

Commands:

```bash
vn extract-locales fixtures/mvp
vn check fixtures/mvp --locale id-ID
vn run fixtures/mvp --locale id-ID
```

Missing translations fall back to source script text at runtime.
