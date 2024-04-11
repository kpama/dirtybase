use dirtybase_db::field_values::FieldValue;
use dirtybase_db_macro::DirtyTable;
// use dirtybase_db::types::FromColumnAndValue;
use dirtybase_db::types::IntoColumnAndValue;
use dirtybase_db::TableEntityTrait;

#[derive(DirtyTable, Default, Clone, Debug)]
#[dirty(table = "address")]
struct Address {
    id: u64,
    name: Option<String>,
}

#[derive(DirtyTable, Default, Debug)]
#[dirty(table = "person", id = "id")]
struct Person {
    #[dirty(col = "internal_id")]
    id: u64,
    age: u64,
    #[dirty(col = "address_id", skip_select)]
    addresses: Vec<Address>,
    #[dirty(from = "field_into_status", col = "is_active")]
    status: bool,
    #[dirty(into = "override_date_to_field_value")]
    created_at: Option<DateCreated>,
    optional_foo: Vec<u64>,
    updated_at: Option<DateCreated>,
}

impl Person {
    pub fn field_into_status(column: Option<&FieldValue>) -> bool {
        FieldValue::from_ref_option_into(column)
    }

    pub fn override_date_to_field_value(&self) -> Option<FieldValue> {
        self.created_at.as_ref().map(|value| match value {
                DateCreated::Morning => FieldValue::String("A".into()),
                DateCreated::Afternoon => FieldValue::String("B".into()),
                DateCreated::Midnight => FieldValue::String("C".into()),
            })
    }
}

fn main() {
    let john = Person {
        created_at: Some(DateCreated::Midnight),
        updated_at: Some(DateCreated::Morning),
        ..Person::default()
    };

    println!("table name: {:?}", Person::table_name());
    println!("table columns: {:#?}", Person::table_columns());
    println!("table columns aliases: {:#?}", Person::column_aliases(None));
    println!("into table: {:#?}", john.into_column_value());
}

#[derive(Debug)]
enum DateCreated {
    Morning,
    Afternoon,
    Midnight,
}

impl Default for DateCreated {
    fn default() -> Self {
        Self::Morning
    }
}

impl From<&DateCreated> for FieldValue {
    fn from(value: &DateCreated) -> Self {
        match value {
            DateCreated::Afternoon => FieldValue::String("afternoon".into()),
            DateCreated::Morning => FieldValue::String("morning".into()),
            DateCreated::Midnight => FieldValue::String("midnight".into()),
        }
    }
}

impl From<FieldValue> for DateCreated {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(v) => match v.as_str() {
                "morning" => DateCreated::Morning,
                "afternoon" => DateCreated::Afternoon,
                "midnight" => DateCreated::Midnight,
                _ => Self::Morning,
            },
            _ => Self::Morning,
        }
    }
}
