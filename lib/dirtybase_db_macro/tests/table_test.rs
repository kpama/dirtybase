#![allow(dead_code)]
use dirtybase_db_macro::DirtyTable;
use dirtybase_db_types::{field_values::FieldValue, types::FromColumnAndValue, TableEntityTrait};
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
        #[dirty(from = "string_to_tuple")]
        discount_allowed: DiscountRange,
    }

    impl Example {
        pub fn string_to_tuple<'a>(_column: Option<&'a FieldValue>) -> DiscountRange {
            vec![2, 25]
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
        FieldValue::Array(vec![FieldValue::U64(50), FieldValue::U64(55)]),
    );

    let instance = Example::from_column_value(data);

    assert_eq!(
        Example::string_to_tuple(Some(&FieldValue::NotSet)),
        vec![2, 25],
        "Custom 'from' should have been called"
    );

    assert_eq!(
        instance.discount_allowed,
        vec![2, 25],
        "Discount allowed should have been (2, 25)"
    )
}
