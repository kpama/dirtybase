use super::{UserEntity, USER_TABLE};
use crate::base::manager::Manager;

pub struct UserService;

impl UserService {
    pub async fn find_or_create_default_user(manager: &mut Manager) -> UserEntity {
        let mut user = UserEntity::from_env();

        let result = manager
            .select_from_table(USER_TABLE, |q| {
                let columns = vec![
                    "internal_id",
                    "id",
                    "username",
                    "email",
                    "password",
                    "created_at",
                    "updated_at",
                    "deleted_at",
                ];
                q.select_multiple(&columns);
                if !user.id.is_empty() {
                    q.eq("id", &user.id);
                } else {
                    q.eq("username", &user.username).eq("email", &user.email);
                }
            })
            .fetch_all_as_json()
            .await;

        if !result.is_empty() {}

        user
    }
}
