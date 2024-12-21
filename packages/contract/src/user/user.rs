use crate::db::types::ArcUlidField;

use super::status::UserStatus;

pub trait UserTrait {
    fn id(&self) -> ArcUlidField;
    fn username(&self) -> String;
    fn status(&self) -> UserStatus;
}
