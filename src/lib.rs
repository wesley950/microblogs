use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};
use rand::Rng;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub mod errors;
pub mod schema;

pub struct AppState {
    pub secret_key: String,
    pub uploads_dir: String,
}

pub fn generate_uid() -> String {
    const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    const LENGTH: usize = 8;

    let mut rng = rand::rngs::OsRng::default();
    let mut uid = String::with_capacity(LENGTH);
    for _ in 0..LENGTH {
        let idx = rng.gen_range(0..CHARSET.len());
        uid.push(CHARSET.as_bytes()[idx] as char);
    }
    uid
}
