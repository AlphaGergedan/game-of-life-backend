use actix_web::http::header::ContentType;
use actix_web::{
    delete, get, post, put,
    web, HttpResponse, Responder
};
use crate::{ AppError, IdType };
use crate::db::{ execute, Pool, Query, QueryResult };
use crate::model::character::{ CharacterList, CharacterFields };
use crate::model::skill::{ SkillFields, SkillList };
use crate::model::task::TaskList;

#[get("/characters/{id}")]
pub async fn get_character(path: web::Path<IdType>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    // using string and parsing manually like below, we can use custom err types?
    //let id: IdType = path.into_inner().parse().map_err(|_| AppError::ValidationError { field: "id".to_string() })?;
    let id = path.into_inner();
    let query_result = execute(&db, Query::GetCharacter(id)).await?;
    match query_result {
        QueryResult::Character(character) => Ok(character),
        _ => Err(AppError::InternalError.into())
    }
}

#[get("/characters")]
pub async fn get_characters(db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let query_result = execute(&db, Query::GetCharacterList).await?;
    match query_result {
        QueryResult::CharacterList(character_list) => Ok(CharacterList(character_list)),
        _ => Err(AppError::InternalError.into())
    }
}

#[get("/characters/{id}/skills")]
pub async fn get_character_skills(path: web::Path<String>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id: IdType = path.into_inner().parse().map_err(|_| AppError::ValidationError { field: "id".to_string() })?;
    let query_result = execute(&db, Query::GetCharacterSkillList(id)).await?;
    match query_result {
        QueryResult::SkillList(skill_list) => Ok(SkillList(skill_list)),
        _ => Err(AppError::InternalError.into())
    }
}

#[post("/characters/{id}/skills")]
pub async fn create_character_skill(path: web::Path<IdType>, form: web::Form<SkillFields>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    let query = Query::CreateCharacterSkill(id, form.into_inner());
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Skill(skill) => Ok(skill),
        _ => Err(AppError::InternalError.into())
    }
}

#[get("/characters/{id}/tasks")]
pub async fn get_character_tasks(path: web::Path<String>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id: IdType = path.into_inner().parse().map_err(|_| AppError::ValidationError { field: "id".to_string() })?;
    let query_result = execute(&db, Query::GetCharacterTaskList(id)).await?;
    match query_result {
        QueryResult::TaskList(task_list) => Ok(TaskList(task_list)),
        _ => Err(AppError::InternalError.into())
    }
}

#[post("/characters")]
pub async fn create_character(form: web::Form<CharacterFields>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let form = form.into_inner();
    let query_result = execute(&db, Query::CreateCharacter(form)).await?;
    match query_result {
        QueryResult::Character(character) => Ok(character),
        _ => Err(AppError::InternalError.into())
    }
}

#[put("/characters/{id}")]
pub async fn update_character(path: web::Path<String>, form: web::Form<CharacterFields>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id: IdType = path.into_inner().parse().map_err(|_| AppError::ValidationError { field: "id".to_string() })?;
    let query_result = execute(&db, Query::UpdateCharacter(id, form.into_inner())).await?;
    match query_result {
        QueryResult::Character(character) => Ok(character),
        _ => Err(AppError::InternalError.into())
    }
}

#[delete("/characters/{id}")]
pub async fn delete_character(path: web::Path<String>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id: IdType = path.into_inner().parse().map_err(|_| AppError::ValidationError { field: "id".to_string() })?;
    let query_result = execute(&db, Query::DeleteCharacter(id)).await?;
    match query_result {
        QueryResult::Success => {
            let msg = format!("Character with id {} is deleted", id);
            let res = HttpResponse::Ok()
                .content_type(ContentType::plaintext())
                .body(msg);
            Ok(res)
        },
        _ => Err(AppError::InternalError.into())
    }
}
