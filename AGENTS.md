# AGENTS.md

Battery-included Rust web framework (axum + sqlx + CEL). Rust 2024, `resolver = "3"`, workspace version `0.1.0`.

## Commands

`just` and `watchexec` are **not installed** in this environment — use the raw cargo commands the `justfile` wraps:

```bash
# dev server (default DB is in-memory sqlite, no external service needed)
RUST_LOG=trace cargo run -p dirtybase_app --example dev -- serve
# other runtime subcommands after `--`:
cargo run -p dirtybase_app --example dev -- migrate up
cargo run -p dirtybase_app --example dev -- migrate list      # up|down|list|refresh|reset
cargo run -p dirtybase_app --example dev -- seed list
cargo run -p dirtybase_app --example dev -- seed -n <name>
cargo run -p dirtybase_app --example dev -- cron start       # start|run <id>|stop <id>|end <id>|exit

# tests
cargo test                                        # ~54 tests, fully hermetic (no DB/Redis/.env)
cargo test -p dirtybase_db_macro --test table_test
cargo test -p dirtybase_contract -- --test-threads=1   # avoids env-mutation flake (see Gotchas)
cargo test <name>                                   # substring match on test fn name
cargo fmt --all
cargo doc --no-deps
```

There is **no CI, no `build.rs`, no `rustfmt.toml`/`clippy.toml`, and no `.github/`**. Nothing gates commits except your own discipline.

`just generate-env` is the one recipe worth remembering: it concatenates every `packages/*/config_template/*.env.defaults` into root `.env.defaults` **and** copies it to `bin/cli/src/stubs/.env.defaults.stub.txt`. Run the concatenation by hand if you add a config key.

## Architecture

- `contract/` (`dirtybase_contract`) — **the boundary crate.** All cross-package interfaces live here (`app_contract`, `db_contract`, `auth_contract`, `http_contract`, `cli_contract`, `config_contract`, …). Feature crates depend on it; they do not depend on each other.
- `packages/app` (`dirtybase_app`) — the composition root. `setup_using()` in `packages/app/src/lib.rs:42` registers core extensions in a **load-order-sensitive** sequence (multitenant first, then session, auth, db, encrypt, entry, cache, cron).
- `packages/db_macro` — the ORM proc-macro crate. Only depends on `dirtybase_common`.
- `bin/cli` (`dirtybase_cli`) — scaffolding tool, separate binary. `bin/ui` — server that embeds `bin/ui/embedded/` via `include_dir!`.
- Dependencies are wired through `[patch.crates-io]` in root `Cargo.toml:25-42` — every `dirtybase_*` crate is patched to its local path, so a `version = "*"` dep always resolves in-tree. `[workspace.metadata.publish] order` (`Cargo.toml:9-23`) defines publish order; `dirtybase_app` must stay last.

Extensions are the composition mechanism: implement `dirtybase_contract::ExtensionSetup` (`contract/src/extension.rs:31`) and `app.register(MyExt).await`. `ExtensionManager::setup_boot_and_run` calls `setup` → `boot` → `run` in registration order and only runs **once** per process.

## Two distinct CLIs — don't confuse them

1. **`dirtybase_cli`** (scaffolding, `bin/cli/src/main.rs`): `new <name>`, `init`, `make migration <name>`, `make seeder <name>`. `--package/-p` is a *top-level* arg, so it must precede the subcommand: `dirtybase_cli -p dirtybase_app init`. It resolves the target crate by shelling out to `cargo metadata --no-deps` (`bin/cli/src/metadata.rs:4`) from the **current directory** — run it from the workspace root.
2. **Runtime app subcommands** — `serve`, `migrate`, `seed`, `cron` are *not* in the CLI crate. Extensions register them into a `CliCommandManager` at startup (`packages/db/src/command.rs:30`, `packages/cron/src/cli.rs:15`, `packages/app/src/dirtybase_entry/commands_setup.rs:5`) and they're dispatched after `--`.

## Config

