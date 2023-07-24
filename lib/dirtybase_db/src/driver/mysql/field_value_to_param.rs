use crate::base::query_values::QueryValue;
use dirtybase_db_types::field_values::FieldValue;

impl QueryValue {
    // Transform base field value to string
    fn field_to_param(&self, field: &FieldValue) -> String {
        match field {
            FieldValue::U64(v) => v.to_string(),
            FieldValue::I64(v) => v.to_string(),
            FieldValue::F64(v) => v.to_string(),
            FieldValue::String(v) => v.clone(),
            FieldValue::Boolean(v) => {
                if *v {
                    1.to_string()
                } else {
                    0.to_string()
                }
            }
            _ => "".into(),
        }
    }
    pub fn to_param(&self, params: &mut Vec<String>) {
        match self {
            QueryValue::Field(field) => match field {
                FieldValue::Null => (),
                FieldValue::Object(_) => (),
                FieldValue::NotSet => (),
                FieldValue::U64(_) => params.push(self.field_to_param(field)),
                FieldValue::I64(_) => params.push(self.field_to_param(field)),
                FieldValue::F64(_) => params.push(self.field_to_param(field)),
                FieldValue::String(v) => params.push(v.clone()),
                FieldValue::Boolean(_) => params.push(self.field_to_param(field)),
                FieldValue::U64s(v) => params.extend(
                    v.as_slice()
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                ),
                FieldValue::I64s(v) => params.extend(
                    v.as_slice()
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                ),
                FieldValue::F64s(v) => params.extend(
                    v.as_slice()
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                ),
                FieldValue::Strings(v) => params.extend(
                    v.as_slice()
                        .iter()
                        .map(|x| x.clone())
                        .collect::<Vec<String>>(),
                ),
                FieldValue::DateTime(v) => params.push(v.to_string()),
                FieldValue::Date(v) => params.push(v.to_string()),
                FieldValue::Time(v) => params.push(v.to_string()),
                FieldValue::Array(v) => params.extend(
                    v.as_slice()
                        .iter()
                        .map(|x| self.field_to_param(x))
                        .collect::<Vec<String>>(),
                ),
            },
            _ => (), // Sub query string nothing to do here
        }
    }
}
