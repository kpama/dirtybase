---
outline: deep
---

# The `DirtyTable` trait effects

 Deriving `DirtyTable` causes few things to happen. Understanding what really going on under the hood will help you to be more comfortable using DirtyBase.

## Multiple methods are implemented for the model

 For each field in the struct these static methods will be generated:

- `col_name_for_[x]`: Gets the table column name for that field. For example, the field `title` will have the method `col_name_for_title()`
  that returns `title` as the column name.
    If the column name is different, use the `col` field of the `dirty` field attribute to specific the name:

    ```rust
    #[dirty(col=post_title)]
    title: String // col_name_for_title() will return "post_title"
    ```
  
- `from_column_for_[x]`: Processes the `FieldValue` from a query. For example, the field `title` will have the method `from_column_for_title`.
  You can override the default implementation by implementing a static method on the model with the following declaration:

  ```rust
  // where X is the type of the field
  pub fn from_column_for_my_field<'a>(field: Option<&'a FieldValue>) -> Option<X> 
  ```

  Using the `from` field of the `dirty` field attribute, override the default implementation:

  ```rust
    #[dirty(from=from_column_for_my_field)]
    title: String
  ```

- `into_column_for_[x]`: Transforms the field value into a `FieldValue`. For example, the field `title` will have the method `into_column_for_title`.
  You can override the default implementation by implementing a method on the model with the following declaration:

  ```rust
  pub fn into_column_for_my_field(&self) -> Option<FieldValue>;
  ```

  Use the `into` field on the `dirty` field attribute to register your implementation

  ```rust
    #[dirty(into=into_column_for_my_field)]
    title: String
  ```

## The `TableModel` is implemented for the model

`TableModel` is the trait the provides metadata information for the model.

- `table_name()`: Returns the model's database table name. By default the table name is the plural form on the struct's name. To specify the table name, set the `name` field on the struct's `dirty` attribute

```rust
#[derive(Debug, Default, DirtyTable)]
#[dirty(name="social_posts")] // table_name() will return "social_posts"
struct Post {
   id: Option<i64>,
   title: String,
   content: String
}
```

- `table_columns()`: Returns a slices of all the model columns name. By default, this will be all the property on the struct.
You can exclude a property by "skipping" it. Use the `skip` field of the `dirty` field attribute

```rust
#[derive(Debug, Default, DirtyTable)]
struct Post {
   id: Option<i64>,
   title: String,
   content: String,
   #[dirty(skip)] // table_columns() will return ["id", "title", "content"]
   change_set: Vec<String>
}
```

There are times you may want to skip field when inserting or selecting. Use `skip_select` or `skip_insert` respectively.

```rust
#[derive(Debug, Default, DirtyTable)]
struct Post {
   id: Option<i64>,
   title: String,
   content: String,
   #[dirty(skip_insert)] // Not included when inserting or updating
   last_updated: i64,
   #[dirty(skip_select)] // Not populated when an instance is created from a query result
   change_set: Vec<String>
}
```

- `id_field()`: Returns the ID field for the model. By default this is `id`. You can change this by setting
the `id` field on the struct's `dirty` attribute.

```rust
#[derive(Debug, Default, DirtyTable)]
#[dirty(id=internal_id)] // id_field() will return "internal_id"
struct Post {
   internal_id: Option<i64>,
   title: String,
   content: String,
}
```

- `foreign_id_column()`: Returns `[model_table]_[id_field]`.

```rust
#[derive(Debug, Default, DirtyTable)]
struct Post {
   id: Option<i64>,
   title: String,
   content: String,
}

_ = Post::foreign_id_column() // returns "posts_id"
```

- `entity_hash()`: Returns the hash representation of the current model instance. This is calculated from the
model's ID property.

- `created_at_column()`: Returns the `created at` column's name for the model. By default `created_at` is the name
for the column.
You can change the name by setting the `created_at` field on the struct's `dirty` attribute.

```rust
#[derive(Debug, Default, DirtyTable)]
#[dirty(created_at=created)] // created_at_column()  will return `Some("created")`
struct Product {
  id: i64,
  //...
  created: DateTimeField
}
```

- `updated_at_column`: Is pretty much list `created_at_column`. Use `updated_at` to change the column name.

```rust

#[derive(Debug, Default, DirtyTable)]
#[dirty(updated_at=last_updated)] // created_at_column()  will return `Some("last_updated")`
struct Product {
  id: i64,
  //...
  last_updated: DateTimeField
}
```
```
```

::: tip
To turn off the timestamps, set the `no_timestamp` on the struct's `dirty` attribute

```rust

#[derive(Debug, Default, DirtyTable)]
#[dirty(no_timestamp)]
struct Product {
  id: i64,
  name: String
}
```
:::

- `deleted_at_column`: Returns the column's name that hold the `datetime` value when model was soft deleted.
By default this column is called `deleted_at`. DirtyBase assumes records are soft deleted by default.
To override the default column name, set the `deleted_at` field on the struct's `dirty` attribute.

```rust

#[derive(Debug, Default, DirtyTable)]
#[dirty(deleted_at=deleted_on)] // deleted_at_column() will return `Some("deleted_on")`
struct Product {
  id: i64,
  name: String,
  deleted_on: DateTimeField
}
```

To turn off the soft delete field set the `no_soft_delete` field on the struct's `dirty` attribute.

```rust
#[derive(Debug, Default, DirtyTable)]
#[dirty(no_soft_delete)] // model does not support soft delete
struct Product {
  id: i64,
  name: String,
}
```