- All keys are `DTY_*` env vars, deserialized by the `config` crate (no derive macro — config structs are hand-written `#[derive(Deserialize)]` + `impl TryFromDirtyConfig`).
- `.env.defaults` is **committed**; `.env`, `.env.prod`, `.env.stage`, `.env.dev` are **gitignored and absent**. `load_dot_env` (`contract/src/config_contract.rs:55`) loads them in the order `.env.defaults` → `.env.prod` → `.env.stage` → `.env` → `.env.dev`, each with `_override` (later wins). **Missing files are fine; malformed ones `panic!`.**
- **CWD-relative.** `load_dot_env(None)` resolves against `./`. Always run examples from the workspace root or you silently lose `.env.defaults`.
- `config::Environment` has no `.separator()`, so keys are only lowercased with the prefix stripped: `DTY_APP_WEB_COOKIE.HTTP_ONLY` → `web_cookie.http_only`.
- Per-tenant JSON overrides take precedence over global config (`contract/src/app_contract/context.rs:111-152`) via `Context::get_config_once::<C>()`.
- Defaults ship `DTY_DB_DEFAULT="sqlite"` with `sqlite::memory:`, so the dev server boots with no external service.

## ORM: `#[derive(DirtyTable)]`

The ORM is **one** derive (`DirtyTable`) plus **one** inert attribute `#[dirty(...)]`. There is no `#[column(...)]` and no `#[dirty_belongs_to]`. Reference shapes: `packages/db_macro/examples/table.rs`, `tutorial.rs`, `morph_many.rs`, `packages/session/src/storage/database.rs:89`.

**The single most important fact: the proc-macro is a pure syntax→code transform. It never touches a database.**

- No compile-time SQL checking anywhere in the workspace — zero `query!`/`query_as!`/`query_scalar!` usages, no `DATABASE_URL` string, no `.sqlx/` dir, no `sqlx-data.json`. `sqlx`'s `macros` feature is enabled but unused. (The `sqlx` dep in `packages/db_macro/Cargo.toml` is dead.)
- All SQL is built at **runtime** by `QueryBuilder` (`packages/common/src/db/table_model.rs:115`) and each backend's `build_query` (`packages/db/src/connector/*/ *_connector.rs`).
- **Consequence: a `DirtyTable` model whose columns don't match the real schema will compile cleanly and only fail at runtime.** Verify schema changes against a live DB, not `cargo check`.

Attribute keys that are **silently ignored** (no unknown-key error path — a typo fails silently):
- `no_timestamp` / `no_soft_delete` appear in nearly every example but are **not recognized**; timestamps and soft delete are already off by default (`packages/db_macro/src/attribute_type.rs:23-38`).
- Relationship keys are `local_col` / `foreign_col`, **not** `local_key` / `foreign_key` (`packages/db_macro/src/attribute_type.rs:150-165`). `examples/relationship.rs` uses the wrong ones.

Recognized: table-level `table`, `id`, `id_column`, `created_at`, `updated_at`, `deleted_at`, `timestamp(s)/timestampable`, `id_not_auto`, `soft_delete/soft_deletable`; field-level `rel(kind=…)`, `col`, `from`, `into`, `skip`, `skip_select`, `skip_insert`, `flatten`, `embedded`. `rel(kind=…)` is the only required sub-key and `panic!`s if missing/unknown — kinds: `has_one`, `belongs_to`, `has_many`, `has_one_through`, `has_many_through`, `morph_one`, `morph_many`.

## Migrations are hand-written and explicitly registered

There is **no directory scan** (no `include_dir`, no `sqlx::migrate!`). Each crate's `dirtybase_entry/migration.rs` lists migrations in a `register_migration![...]` macro (`contract/src/lib.rs:30`); `Migrator` collects them from every extension and sorts by `id()`.

- `Migration::id()` is the **lowercased struct name**, not the filename (`contract/src/db_contract/migration.rs:30`).
- A migration is only visible if its extension's `migrations()` returns it, gated on config (e.g. auth only exposes them when enabled *and* storage is db — `packages/auth/src/dirtybase_entry.rs:56`).
- Convention: file `mig_<unix_ts>_<lowercased_name>.rs`, struct `Mig<unix_ts><PascalCase>`, e.g. `packages/session/src/dirtybase_entry/migration/mig_1744202277_create_session_table.rs`.
- **No migration-generating macro exists** — `packages/db_macro/src/lib.rs:159` is a TODO. Write `up`/`down` by hand using `manager.create_table_schema(...)` / `manager.drop_table(...)`.
- Seeders: no discovery either. Register explicitly in `dirtybase_entry/seeder.rs` via `SeederRegisterer::register(name, …)`; a seeder named `"all"` runs for every `--name`. Triggered by `on_cli_command` when `cmd == "seed"`.

