use actix_web::{
    delete, post,
    web::{self, ServiceConfig},
    Error, HttpResponse,
};
use chrono::NaiveDateTime;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, Insertable, QueryDsl, Queryable,
    RunQueryDsl, Selectable, SelectableHelper,
};
use microblogs::{
    errors::ServiceError,
    generate_uid,
    schema::{self, posts::like_count},
    DbPool,
};
use serde::{Deserialize, Serialize};

use crate::users::UserDetails;

#[derive(Deserialize)]
struct PostCreate {
    parent_uuid: Option<String>,
    body: String,
}

#[derive(Deserialize)]
struct PostLikeQuery {
    uuid: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::posts)]
struct NewPost<'a> {
    pub uuid: String,
    pub parent_id: Option<i32>,
    pub poster_id: i32,
    pub body: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = schema::likes)]
struct NewLike {
    pub user_id: i32,
    pub post_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub uuid: String,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub reply_count: i32,
    pub like_count: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name=schema::likes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub created_at: NaiveDateTime,
    pub deleted: bool,
}

#[derive(Serialize)]
struct PostRead {
    uuid: String,
}

impl From<Post> for PostRead {
    fn from(post: Post) -> Self {
        Self { uuid: post.uuid }
    }
}

#[derive(Serialize)]
struct LikeRead {
    user_id: i32,
    post_id: i32,
    created_at: String,
    deleted: bool,
}

impl From<Like> for LikeRead {
    fn from(like: Like) -> Self {
        LikeRead {
            user_id: like.user_id,
            post_id: like.post_id,
            created_at: like.created_at.to_string(),
            deleted: like.deleted,
        }
    }
}

#[post("/create")]
async fn create_post(
    info: web::Json<PostCreate>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, Error> {
    use schema::posts::dsl::*;

    let post = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        let result = conn.transaction::<Post, diesel::result::Error, _>(|conn| {
            let updated_parent_id = match &info.parent_uuid {
                Some(parent_uuid) => Some(
                    diesel::update(posts)
                        .filter(uuid.eq(parent_uuid))
                        .set(reply_count.eq(reply_count + 1))
                        .returning(Post::as_returning())
                        .get_result(conn)?
                        .id,
                ),
                None => None,
            };

            let post_uuid = generate_uid();
            let new_post = NewPost {
                uuid: post_uuid,
                parent_id: updated_parent_id,
                poster_id: current_user.id,
                body: &info.body,
            };

            let post = match diesel::insert_into(posts)
                .values(&new_post)
                .returning(Post::as_returning())
                .get_result(conn)
            {
                Ok(post) => post,
                Err(_) => return Err(diesel::result::Error::RollbackTransaction),
            };

            Ok(post)
        });

        match result {
            Ok(post) => Ok(post),
            Err(_) => Err(ServiceError::BadRequest),
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(PostRead::from(post)))
}

#[post("/like")]
async fn like_post(
    post_like: web::Query<PostLikeQuery>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, Error> {
    use schema::likes::dsl::{deleted as like_deleted, likes, user_id as like_user_id};
    use schema::posts::dsl::{deleted as post_deleted, id as post_id, posts, uuid as post_uuid};

    let result = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        let like = conn.transaction::<Like, diesel::result::Error, _>(|conn| {
            if let Ok((_, post)) = likes
                .inner_join(posts)
                .filter(
                    post_uuid
                        .eq(&post_like.uuid)
                        .and(like_user_id.eq(current_user.id))
                        .and(like_deleted.eq(false)),
                )
                .select((Like::as_select(), Post::as_select()))
                .first(conn)
            {
                return Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    Box::new(format!(
                        "User {} already liked post {}",
                        current_user.username, post.uuid
                    )),
                ));
            };

            let post: Post = posts
                .filter(post_uuid.eq(&post_like.uuid))
                .select(Post::as_select())
                .first(conn)?;

            let new_like = NewLike {
                post_id: post.id,
                user_id: current_user.id,
            };

            let like = match diesel::insert_into(likes)
                .values(&new_like)
                .returning(Like::as_returning())
                .get_result(conn)
            {
                Ok(like) => {
                    diesel::update(posts)
                        .filter(post_id.eq(post.id).and(post_deleted.eq(false)))
                        .set(like_count.eq(like_count + 1))
                        .execute(conn)?;
                    like
                }
                Err(_) => return Err(diesel::result::Error::RollbackTransaction),
            };

            Ok(like)
        });

        match like {
            Ok(like) => Ok(like),
            Err(_) => Err(ServiceError::BadRequest),
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(LikeRead::from(result)))
}

#[delete("/like")]
async fn unlike_post(
    post_like: web::Query<PostLikeQuery>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, Error> {
    use schema::likes::dsl::{
        deleted as like_deleted, id as like_id, likes, user_id as like_user_id,
    };
    use schema::posts::dsl::{deleted as post_deleted, id as post_id, posts, uuid as post_uuid};

    let result = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        let like = conn.transaction::<Like, diesel::result::Error, _>(|conn| {
            // check if like exists
            let (like, post) = likes
                .inner_join(posts)
                .filter(
                    post_uuid
                        .eq(&post_like.uuid)
                        .and(like_user_id.eq(current_user.id))
                        .and(like_deleted.eq(false)),
                )
                .select((Like::as_select(), Post::as_select()))
                .first(conn)?;

            // set as deleted
            diesel::update(likes)
                .filter(like_id.eq(like.id))
                .set(like_deleted.eq(true))
                .execute(conn)?;

            // update post like count
            diesel::update(posts)
                .filter(post_id.eq(post.id).and(post_deleted.eq(false)))
                .set(like_count.eq(like_count - 1))
                .execute(conn)?;

            Ok(like)
        });

        match like {
            Ok(like) => Ok(like),
            Err(_) => Err(ServiceError::BadRequest),
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(LikeRead::from(result)))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/posts")
            .service(create_post)
            .service(like_post)
            .service(unlike_post),
    );
}
