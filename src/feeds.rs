use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use microblogs::{schema, DbPool};
use serde::{Deserialize, Serialize};

use crate::{posts::Post, users::UserDetails};

#[derive(Deserialize)]
struct Pagination {
    offset: i32,
    limit: i32,
}

#[derive(Serialize)]
struct PostRead {
    id: i32,
    body: String,
    created_at: String,
}

impl From<Post> for PostRead {
    fn from(post: Post) -> Self {
        PostRead {
            id: post.id,
            body: post.body,
            created_at: post.created_at.to_string(),
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
    _current_user: UserDetails,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::posts::dsl::*;

    let results = web::block(move || {
        let mut conn = pool.get().unwrap();

        let results: Result<Vec<Post>, diesel::result::Error> = posts
            .filter(deleted.eq(false))
            .select(Post::as_select())
            .offset(pagination.offset as i64)
            .limit(pagination.limit as i64)
            .order_by(created_at.desc())
            .load::<Post>(&mut conn);

        results
    })
    .await?;

    match results {
        Ok(returned_posts) => Ok(HttpResponse::Ok().json(FeedRead {
            posts: returned_posts.into_iter().map(|post| post.into()).collect(),
        })),
        Err(err) => Ok(HttpResponse::InternalServerError().body(err.to_string())),
    }
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/feeds").service(get_feed));
}
