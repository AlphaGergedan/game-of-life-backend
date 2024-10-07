use actix_web::http::header::ContentType;
use actix_web::{
    delete, get, post, put,
    web, HttpResponse, Responder,
};
use crate::db::{ execute, Pool, Query, QueryResult, };
use crate::model::skill::{ SkillFields, SkillList };
use crate::model::task::{TaskFields, TaskList};
use crate::{AppError, IdType};

#[get("/skills")]
pub async fn get_skills(db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let query_result = execute(&db, Query::GetSkillList).await?;
    match query_result {
        QueryResult::SkillList(skill_list) => Ok(SkillList(skill_list)),
        _ => Err(AppError::InternalError.into())
    }
}

#[get("/skills/{id}")]
pub async fn get_skill(path: web::Path<IdType>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    let query_result = execute(&db, Query::GetSkill(id)).await?;
    match query_result {
        QueryResult::Skill(skill) => Ok(skill),
        _ => Err(AppError::InternalError.into())
    }
}

#[get("/skills/{id}/tasks")]
pub async fn get_skill_tasks(path: web::Path<IdType>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    let query = Query::GetSkillTaskList(id);
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::TaskList(task_list) => Ok(TaskList(task_list)),
        _ => Err(AppError::InternalError.into())
    }
}

#[post("/skills/{id}/tasks")]
pub async fn create_skill_task(path: web::Path<IdType>, form: web::Form<TaskFields>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    let query = Query::CreateSkillTask(id, form.into_inner());
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Task(task) => Ok(task),
        _ =>  Err(AppError::InternalError.into())
    }
}

#[put("/skills/{id}")]
pub async fn update_skill(path: web::Path<IdType>, form: web::Form<SkillFields>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    let query = Query::UpdateSkill(id, form.into_inner());
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Skill(skill) => Ok(skill),
        _ => Err(AppError::InternalError.into())
    }
}

#[delete("/skills/{id}")]
pub async fn delete_skill(path: web::Path<IdType>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    let query = Query::DeleteSkill(id);
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Success => {
            let msg = format!("Skill with id {} is deleted", id);
            let res = HttpResponse::Ok()
                .content_type(ContentType::plaintext())
                .body(msg);
            Ok(res)
        }
        _ => Err(AppError::InternalError.into())
    }
}
