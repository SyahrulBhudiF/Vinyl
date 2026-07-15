# The Locked Room

A tested multi-file Vinyl example with cross-file labels and jumps, nested script directories, variables, arithmetic assignment, conditions, conditional choices, localization, transitions, sprites, music, and multiple endings.

From the repository root:

```bash
vn check examples/branching-mystery
vn smoke examples/branching-mystery
vn run examples/branching-mystery
vn run examples/branching-mystery --locale id-ID
```

`vn smoke` follows the first menu choice, verifies save serialization and restoration at the menu, then verifies rollback.
