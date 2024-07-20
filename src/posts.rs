use actix_web::{
    error::ErrorInternalServerError,
    post,
    web::{self, ServiceConfig},
    Error, HttpResponse,
};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, RunQueryDsl, Selectable, SelectableHelper};
use microblogs::{schema, DbPool};
use serde::Deserialize;

use crate::users::UserDetails;

#[derive(Deserialize)]
struct PostCreate {
    body: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::posts)]
struct NewPost<'a> {
    pub poster_id: i32,
    pub body: &'a str,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name=schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub body: String,
    pub created_at: NaiveDateTime,
}

#[post("/create")]
async fn create_post(
    info: web::Json<PostCreate>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, Error> {
    use schema::posts::dsl::*;

    let post = web::block(move || {
        let mut conn = pool.get().unwrap();

        let new_post = NewPost {
            poster_id: current_user.id,
            body: &info.body,
        };

        diesel::insert_into(posts)
            .values(&new_post)
            .returning(Post::as_returning())
            .get_result(&mut conn)
    })
    .await?;

    match post {
        Ok(post) => Ok(HttpResponse::Created()
            .append_header(("Location", format!("/posts/{}", post.id)))
            .finish()),
        Err(err) => Err(ErrorInternalServerError(err.to_string())),
    }
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/posts").service(create_post));
}
