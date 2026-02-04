---
outline: deep
---

# Migrations

Create new migrations by using the `dirtybase_cli` to generate new migrations.
This will create the migration file in the `src/dirtybase_entry/migration`
folder. This will also add an entry to your `src/dirtybase_entry/migration.rs`
file. This file controls what migrations are run when you do a `cargo run -- migrate <COMMAND>`.

```bash
dirtybase_cli make migration create_cars_table
```
