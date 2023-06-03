use actix_web::*;
use dirtybase;
use leptos::*;
use leptos_actix::{generate_route_list, LeptosRoutes};

fn app(cx: leptos::Scope) -> impl IntoView {
    use dirtybase_tos_app::*;

    view! { cx, <App /> }
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    use actix_files::Files;

    _ = dotenvy::dotenv();

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr.clone();

    log::info!("serving at {addr}");

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(app);

    let dirtybase_app = actix_web::web::Data::new(dirtybase::app::setup().await);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;

        let site_root = leptos_options.site_root.clone();

        App::new()
            .app_data(dirtybase_app.clone())
            .leptos_routes(leptos_options.to_owned(), routes.to_owned(), app)
            .service(Files::new("/", site_root.to_owned()))
            .wrap(middleware::Compress::default())
    })
    .bind(addr)?
    .run()
    .await
}
