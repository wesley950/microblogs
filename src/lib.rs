use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub mod schema;
