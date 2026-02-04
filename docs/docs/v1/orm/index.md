---
outline: deep
---

# ORM

DirtyBase ORM is a very thin layer over your model. A model in Database is struct that has one or more fields link to a database table column.

For example, a struct that represents a post table:

```rust
#[derive(Debug)]
struct Post {
   id: Option<i64>,
   title: String,
   content: String
}
```

DirtyBase expects the struct to derive `Default` and `DirtyTable`.

```rust
#[derive(Debug, Default, DirtyTable)]
struct Post {
   id: Option<i64>,
   title: String,
   content: String
}
```
