use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse,
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, Queryable, RunQueryDsl,
    Selectable, SelectableHelper,
};
use microblogs::{errors::ServiceError, schema, DbPool, Pagination};
use serde::Serialize;

use crate::{
    posts::{Like, Post},
    users::UserDetails,
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Poster {
    pub username: String,
    pub real_name: String,
}

#[derive(Serialize)]
pub struct PosterRead {
    username: String,
    real_name: String,
}

#[derive(Serialize)]
pub struct PostRead {
    uuid: String,
    body: String,
    created_at: String,
    reply_count: i32,
    like_count: i32,
    liked_by_user: bool,
    poster: PosterRead,
}

impl From<(Post, Poster, Option<Like>)> for PostRead {
    fn from((post, poster, like): (Post, Poster, Option<Like>)) -> Self {
        Self {
            uuid: post.uuid,
            body: post.body,
            created_at: post.created_at.to_string(),
            reply_count: post.reply_count,
            like_count: post.like_count,
            liked_by_user: like.is_some(),
            poster: PosterRead {
                username: poster.username,
                real_name: poster.real_name,
            },
        }
    }
}

#[derive(Serialize)]
struct FeedRead {
    posts: Vec<PostRead>,
}

#[derive(Serialize)]
struct RepliesRead {
    replies: Vec<PostRead>,
}

#[get("/list")]
async fn get_feed(
    pagination: web::Query<Pagination>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::likes::dsl::{
        deleted as like_deleted, likes, post_id as like_post_id, user_id as like_user_id,
    };
    use schema::posts::dsl::{
        created_at as post_created_at, deleted as post_deleted, id as post_id,
        parent_id as post_parent_id, posts,
    };
    use schema::users::dsl::users;

    let returned_posts = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível conectar ao banco de dados."
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
            .filter(post_deleted.eq(false).and(post_parent_id.is_null()))
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
                    "Não foi possível carregar as postagens."
                )))
            }
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(FeedRead {
        posts: returned_posts
            .into_iter()
            .map(|(post, poster, like)| PostRead::from((post, poster, like)))
            .collect(),
    }))
}

#[get("/details/{target_post_uuid}")]
async fn get_post_details(
    target_post_uuid: web::Path<String>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::likes::dsl::{
        deleted as like_deleted, likes, post_id as like_post_id, user_id as like_user_id,
    };
    use schema::posts::dsl::{deleted as post_deleted, id as post_id, posts, uuid as post_uuid};
    use schema::users::dsl::users;

    let ((post, poster), like) = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível conectar ao banco de dados."
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
                    .and(post_uuid.eq(target_post_uuid.as_str())),
            )
            .select((
                Post::as_select(),
                Poster::as_select(),
                Option::<Like>::as_select(),
            ))
            .first(&mut conn)
        {
            Ok((post, poster, like)) => Ok(((post, poster), like)),
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível carregar a postagem {}.",
                    target_post_uuid
                )))
            }
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(PostRead::from((post, poster, like))))
}

#[get("/replies/{target_post_uuid}")]
async fn get_replies(
    target_post_uuid: web::Path<String>,
    pagination: web::Query<Pagination>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::likes::dsl::{
        deleted as like_deleted, likes, post_id as like_post_id, user_id as like_user_id,
    };
    use schema::posts::dsl::{
        created_at, deleted as post_deleted, id as post_id, parent_id, posts, uuid as post_uuid,
    };
    use schema::users::dsl::users;

    let returned_posts = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível conectar ao banco de dados."
                )))
            }
        };

        let target_parent_id = match posts
            .filter(
                post_uuid
                    .eq(target_post_uuid.as_str())
                    .and(post_deleted.eq(false)),
            )
            .select(Post::as_select())
            .first(&mut conn)
        {
            Ok(post) => post.id,
            Err(_) => {
                return Err(ServiceError::NotFound(format!(
                    "Postagem \"{}\" não encontrada.",
                    target_post_uuid
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
            .filter(post_deleted.eq(false).and(parent_id.eq(target_parent_id)))
            .select((
                Post::as_select(),
                Poster::as_select(),
                Option::<Like>::as_select(),
            ))
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .order_by(created_at.asc())
            .load::<(Post, Poster, Option<Like>)>(&mut conn)
        {
            Ok(returned_posts) => Ok(returned_posts),
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível carregar as respostas da postagem {}.",
                    target_post_uuid
                )))
            }
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(RepliesRead {
        replies: returned_posts
            .into_iter()
            .map(|(post, poster, like)| PostRead::from((post, poster, like)))
            .collect(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/feeds")
            .service(get_feed)
            .service(get_post_details)
            .service(get_replies),
    );
}
