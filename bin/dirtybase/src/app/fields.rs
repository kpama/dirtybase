#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub enum FieldType {
    Text,
    Editor,
    Number,
    Decimal,
    Bool,
    Email,
    Url,
    Date,
    Select,
    MultiSelect,
    Relation,
    MultiRelation,
    File,
    MultiFile,
    Json,
}

pub struct Field {
    name: String,
    field_type: FieldType,
    multiple: bool,
}

impl Field {
    pub fn new(name: &str, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type: field_type.clone(),
            multiple: match field_type {
                FieldType::MultiSelect | FieldType::MultiRelation | FieldType::MultiFile => true,
                _ => false,
            },
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn name_str(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    pub fn set_field_type(&mut self, field_type: FieldType) {
        self.field_type = field_type;
    }

    pub fn multiple(&self) -> bool {
        self.multiple
    }

    pub fn set_multiple(&mut self, is_multiple: bool) {
        self.multiple = is_multiple
    }
}
