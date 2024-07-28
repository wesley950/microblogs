use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse,
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, Queryable, RunQueryDsl,
    Selectable, SelectableHelper,
};
use microblogs::{errors::ServiceError, schema, DbPool};
use serde::{Deserialize, Serialize};

use crate::{
    posts::{Like, Post},
    users::UserDetails,
};

#[derive(Deserialize)]
struct Pagination {
    offset: i32,
    limit: i32,
}

#[derive(Deserialize)]
struct TargetPostQuery {
    uuid: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct Poster {
    pub username: String,
    pub real_name: String,
}

#[derive(Serialize)]
struct PosterRead {
    username: String,
    real_name: String,
}

#[derive(Serialize)]
struct PostRead {
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
    parent: PostRead,
    replies: Vec<PostRead>,
}

#[get("/")]
async fn get_feed(
    pagination: web::Query<Pagination>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, actix_web::Error> {
    use diesel::prelude::*;
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
            Err(_) => return Err(ServiceError::InternalServerError),
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
            Err(_) => return Err(ServiceError::InternalServerError),
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

#[get("/replies")]
async fn get_replies(
    target_post: web::Query<TargetPostQuery>,
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

    let ((parent_post, poster, like), returned_posts) = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        let (parent_post, poster, like): (Post, Poster, Option<Like>) = match posts
            .inner_join(users)
            .left_join(
                likes.on(like_post_id
                    .eq(post_id)
                    .and(like_user_id.eq(current_user.id))
                    .and(like_deleted.eq(false))),
            )
            .filter(post_uuid.eq(&target_post.uuid).and(post_deleted.eq(false)))
            .select((
                Post::as_select(),
                Poster::as_select(),
                Option::<Like>::as_select(),
            ))
            .first::<(Post, Poster, Option<Like>)>(&mut conn)
        {
            Ok((post, poster, like)) => (post, poster, like),
            Err(_) => return Err(ServiceError::NotFound),
        };

        match posts
            .inner_join(users)
            .left_join(
                likes.on(like_post_id
                    .eq(post_id)
                    .and(like_user_id.eq(current_user.id))
                    .and(like_deleted.eq(false))),
            )
            .filter(post_deleted.eq(false).and(parent_id.eq(parent_post.id)))
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
            Ok(returned_posts) => Ok(((parent_post, poster, like), returned_posts)),
            Err(_) => return Err(ServiceError::InternalServerError),
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(RepliesRead {
        parent: PostRead::from((parent_post, poster, like)),
        replies: returned_posts
            .into_iter()
            .map(|(post, poster, like)| PostRead::from((post, poster, like)))
            .collect(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/feeds").service(get_feed).service(get_replies));
}
