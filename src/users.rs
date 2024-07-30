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
use diesel::{
    query_dsl::filter_dsl::FilterDsl, query_dsl::methods::SelectDsl, BoolExpressionMethods,
    ExpressionMethods, Insertable, Queryable, RunQueryDsl, Selectable, SelectableHelper,
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

pub struct UserDetails {
    pub id: i32,
    pub username: String,
    pub real_name: String,
}

impl From<User> for UserDetails {
    fn from(user: User) -> Self {
        UserDetails {
            id: user.id,
            username: user.username,
            real_name: user.real_name,
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
            None => {
                return ready(Err(ServiceError::InternalServerError(format!(
                    "Impossível conectar ao banco de dados."
                ))
                .into()))
            }
        };
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(app_state) => app_state,
            None => {
                return ready(Err(ServiceError::InternalServerError(format!(
                    "Impossível obter dados da aplicação."
                ))
                .into()))
            }
        };

        let access_token = match req.headers().get("Authorization") {
            // first, try from authorization header
            Some(header) => {
                let header_str = match header.to_str() {
                    Ok(header) => header,
                    Err(_) => {
                        return ready(Err(ServiceError::InternalServerError(format!(
                            "Falha ao converter o cabeçalho de autorização."
                        ))
                        .into()))
                    }
                };

                let bearer_token = match header_str.strip_prefix("Bearer ") {
                    Some(token) => token.to_string(),
                    None => {
                        return ready(Err(ServiceError::InternalServerError(format!(
                            "Falha ao obter a chave de autorização do valor do cabeçalho."
                        ))
                        .into()))
                    }
                };

                bearer_token
            }

            // second, try from cookies
            None => {
                let access_token_cookie = req.cookie("accessToken");
                let access_token = match access_token_cookie {
                    Some(cookie) => cookie.value().to_owned(),
                    None => {
                        return ready(Err(ServiceError::Unauthorized(format!(
                            "Acesso negado porque o cookie \"accessToken\" não foi encontrado."
                        ))
                        .into()))
                    }
                };

                access_token
            }
        };

        let secret = app_state.secret_key.clone();
        let token_data = match decode::<Claims>(
            &access_token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(token) => token,
            Err(_) => {
                return ready(Err(ServiceError::Unauthorized(format!(
                    "Não foi possível decodificar a chave."
                ))
                .into()))
            }
        };
        let username_in_session = token_data.claims.sub;

        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return ready(Err(ServiceError::InternalServerError(format!(
                    "Não foi possível conectar ao banco de dados."
                ))
                .into()))
            }
        };

        let user: User = match users
            .filter(username.eq(&username_in_session).and(deleted.eq(false)))
            .select(User::as_select())
            .first(&mut conn)
        {
            Ok(user) => user,
            Err(_) => {
                return ready(Err(ServiceError::Unauthorized(format!(
                    "Usuário \"{}\" inexistente.",
                    username_in_session
                ))
                .into()))
            }
        };

        ready(Ok(user.into()))
    }
}

#[derive(Serialize)]
struct AccessInfo {
    token: String,
    username: String,
    real_name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct User {
    pub id: i32,
    pub username: String,
    pub real_name: String,
    pub password: String,
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
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível conectar ao banco de dados."
                )));
            }
        };

        let password_bytes = info.password.as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = match argon2.hash_password(password_bytes, &salt) {
            Ok(hashed_password) => hashed_password.to_string(),
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Falha ao criptografar a senha."
                )));
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
            Err(_) => {
                return Err(ServiceError::BadRequest(format!(
                    "Falha ao registrar o novo usuário."
                )))
            }
        }
    })
    .await??;

    let claims = Claims {
        sub: user.username.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let secret = app_state.secret_key.clone();
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => {
            return Err(
                ServiceError::InternalServerError(format!("Falha ao gerar a chave.")).into(),
            )
        }
    };

    let access_info = AccessInfo {
        token,
        username: user.username,
        real_name: user.real_name,
    };
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
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Impossível conectar ao banco de dados."
                )))
            }
        };

        let user: User = match users
            .filter(username.eq(target_username.as_str()))
            .select(User::as_select())
            .first::<User>(&mut conn)
        {
            Ok(user) => user,
            Err(_) => {
                return Err(ServiceError::Unauthorized(format!(
                    "Usuário \"{}\" inexistente.",
                    target_username
                ))
                .into())
            }
        };

        Ok(user)
    })
    .await??;

    let parsed_password_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return Err(ServiceError::InternalServerError(format!(
                "Falha ao descriptografar a senha."
            ))
            .into())
        }
    };

    let argon2 = Argon2::default();
    let verified = argon2.verify_password(target_password.as_bytes(), &parsed_password_hash);

    if verified.is_err() {
        return Err(ServiceError::Unauthorized(format!("Credenciais inválidas.")).into());
    }

    let claims = Claims {
        sub: user.username.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let secret = app_state.secret_key.clone();
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => {
            return Err(ServiceError::InternalServerError(format!(
                "Falha ao gerar uma nova chave."
            ))
            .into())
        }
    };

    let access_info = AccessInfo {
        token,
        username: user.username,
        real_name: user.real_name,
    };
    Ok(HttpResponse::Ok().json(access_info))
}

#[get("/refresh_access")]
async fn refresh_access(
    current_user: UserDetails,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let claims = Claims {
        sub: current_user.username.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };
    let secret = app_state.secret_key.clone();
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => token,
        Err(_) => {
            return Err(ServiceError::InternalServerError(format!(
                "Falha ao gerar uma nova chave."
            ))
            .into())
        }
    };

    let access_info = AccessInfo {
        token,
        username: current_user.username,
        real_name: current_user.real_name,
    };
    Ok(HttpResponse::Ok().json(access_info))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(register_user)
            .service(authenticate_user)
            .service(refresh_access),
    );
}
