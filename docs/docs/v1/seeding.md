---
outline: deep
---

# Seeding

Create new seeder by using the `dirtybase_cli`. To create a new seeder run the following command.

```bash
dirtybase_cli make seeder users
```

If this is the first time running the seeder command a few things will be created. A `seeder.rs` file in `src/dirtybase_entry` and a `users_seeder.rs` file in the newly created `src/dirtybase_entry/seeder` directory.

## Dirtybase Entry File

You will need make a change to your entry file in order to run your seeder. If
you do not already have a `on_cli_command()` function in your entry file, add
the following to your `dirtybase_entry.rs`.

```rust
mod seeder;
use seeder::register_seeders;

#[derive(Default)]
pub struct Extension;

#[dirtybase_contract::async_trait]
impl dirtybase_contract::ExtensionSetup for Extension {
    ....

    // Put this after your setup function

    async fn on_cli_command(
        &self,
        cmd: &str,
        matches: ArgMatches,
        _context: Context,
    ) -> ArgMatches {
        // If you already have the on_cli_command function you only need to add the if statement.
        if cmd == "seed" {
            register_seeders().await;
        };

        matches
    }

  // End new code

  ....
```

## Seeder File

TBD

## Seeder Directory Files

TBD

```

```
