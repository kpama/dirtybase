use std::{collections::HashMap, fmt::Display};

use dirtybase_helper::time::current_datetime;
use serde::{Deserialize, Serialize};

use crate::db_contract::{
    field_values::FieldValue,
    types::{ArcUuid7, ColumnAndValue, DateTimeField, FromColumnAndValue, NameField},
};

use super::{permission::Permission, role::Role, PermissionRecordAction};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Actor {
    id: Option<ArcUuid7>,
    user_id: Option<ArcUuid7>,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    roles: Option<HashMap<NameField, ArcUuid7>>,
    permissions: Option<HashMap<NameField, ArcUuid7>>,
}

pub struct ActorPayload {
    pub action: Option<PermissionRecordAction>,
    pub id: Option<ArcUuid7>,
    pub user_id: Option<ArcUuid7>,
}

impl Actor {
    pub fn touch_created_at(&mut self) {
        self.created_at = current_datetime().into();
    }

    pub fn touch_updated_at(&mut self) {
        self.updated_at = current_datetime().into();
    }

    pub fn touch_deleted_at(&mut self) {
        self.deleted_at = current_datetime().into()
    }
}

impl Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.id.as_ref() {
            Some(id) => write!(f, "actor:{id}"),
            None => write!(f, "actor:"),
        }
    }
}

impl FromColumnAndValue for Actor {
    fn from_column_value(
        mut cv: crate::db_contract::types::ColumnAndValue,
    ) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let mut actor = Self {
            id: cv.remove("id").map(ArcUuid7::from),
            user_id: cv.remove("user_id").map(ArcUuid7::from),
            created_at: cv.remove("created_at").map(DateTimeField::from),
            updated_at: cv.remove("updated_at").map(DateTimeField::from),
            deleted_at: cv.remove("updated_at").map(DateTimeField::from),
            ..Default::default()
        };

        // roles
        actor.roles = cv.remove("roles").map(|en| {
            let mut map = HashMap::new();
            if let FieldValue::Array(roles) = en {
                for a_role in roles {
                    if let Ok(r) = Role::from_column_value(ColumnAndValue::from(a_role)) {
                        map.insert(r.name(), r.id());
                    }
                }
            }

            map
        });

        // permissions
        actor.permissions = cv.remove("permissions").map(|en| {
            let mut map = HashMap::new();
            if let FieldValue::Array(roles) = en {
                for a_role in roles {
                    if let Ok(r) = Permission::from_column_value(ColumnAndValue::from(a_role)) {
                        map.insert(r.name(), r.id());
                    }
                }
            }
            map
        });

        if !cv.is_empty() {
            return Err(anyhow::anyhow!(
                "one or more actor column value was not handled: {:#?}",
                cv.keys().collect::<Vec<&String>>()
            ));
        }

        Ok(actor)
    }
}
