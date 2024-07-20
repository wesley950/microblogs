use std::{
    fmt::Display,
    future::{ready, Ready},
};

use actix_identity::Identity;
use actix_web::{
    get, post,
    web::{self, ServiceConfig},
    Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, ResponseError,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error::NotFound};
use serde::{Deserialize, Serialize};

use microblogs::{schema, DbPool};

#[derive(Deserialize)]
struct UserRegister {
    username: String,
    email: String,
    real_name: String,
    #[serde(default = "String::new")]
    summary: String,
    password: String,
}

#[derive(Deserialize)]
struct UserLogin {
    username: String,
    password: String,
}

#[derive(Debug)]
enum ServiceError {
    InternalServerError,
    Unauthorized,
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::InternalServerError => write!(f, "Internal server error"),
            ServiceError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError => HttpResponse::InternalServerError().finish(),
            ServiceError::Unauthorized => HttpResponse::Unauthorized().finish(),
        }
    }
}

#[derive(Serialize)]
pub struct UserDetails {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub real_name: String,
    pub summary: String,
    pub created_at: String,
}

impl From<User> for UserDetails {
    fn from(user: User) -> Self {
        UserDetails {
            id: user.id,
            username: user.username,
            email: user.email,
            real_name: user.real_name,
            summary: user.summary,
            created_at: user.created_at.to_string(),
        }
    }
}

impl FromRequest for UserDetails {
    type Error = Error;
    type Future = Ready<Result<UserDetails, Error>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        use schema::users::dsl::*;

        let pool = req.app_data::<web::Data<DbPool>>().unwrap();
        let identity = Identity::from_request(&req, payload).into_inner();

        if identity.is_err() {
            return ready(Err(ServiceError::InternalServerError.into()));
        }

        let identity = identity.unwrap();
        let username_in_session = identity.id();

        if username_in_session.is_err() {
            return ready(Err(ServiceError::InternalServerError.into()));
        }

        let username_in_session = username_in_session.unwrap();
        let conn = pool.get();

        if conn.is_err() {
            return ready(Err(ServiceError::InternalServerError.into()));
        }

        let mut conn = conn.unwrap();
        let user: Result<User, diesel::result::Error> = users
            .filter(username.eq(username_in_session))
            .first(&mut conn);

        match user {
            Ok(user) => ready(Ok(user.into())),
            Err(_) => ready(Err(ServiceError::Unauthorized.into())),
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub real_name: String,
    pub summary: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub deleted: bool,
}

#[derive(Insertable)]
#[diesel(table_name = schema::users)]
struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub real_name: &'a str,
    pub summary: &'a str,
    pub password: &'a str,
}

fn hash_password(password: &str) -> String {
    let password_bytes = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(password_bytes, &salt)
        .unwrap()
        .to_string();
    hashed_password
}

#[post("/register")]
async fn register_user(
    request: HttpRequest,
    info: web::Json<UserRegister>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::users::dsl::*;

    let user = web::block(move || {
        let mut conn = pool.get().unwrap();

        let hashed_password = hash_password(&info.password);
        let new_user = NewUser {
            username: &info.username,
            email: &info.email, // TODO: validate email
            real_name: &info.real_name,
            summary: &info.summary,
            password: &hashed_password,
        };

        diesel::insert_into(users)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
    })
    .await?;

    match user {
        Ok(user) => {
            Identity::login(&mut request.extensions(), user.username.clone()).unwrap();
            let details: UserDetails = user.into();
            Ok(HttpResponse::Created().json(details))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().body(err.to_string())),
    }
}

#[post("/login")]
async fn authenticate_user(
    request: HttpRequest,
    info: web::Json<UserLogin>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::users::dsl::*;

    let (target_username, target_password) = (info.username.clone(), info.password.clone());

    let user = web::block(move || {
        let mut conn = pool.get().unwrap();

        let user: Result<User, diesel::result::Error> = users
            .filter(username.eq(target_username.as_str()))
            .first(&mut conn);

        if user.is_err() {
            return Err(NotFound);
        }

        user
    })
    .await?;

    if user.is_err() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let user = user.unwrap();
    let parsed_hash = PasswordHash::new(&user.password);

    if parsed_hash.is_err() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let argon2 = Argon2::default();
    let parsed_hash = parsed_hash.unwrap();
    let result = argon2.verify_password(target_password.as_bytes(), &parsed_hash);

    if result.is_err() || user.deleted {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    Identity::login(&mut request.extensions(), user.username.clone()).unwrap();
    let details: UserDetails = user.into();
    Ok(HttpResponse::Found().json(details))
}

#[get("/me")]
async fn get_details(current_user: UserDetails) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().json(current_user))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(register_user)
            .service(authenticate_user)
            .service(get_details),
    );
}
