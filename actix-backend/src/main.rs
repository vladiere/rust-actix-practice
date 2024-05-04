#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_backend::{
        appstate::*,
        middleware::*,
        services::auth::{basic_auth, register_user},
        services::products::{create_product, delete_product, retrieve_product, update_product},
    };
    use actix_cors::Cors;
    use actix_web::{
        http,
        middleware::Logger,
        web::{self, Data},
        App, HttpServer,
    };
    use actix_web_httpauth::middleware::HttpAuthentication;
    use dotenv::dotenv;
    use sqlx::postgres::PgPoolOptions;
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .service(register_user)
            .service(basic_auth)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(create_product)
                    .service(update_product)
                    .service(delete_product)
                    .service(retrieve_product),
            )
            .wrap(cors)
            .wrap(Logger::new("%a - %r %s [%{User-Agent}i]"))
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
