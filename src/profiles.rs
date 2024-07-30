use actix_web::{
    get,
    web::{self, ServiceConfig},
    Error, HttpResponse,
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, Queryable, RunQueryDsl,
    Selectable, SelectableHelper,
};
use microblogs::{errors::ServiceError, schema, DbConn, DbPool, Pagination};
use serde::Serialize;

use crate::{
    feeds::{PostRead, Poster},
    posts::{Like, Post},
    users::UserDetails,
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct Profile {
    username: String,
    real_name: String,
    summary: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Serialize)]
struct ProfileRead {
    username: String,
    real_name: String,
    summary: String,
    created_at: String,
}

#[derive(Serialize)]
struct ProfilePostsRead {
    posts: Vec<PostRead>,
}

impl From<Profile> for ProfileRead {
    fn from(profile: Profile) -> Self {
        ProfileRead {
            username: profile.username,
            real_name: profile.real_name,
            summary: profile.summary,
            created_at: profile.created_at.to_string(),
        }
    }
}

fn get_profile(target_username: &str, conn: &mut DbConn) -> Result<Profile, ServiceError> {
    use schema::users::dsl::*;

    let profile: Profile = match users
        .filter(
            username
                .eq(target_username.to_string())
                .and(deleted.eq(false)),
        )
        .select(Profile::as_select())
        .first(conn)
    {
        Ok(user) => user,
        Err(_) => {
            return Err(ServiceError::NotFound(format!(
                "Usuário \"{}\" não encontrado.",
                target_username
            )))
        }
    };

    Ok(profile)
}

#[get("/{target_username}/details")]
async fn get_profile_details(
    target_username: web::Path<String>,
    pool: web::Data<DbPool>,
    _current_user: UserDetails,
) -> Result<HttpResponse, Error> {
    let result = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Impossível conectar ao banco de dados."
                )))
            }
        };

        let profile = get_profile(&target_username, &mut conn)?;
        Ok(profile)
    })
    .await??;

    Ok(HttpResponse::Ok().json(ProfileRead::from(result)))
}

#[get("/{target_username}/posts")]
async fn get_profile_posts(
    target_username: web::Path<String>,
    pagination: web::Query<Pagination>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, Error> {
    use schema::likes::dsl::{
        deleted as like_deleted, likes, post_id as like_post_id, user_id as like_user_id,
    };
    use schema::posts::dsl::{
        created_at as post_created_at, deleted as post_deleted, id as post_id, posts,
    };
    use schema::users::dsl::{deleted as user_deleted, username, users};

    let returned_posts = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Impossível conectar ao banco de dados."
                )))
            }
        };

        match posts
            .inner_join(users)
            .left_join(
                likes.on(like_post_id
                    .eq(post_id)
                    .and(like_user_id.eq(current_user.id))
                    .and(like_deleted.eq(false))),
            )
            .filter(
                post_deleted
                    .eq(false)
                    .and(username.eq(target_username.as_str()))
                    .and(user_deleted.eq(false)),
            )
            .select((
                Post::as_select(),
                Poster::as_select(),
                Option::<Like>::as_select(),
            ))
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .order_by(post_created_at.desc())
            .load::<(Post, Poster, Option<Like>)>(&mut conn)
        {
            Ok(returned_posts) => Ok(returned_posts),
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível carregar as postagens feitas por {}.",
                    target_username.as_str()
                )))
            }
        }
    })
    .await??;

    let returned_posts: Vec<PostRead> = returned_posts
        .into_iter()
        .map(|(post, poster, like)| PostRead::from((post, poster, like)))
        .collect();

    Ok(HttpResponse::Ok().json(ProfilePostsRead {
        posts: returned_posts,
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/profiles")
            .service(get_profile_details)
            .service(get_profile_posts),
    );
}
