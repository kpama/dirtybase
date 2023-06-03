# DirtyBase


## Developing
To run the application and watch for code changes run the following command in the application's root folder.

`cargo watch -c -q -x "run -p dirtybase -- serve" `

If you want to display logs run the following command from the application's root folder.

`RUST_LOG=debug cargo watch -c -q -x "run -p dirtybase -- serve" `
