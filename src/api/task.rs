use actix_web::{
    delete, get, http::header::ContentType, put, web, HttpResponse, Responder
};
use crate::{ db::{ execute, Pool, Query, QueryResult }, model::task::TaskFields, AppError, IdType };
use crate::model::task::{ TaskList };


#[get("/tasks")]
pub async fn get_tasks(db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let query = Query::GetTaskList;
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::TaskList(task_list) => Ok(TaskList(task_list)),
        _ => Err(AppError::InternalError.into())
    }
}

#[get("/tasks/{id}")]
pub async fn get_task(path: web::Path<IdType>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    let query = Query::GetTask(id);
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Task(task) => Ok(task),
        _ => Err(AppError::InternalError.into())
    }
}

#[put("/tasks/{id}")]
pub async fn update_task(path: web::Path<IdType>, form: web::Form<TaskFields>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let task_id = path.into_inner();
    let query = Query::UpdateTask(task_id, form.into_inner());
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Task(task) => Ok(task),
        _ => Err(AppError::InternalError.into())
    }
}

#[delete("/tasks/{id}")]
pub async fn delete_task(path: web::Path<IdType>, db: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let task_id = path.into_inner();
    let query = Query::DeleteTask(task_id);
    let query_result = execute(&db, query).await?;
    match query_result {
        QueryResult::Success => {
            let msg = format!("Task with id {} is deleted", task_id);
            let res = HttpResponse::Ok()
                .content_type(ContentType::plaintext())
                .body(msg);
            Ok(res)
        },
        _ => Err(AppError::InternalError.into())
    }
}
