#![allow(dead_code)]
use dirtybase_db::{TableEntityTrait, field_values::FieldValue, types::FromColumnAndValue};
use dirtybase_db_macro::DirtyTable;
use std::collections::HashMap;

#[test]
fn test_basic_fields() {
    #[derive(Debug, Default, DirtyTable)]
    #[dirty(table = "example_table")]
    struct Example {
        id: u64,
        name: String,
    }

    assert_eq!(
        Example::table_name(),
        "example_table".to_string(),
        "Table name did not match"
    );
    assert_eq!(
        Example::table_columns().len(),
        2,
        "Table suppose to have 2 fields"
    );
}

#[test]
fn test_field_skipping() {
    #[derive(Debug, Default, DirtyTable)]
    #[dirty(table = "example_table")]
    struct Example {
        id: u64,
        name: String,
        #[dirty(skip)]
        skip_this: bool,
    }

    assert_eq!(
        Example::table_columns().len(),
        2,
        "Table suppose to have 2 fields"
    );
}

#[test]
fn test_column_name() {
    #[derive(Debug, Default, DirtyTable)]
    #[dirty(table = "example_table")]
    struct Example {
        id: u64,
        name: String,
        #[dirty(col = "created_at")]
        date: String,
    }

    let columns = Example::table_columns();

    assert!(
        columns.contains(&"created_at"),
        "'created_at' should be in the list of columns"
    );
}

#[test]
fn test_complex_field() {
    type DiscountRange = Vec<u64>;

    #[derive(Debug, Default, DirtyTable)]
    #[dirty(table = "example_table")]
    struct Example {
        id: u64,
        name: String,
        #[dirty(col = "created_at")]
        date: String,
        #[dirty(from = "string_to_vec")]
        discount_allowed: DiscountRange,
    }

    impl Example {
        pub fn string_to_vec(column: Option<&FieldValue>) -> DiscountRange {
            if let Some(value) = column {
                return value
                    .to_string()
                    .split(',')
                    .map(|v| v.trim().parse::<u64>().unwrap_or_default())
                    .collect::<Vec<u64>>();
            }
            Vec::new()
        }
    }

    let mut data = HashMap::new();
    data.insert("name".to_string(), FieldValue::String("John Doe".into()));
    data.insert(
        "created_at".to_string(),
        FieldValue::String("July-20-2023".into()),
    );
    data.insert(
        "discount_allowed".to_string(),
        FieldValue::String("25, 20".to_string()),
    );

    let instance = Example::from_column_value(data).unwrap();

    assert_eq!(
        instance.discount_allowed,
        vec![25, 20],
        "Discount allowed should have been (25, 20)"
    )
}

#[test]
fn test_xyz() {}
