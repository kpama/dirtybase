use std::collections::HashMap;

use actix_web::{get, HttpResponse, Responder, Scope};
use busybody::helpers::{provide, service};
use dirtybase_db::entity::user::{UserEntity, USER_TABLE};
use dirtybase_db_types::TableEntityTrait;

use crate::app::{
    cache_manager::CacheManager,
    core::time::now,
    model::{
        app::{AppEntity, AppRepository},
        company::CompanyEntity,
        dirtybase_user::{
            dtos::out_logged_in_user_dto::LoggedInUser, DirtybaseUserEntity,
            DirtybaseUserRepository,
        },
        role::RoleEntity,
        role_user::RoleUserEntity,
    },
    DirtyBase,
};

pub fn register_routes(scope: Scope) -> Scope {
    scope.service(cache_endpoint)
}

#[get("/cache")]
async fn cache_endpoint() -> impl Responder {
    let cache_manager: CacheManager = provide().await;

    let tag_cache_manager = cache_manager.tags(&["tag_one"]).await;

    let status: UptimeStatus = tag_cache_manager
        .remember("uptime", None, || async {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            UptimeStatus {
                is_up: true,
                msg: "Version 1".into(),
            }
        })
        .await;

    let f = UptimeStatus {
        is_up: false,
        msg: "Version 2".into(),
    };
    let time = now();
    _ = cache_manager
        .add("uptime", &f, time.add_days(20).into())
        .await;

    HttpResponse::Ok().json(status)
}

#[get("/d-users")]
async fn serve_d_users() -> impl Responder {
    let core_user_id = "01h4qpe1gr7nm7d6zkpq7gxedx";
    let app = service::<DirtyBase>().await;
    let repo = provide::<AppRepository>().await;
    let dirty_user_repo: DirtybaseUserRepository = provide().await;

    let result = dirty_user_repo
        .get_user_logged_in_info(core_user_id)
        .await
        .unwrap_or_else(|e| {
            log::error!("{}", e);
            LoggedInUser::default()
        });

    // let result = dirty_user_repo.fake(core_user_id).await;

    // let result = app
    //     .schema_manger()
    //     .select_from_table(DirtybaseUserEntity::table_name(), |q| {
    //         q.select_multiple(&DirtybaseUserEntity::table_column_full_names())
    //             .left_join_table_and_select::<UserEntity, DirtybaseUserEntity>(
    //                 UserEntity::id_column().unwrap(),
    //                 UserEntity::foreign_id_column().unwrap(),
    //                 Some("user"),
    //             );
    //     })
    //     .fetch_all_to::<LoggedInUser>()
    //     .await
    //     .unwrap();
    // let result = repo.find_all_by_user(core_user_id).await;

    HttpResponse::Ok().json(result)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<center><h1>DirtyBase is Up</h1></center>")
}

#[get("/users")]
async fn serve_users() -> impl Responder {
    let app = service::<DirtyBase>().await;
    let manager = app.schema_manger();
    let result = manager
        .select_from_table(USER_TABLE, |q| {
            q.select_table::<UserEntity>()
                .left_join_table::<RoleUserEntity, UserEntity>("core_user_id", "id")
                .left_join_table::<RoleEntity, RoleUserEntity>("id", "core_app_role_id")
                .left_join_table_and_select::<AppEntity, RoleEntity>(
                    "id",
                    "core_app_id",
                    Some("app"),
                )
                .left_join_table_and_select::<CompanyEntity, AppEntity>(
                    "id",
                    "core_company_id",
                    Some("app.company"),
                );
        })
        .fetch_all()
        .await;

    // let x = StructuredColumnAndValue::from_results(result.unwrap());
    // HttpResponse::Ok().json(x)
    HttpResponse::Ok().json(result.unwrap())
}

#[get("/d-children")]
async fn serve_d_children() -> impl Responder {
    let app = service::<DirtyBase>().await;
    let manager = app.schema_manger();

    let core_user_id = "01h4qpe1gr7nm7d6zkpq7gxedx";

    let query_result = manager
        .select_from_table(DirtybaseUserEntity::table_name(), |query| {
            query
                .select_multiple(&DirtybaseUserEntity::table_column_full_names())
                .left_join_table_and_select::<UserEntity, DirtybaseUserEntity>(
                    UserEntity::id_column().unwrap(),
                    UserEntity::foreign_id_column().unwrap(),
                    Some("user"),
                )
                .left_join_table::<RoleUserEntity, UserEntity>(
                    RoleUserEntity::role_user_fk_column(),
                    UserEntity::id_column().unwrap(),
                )
                .left_join_table_and_select::<RoleEntity, RoleUserEntity>(
                    RoleEntity::id_column().unwrap(),
                    RoleUserEntity::app_role_fk_column(),
                    Some("app.role"),
                )
                .left_join_table_and_select::<AppEntity, RoleEntity>(
                    AppEntity::id_column().unwrap(),
                    AppEntity::foreign_id_column().unwrap(),
                    Some("app"),
                )
                .left_join_table_and_select::<CompanyEntity, AppEntity>(
                    CompanyEntity::id_column().unwrap(),
                    CompanyEntity::foreign_id_column().unwrap(),
                    Some("app.company"),
                )
                .without_table_trash::<AppEntity>()
                .without_table_trash::<CompanyEntity>()
                .without_table_trash::<UserEntity>()
                .without_table_trash::<RoleUserEntity>()
                .eq(DirtybaseUserEntity::user_id_column(), core_user_id);
        })
        .fetch_all()
        .await
        .unwrap();

    HttpResponse::Ok().json(query_result)
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct UptimeStatus {
    is_up: bool,
    msg: String,
}
