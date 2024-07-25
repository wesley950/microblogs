use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse,
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
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
    id: i32,
}

#[derive(Serialize)]
struct PostRead {
    id: i32,
    body: String,
    created_at: String,
    liked_by_user: bool,
}

impl From<(Post, Option<Like>)> for PostRead {
    fn from((post, like): (Post, Option<Like>)) -> Self {
        Self {
            id: post.id,
            body: post.body,
            created_at: post.created_at.to_string(),
            liked_by_user: like.is_some(),
        }
    }
}

#[derive(Serialize)]
struct FeedRead {
    posts: Vec<PostRead>,
}

#[get("/")]
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

    let returned_posts = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        match posts
            .left_join(
                likes.on(like_post_id
                    .eq(post_id)
                    .and(like_user_id.eq(current_user.id))
                    .and(like_deleted.eq(false))),
            )
            .filter(post_deleted.eq(false).and(post_parent_id.is_null()))
            .select((Post::as_select(), Option::<Like>::as_select()))
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .order_by(post_created_at.desc())
            .load::<(Post, Option<Like>)>(&mut conn)
        {
            Ok(returned_posts) => Ok(returned_posts),
            Err(_) => return Err(ServiceError::InternalServerError),
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(FeedRead {
        posts: returned_posts
            .into_iter()
            .map(|(post, like)| PostRead::from((post, like)))
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
        created_at, deleted as post_deleted, id as post_id, parent_id, posts,
    };

    let returned_posts = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => return Err(ServiceError::InternalServerError),
        };

        match posts
            .left_join(
                likes.on(like_post_id
                    .eq(post_id)
                    .and(like_user_id.eq(current_user.id))
                    .and(like_deleted.eq(false))),
            )
            .filter(post_deleted.eq(false).and(parent_id.eq(target_post.id)))
            .select((Post::as_select(), Option::<Like>::as_select()))
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .order_by(created_at.asc())
            .load::<(Post, Option<Like>)>(&mut conn)
        {
            Ok(returned_posts) => Ok(returned_posts),
            Err(_) => return Err(ServiceError::InternalServerError),
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(FeedRead {
        posts: returned_posts
            .into_iter()
            .map(|(post, like)| PostRead::from((post, like)))
            .collect(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/feeds").service(get_feed).service(get_replies));
}
