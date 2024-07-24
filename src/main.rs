use std::{env, fs::create_dir, path::Path};

use actix_cors::Cors;
use actix_web::{middleware::Logger, web};
use diesel::{r2d2, SqliteConnection};
use dotenvy::dotenv;
use env_logger::Env;
use microblogs::AppState;

mod attachments;
mod feeds;
mod posts;
mod users;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    use actix_web::{App, HttpServer};

    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let uploads_dir = env::var("UPLOADS_DIR").expect("UPLOADS_DIR must be set");
    let uploads_dir_path = Path::new(&uploads_dir);
    if !uploads_dir_path.exists() {
        create_dir(uploads_dir_path).unwrap();
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(AppState {
                secret_key: std::env::var("SECRET_KEY").expect("SECRET_KEY must be set"),
                uploads_dir: uploads_dir.clone(),
            }))
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin(
                        std::env::var("FRONTEND_ORIGIN")
                            .expect("FRONTEND_ORIGIN must be set")
                            .as_str(),
                    )
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .configure(users::configure)
            .configure(attachments::configure)
            .configure(posts::configure)
            .configure(feeds::configure)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
