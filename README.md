# DirtyBase


## Packages 
Prefix commands with `RUST_LOG=debug` for debug logs

|Package                |Path                       | Type | Commands                                                            |
|-----------------------|---------------------------|------|---------------------------------------------------------------------|
| dirtybase             | bin/dirtybase             | bin  | `cargo watch -c -q -x "run -p dirtybase -- serve" `
| dirtybase-ui          | bin/dirtybase-ui          | bin  | cd into the directory and run `cargo leptos watch` 
| dirtybase_tos         | bin/dirtybase_tos         | bin  | `cargo leptos watch -p dirtybase-tos` 
| dirtybase_db          | lib/dirtybase_db          | lib  | N/A
| dirtybase_db_internal | lib/dirtybase_db_internal | lib  | N/A
| dirtybase_db_macro    | lib/dirtybase_db_macro    | lib  | N/A