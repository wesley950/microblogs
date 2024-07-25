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
    schema::{self, posts::like_count},
    DbPool,
};
use serde::{Deserialize, Serialize};

use crate::users::UserDetails;

#[derive(Deserialize)]
struct PostCreate {
    parent_id: Option<i32>,
    body: String,
}

#[derive(Deserialize)]
struct PostLikeQuery {
    id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = schema::posts)]
struct NewPost<'a> {
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
#[diesel(table_name=schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub poster_id: i32,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub deleted: bool,
    pub reply_count: i32,
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
struct LikeRead {
    id: i32,
    user_id: i32,
    post_id: i32,
    created_at: String,
    deleted: bool,
}

impl From<Like> for LikeRead {
    fn from(like: Like) -> Self {
        LikeRead {
            id: like.id,
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
            if let Some(target_parent_id) = info.parent_id {
                match diesel::update(posts)
                    .filter(id.eq(target_parent_id))
                    .set(reply_count.eq(reply_count + 1))
                    .returning(Post::as_returning())
                    .get_result(conn)
                {
                    Ok(_) => (),
                    Err(_) => return Err(diesel::result::Error::RollbackTransaction),
                }
            };

            let new_post = NewPost {
                parent_id: info.parent_id,
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

    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/posts/{}", post.id)))
        .finish())
}

#[post("/like")]
async fn like_post(
    post_like: web::Query<PostLikeQuery>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, Error> {
    use schema::likes::dsl::{
        deleted as like_deleted, likes, post_id as like_post_id, user_id as like_user_id,
    };
    use schema::posts::dsl::{deleted as post_deleted, id as post_id, posts};

    let result = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        let like = conn.transaction::<Like, diesel::result::Error, _>(|conn| {
            // check if user has already liked the post
            match likes
                .filter(
                    like_post_id
                        .eq(post_like.id)
                        .and(like_user_id.eq(current_user.id).and(like_deleted.eq(false))),
                )
                .first::<Like>(conn)
            {
                Ok(_) => {
                    return Err(diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        Box::new(format!(
                            "User {} already liked post {}",
                            current_user.id, post_like.id
                        )),
                    ))
                }
                Err(_) => {
                    // insert like
                    let new_like = NewLike {
                        post_id: post_like.id,
                        user_id: current_user.id,
                    };

                    let like = match diesel::insert_into(likes)
                        .values(&new_like)
                        .returning(Like::as_returning())
                        .get_result(conn)
                    {
                        Ok(like) => {
                            if let Err(_) = diesel::update(posts)
                                .filter(post_id.eq(post_like.id).and(post_deleted.eq(false)))
                                .set(like_count.eq(like_count + 1))
                                .execute(conn)
                            {
                                return Err(diesel::result::Error::RollbackTransaction);
                            }

                            like
                        }
                        Err(_) => return Err(diesel::result::Error::RollbackTransaction),
                    };

                    Ok(like)
                }
            }
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
        deleted as like_deleted, id as like_id, likes, post_id as like_post_id,
        user_id as like_user_id,
    };
    use schema::posts::dsl::{deleted as post_deleted, id as post_id, posts};

    let result = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        let like = conn.transaction::<Like, diesel::result::Error, _>(|conn| {
            match likes
                .filter(
                    like_post_id
                        .eq(post_like.id)
                        .and(like_user_id.eq(current_user.id).and(like_deleted.eq(false))),
                )
                .first::<Like>(conn)
            {
                Ok(like) => {
                    let like: Like = match diesel::update(likes)
                        .filter(like_id.eq(like.id))
                        .set(like_deleted.eq(true))
                        .returning(Like::as_returning())
                        .get_result(conn)
                    {
                        Ok(like) => {
                            if let Err(_) = diesel::update(posts)
                                .filter(post_id.eq(post_like.id).and(post_deleted.eq(false)))
                                .set(like_count.eq(like_count - 1))
                                .execute(conn)
                            {
                                return Err(diesel::result::Error::RollbackTransaction);
                            }

                            like
                        }
                        Err(_) => return Err(diesel::result::Error::RollbackTransaction),
                    };
                    Ok(like)
                }
                Err(_) => return Err(diesel::result::Error::NotFound),
            }
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
