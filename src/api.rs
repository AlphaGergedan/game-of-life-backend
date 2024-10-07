use actix_web::{body::BoxBody, get, guard, http::header::ContentType, post, web, HttpResponse, Responder, Result};
use serde::{ Serialize, Deserialize };
use futures::future::join_all;
use crate::db::{ execute, Pool, Query, QueryResult };
use crate::AppError;

use crate::model::character::CharacterList;

mod character;
mod skill;
mod task;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .guard(guard::Host("localhost").scheme("http"))

            // CHARACTER ROUTES
            .service(character::get_characters)
            .service(character::get_character)
            .service(character::create_character)
            .service(character::update_character)
            .service(character::delete_character)
            .service(character::get_character_skills)   // FIXME
            .service(character::create_character_skill) // FIXME
            .service(character::get_character_tasks)    // FIXME

            // SKILL ROUTES
            .service(skill::get_skills)
            .service(skill::get_skill)
            .service(skill::get_skill_tasks)
            .service(skill::create_skill_task)
            .service(skill::update_skill)
            .service(skill::delete_skill)

            // TASK ROUTES
            .service(task::get_tasks)
            .service(task::get_task)
            .service(task::update_task)
            .service(task::delete_task)

            .service(reset_db)

            .service(hello)
            .service(echo)
    );
}

#[post("/reset_db")]
async fn reset_db(db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let query = Query::ResetDB;
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Success => {
            let msg = format!("db is reset");
            let res = HttpResponse::Created()
                .content_type(ContentType::plaintext())
                .body(msg);
            Ok(res)
        }
        _ => Err(AppError::DBError { error_msg: "cannot reset db /reset_db".to_string() }.into())
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
