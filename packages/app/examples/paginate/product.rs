use dirtybase_contract::{http_contract::api::ApiResponse, prelude::*};
use dirtybase_db::{
    SeederRegisterer,
    base::{cursor_builder::CursorBuilder, manager::Manager, paginate_builder::PaginateBuilder},
};

pub struct ProductApp;

#[async_trait::async_trait]
impl ExtensionSetup for ProductApp {
    async fn setup(&mut self, _global_context: &Context) {
        tracing::info!("production application setting up");
        SeederRegisterer::register("product", |manager, _| async move {
            models::Product::seed(100, &manager).await
        })
        .await;

        ContextResourceManager::scoped("product-repo", |ctx| async move {
            let db_manager = ctx.get::<Manager>().await?;
            Ok(models::ProductRepo::new(&db_manager))
        })
        .await;
    }

    async fn migrations(&self, _context: &Context) -> Option<ExtensionMigrations> {
        use models::ProductMigration;
        Some(vec![Box::new(ProductMigration)])
    }

    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(Some("/products"), |router| {
            router.get_x("/", serve_product_list);
            router.get_x("/cursor", serve_product_list2);
        });
    }
}

async fn serve_product_list(
    CtxExt(mut repo): CtxExt<models::ProductRepo>,
    page: PaginateBuilder,
) -> impl IntoResponse {
    // let page = None;
    ApiResponse::from(repo.paginate(Some(page)).await)
}

async fn serve_product_list2(
    CtxExt(mut repo): CtxExt<models::ProductRepo>,
    cursor: CursorBuilder,
) -> impl IntoResponse {
    // let page = None;
    ApiResponse::from(repo.cursor_paginate(Some(cursor)).await)
}

pub mod models {
    use dirtybase_db::{
        TableModel,
        base::manager::Manager,
        migration::Migration,
        types::{ArcStrField, ArcUuid7, NameField, StringField},
    };
    use dirtybase_db_macro::DirtyTable;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, DirtyTable, Serialize, Deserialize)]
    pub struct Product {
        id: Option<ArcUuid7>,
        name: StringField,
        sku: NameField,
        description: ArcStrField,
    }

    impl Product {
        pub async fn seed(total: usize, manager: &Manager) {
            let mut repo = ProductRepo::new(manager);
            for count in 0..total {
                let id = count + 1;
                let record = Product {
                    id: Some(ArcUuid7::default()),
                    name: format!("Product: {}", id).into(),
                    sku: NameField::new(&format!("prod-{}", id)),
                    description: format!("Product {} description", id).into(),
                };
                _ = repo.insert(record).await;
                // .expect("could not insert a new product");
            }
        }
    }

    pub struct ProductMigration;

    #[async_trait::async_trait]
    impl Migration for ProductMigration {
        /// Migrate up aka apply the migration
        async fn up(&self, manager: &Manager) -> Result<(), anyhow::Error> {
            manager
                .create_table_schema(Product::table_name(), |bp| {
                    bp.uuid_as_id(None);
                    bp.string(Product::col_name_for_name());
                    bp.string(Product::col_name_for_sku())
                        .set_is_unique(true)
                        .set_is_nullable(false);
                    bp.text(Product::col_name_for_description());
                })
                .await
        }

        /// Migrate down aka revert the migration
        async fn down(&self, manager: &Manager) -> Result<(), anyhow::Error> {
            manager.drop_table(Product::table_name()).await
        }
    }
}
