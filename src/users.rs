use std::future::{ready, Ready};

use actix_web::{
    get, post,
    web::{self, ServiceConfig},
    Error, FromRequest, HttpRequest, HttpResponse,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::NaiveDateTime;
use diesel::{
    query_dsl::filter_dsl::FilterDsl, ExpressionMethods, Insertable, Queryable, RunQueryDsl,
    Selectable, SelectableHelper,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use microblogs::{errors::ServiceError, schema, AppState, DbPool};

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

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Serialize)]
pub struct UserDetails {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub real_name: String,
    pub summary: String,
    pub created_at: String,
    pub deleted: bool,
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
            deleted: user.deleted,
        }
    }
}

impl FromRequest for UserDetails {
    type Error = Error;
    type Future = Ready<Result<UserDetails, Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        use schema::users::dsl::*;

        let pool = match req.app_data::<web::Data<DbPool>>() {
            Some(pool) => pool,
            None => return ready(Err(ServiceError::InternalServerError.into())),
        };
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(app_state) => app_state,
            None => return ready(Err(ServiceError::InternalServerError.into())),
        };

        let authorization_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => return ready(Err(ServiceError::Unauthorized.into())),
        };

        let authorization_header = match authorization_header.to_str() {
            Ok(header) => header,
            Err(_) => return ready(Err(ServiceError::InternalServerError.into())),
        };

        let bearer_token = match authorization_header.strip_prefix("Bearer ") {
            Some(token) => token,
            None => return ready(Err(ServiceError::InternalServerError.into())),
        };

        let secret = app_state.secret_key.clone();
        let token_data = match decode::<Claims>(
            bearer_token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(token) => token,
            Err(_) => return ready(Err(ServiceError::Unauthorized.into())),
        };
        let username_in_session = token_data.claims.sub;

        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return ready(Err(ServiceError::InternalServerError.into())),
        };

        let user: User = match users
            .filter(username.eq(username_in_session))
            .first(&mut conn)
        {
            Ok(user) => user,
            Err(_) => return ready(Err(ServiceError::Unauthorized.into())),
        };

        ready(Ok(user.into()))
    }
}

#[derive(Serialize)]
struct AccessInfo {
    token: String,
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

#[post("/register")]
async fn register_user(
    info: web::Json<UserRegister>,
    pool: web::Data<DbPool>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::users::dsl::*;

    let user = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError);
            }
        };

        let password_bytes = info.password.as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = match argon2.hash_password(password_bytes, &salt) {
            Ok(hashed_password) => hashed_password.to_string(),
            Err(_) => {
                return Err(ServiceError::InternalServerError);
            }
        };
        let new_user = NewUser {
            username: &info.username,
            email: &info.email, // TODO: validate email
            real_name: &info.real_name,
            summary: &info.summary,
            password: &hashed_password,
        };

        match diesel::insert_into(users)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
        {
            Ok(user) => return Ok(user),
            Err(_) => return Err(ServiceError::BadRequest),
        }
    })
    .await??;

    let claims = Claims {
        sub: user.username,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let secret = app_state.secret_key.clone();
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => return Err(ServiceError::InternalServerError.into()),
    };

    let access_info = AccessInfo { token };
    Ok(HttpResponse::Ok().json(access_info))
}

#[post("/login")]
async fn authenticate_user(
    info: web::Json<UserLogin>,
    pool: web::Data<DbPool>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::users::dsl::*;

    let (target_username, target_password) = (info.username.clone(), info.password.clone());

    let user = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        let user: User = match users
            .filter(username.eq(target_username.as_str()))
            .first(&mut conn)
        {
            Ok(user) => user,
            Err(_) => return Err(ServiceError::Unauthorized),
        };

        Ok(user)
    })
    .await??;

    let parsed_password_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => return Err(ServiceError::InternalServerError.into()),
    };

    let argon2 = Argon2::default();
    let verified = argon2.verify_password(target_password.as_bytes(), &parsed_password_hash);

    if verified.is_err() {
        return Err(ServiceError::Unauthorized.into());
    }

    let claims = Claims {
        sub: user.username,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let secret = app_state.secret_key.clone();
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => return Err(ServiceError::InternalServerError.into()),
    };

    let access_info = AccessInfo { token };
    Ok(HttpResponse::Ok().json(access_info))
}

#[get("/refresh_access")]
async fn refresh_access(
    current_user: UserDetails,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = Claims {
        sub: current_user.username,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let secret = app_state.secret_key.clone();
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => return Err(ServiceError::InternalServerError.into()),
    };

    let access_info = AccessInfo { token };
    Ok(HttpResponse::Ok().json(access_info))
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
            .service(refresh_access)
            .service(get_details),
    );
}
