use std::env;

use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    middleware::Logger,
    web,
};
use diesel::{r2d2, SqliteConnection};
use dotenvy::dotenv;
use env_logger::Env;

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

    let domain_name = env::var("DOMAIN_NAME").unwrap_or("localhost".to_string());
    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                    .cookie_name("microblogs_session".to_string())
                    .cookie_secure(false)
                    .cookie_domain(Some(domain_name.clone()))
                    .cookie_path("/".to_string())
                    .build(),
            )
            .configure(users::configure)
            .configure(posts::configure)
            .configure(feeds::configure)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