## HTTP routing

Inside `ExtensionSetup::register_routes(&self, manager: &mut RouterManager)`, pick a route group — each has a configurable prefix from config: `manager.general(None, |r| …)`, `.backend(…)`, `.api(…)`, `.insecure_api(…)`, `.dev(…)`.

`get/post/put/delete(path, handler, name)` register a **named** route (for `named_routes_axum` reverse lookup); `get_x/post_x/…(path, handler)` are the anonymous variants. `*_with_middleware` variants take a middleware list. See `packages/app/examples/dev.rs:47-55` and `contract/src/http_contract/router_builder.rs:113,149`.

## Features

`packages/app/Cargo.toml`: `default = ["full"]` where `full = ["template", "realtime", "auth", "permission"]`. Note **`multitenant` is NOT in `full`** despite being an optional dep — opt in explicitly with `--features multitenant`. `dev = ["full", "dirtybase_auth/seeders"]`.

`packages/auth`: `openid`, `permission`, `migration`, `seeders` — only `permission` is actually read in code; the other three are declared-but-unused. **No test is feature-gated**, so everything runs under default features.

## Examples are the integration surface

`cargo test` never executes examples; they're run manually and most use `make_sqlite_in_memory_manager()`. The common `setup_db`/`create_tables`/`seed_tables` triple appears throughout `packages/db_macro/examples/`.

Three examples require external services and will hang or panic without them:
- `packages/app/examples/db.rs` — hardcoded Postgres at `postgres://dbuser:dbpassword@postgres/dirtybase`
- `packages/3rd_client/examples/redis.rs` — needs Redis, `.unwrap()`s the connection
- `packages/mail/examples/basic.rs` — hardcoded SMTP relay `192.168.0.144:1026`, `panic!`s on failure

Several examples are deliberately inert to keep compiling (empty `main`, `#![allow(dead_code)]`) — a warning-free example is not evidence it works.

## Gotchas / known bugs

- **`make migration -p <pkg>` does not run `cargo fmt`.** `bin/cli/src/commands/make_migration.rs:62` does `Command::new(format!("cargo -p {}", pkg))`, passing the whole string as the program name — it silently fails. Same in `make_seeder.rs`. Run `cargo fmt --all` yourself after scaffolding.
- **`dirtybase_cli init` doesn't create the `dirtybase_entry/seeder` directory** (`bin/cli/src/content.rs:9-17` omits it), so the `seeder/.gitkeep` write is silently discarded. `make seeder` creates it.
- **The CLI stub `.env.defaults` is stale**: it is missing `DTY_APP_WEB_STATIC_FILES_ROUTE` and `DTY_AUTH_JWT_KEY`, both of which have no serde default. A freshly scaffolded project panics at config load until they're added. Re-run the `.env.defaults` concatenation and copy to the stub to fix.
- `contract/src/config_contract/dirtybase_config.rs:273-289` writes a temp `.env` and mutates the **process environment** via `from_filename_override`; `test_default` in the same module reads those vars. Latent flake under `cargo test`'s parallel threads — use `--test-threads=1` for `dirtybase_contract`.
- The suite is not instant: `context_manager.rs:403` sleeps 5s, `lock_manager.rs` sleeps 0.5–1s.
- `CurrentEnvironment` silently falls back to `Development` for any unrecognized `DTY_APP_ENV` value. The toml file variant is `_stage`, not `_staging`.
- `contract/src/dot_env_man.rs` `DotEnvManipulator` `unwrap()`s `split_once("#")` on every value line — it panics on a value without `#`.
- The `docs` git submodule (`.gitmodules`) is **not checked out**; don't assume `docs/` exists.

## Conventions

Conventional Commits, lowercase, scoped to the crate with an underscore: `fix(core): …`, `doc(app_contract): …`, `test(config_contract): …`, `fix(auth): …`. Work lands on `dev` via PRs (merge commits `Merge pull request #N from kpama/dev`).

Note: `Cargo.lock` is currently dirty in-tree — the last commit updated `Cargo.toml` pins (`cel 0.14.5`, `syn 3.0.6`) without regenerating the lock, so any build re-resolves it. Expect that diff.
