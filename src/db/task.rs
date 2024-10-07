use actix_web::{body::BoxBody, http::header::ContentType, HttpResponse, Responder, Result};
use rusqlite::{params, Row};
use crate::{
    IdType, now,
    db::Connection,
    db::character::touch as touch_character,
    db::skill::touch as touch_skill,
    model::task::{
        TaskFields, Task, TaskList,
    },
};

impl Responder for Task {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl Responder for TaskList {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self.0).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}


pub fn get_task_list(conn: &Connection, skill_id: Option<IdType>) -> Result<TaskList, rusqlite::Error> {
    match skill_id {
        Some(skill_id) => {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, completed, skill_id, created_at, updated_at FROM task WHERE skill_id = ?1"
            )?;
            let tasks = stmt.query_map(params![skill_id], to_task).and_then(Iterator::collect)?;
            Ok(TaskList(tasks))
        },
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, completed, skill_id, created_at, updated_at FROM task"
            )?;
            let tasks = stmt.query_map([], to_task).and_then(Iterator::collect)?;
            Ok(TaskList(tasks))
        }
    }
}

pub fn get_task(conn: &Connection, id: IdType) -> Result<Task, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, completed, skill_id, created_at, updated_at FROM task WHERE id = ?1"
    )?;
    let task = stmt.query_row(params![id], to_task)?;
    Ok(task)
}

pub fn create_task(conn: &Connection, skill_id: IdType, fields: TaskFields) -> Result<(), rusqlite::Error> {
    let timestamp = now();
    let num_rows_inserted = conn.execute(
        "INSERT INTO task (name, description, completed, skill_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![fields.name, fields.description, fields.completed, skill_id, timestamp, timestamp]
    )?;
    assert_eq!(num_rows_inserted, 1);

    // update parents' updated_at attributes
    touch_skill(conn, skill_id, timestamp)?;
    let mut stmt = conn.prepare(
        "SELECT character_id FROM skill WHERE id = ?1"
    )?;
    let character_id = stmt.query_row(params![skill_id], to_id)?;
    touch_character(conn, character_id, timestamp)?;

    Ok(())
}

pub fn update_task(conn: &Connection, id: IdType, fields: TaskFields) -> Result<(), rusqlite::Error> {
    let timestamp = now();
    let num_rows_updated = conn.execute(
        "UPDATE task SET name = ?1, description = ?2, completed = ?3, updated_at = ?4 WHERE id = ?5",
        params![fields.name, fields.description, fields.completed, timestamp, id]
    )?;
    assert_eq!(num_rows_updated, 1);

    // update parents' updated_at attributes

    let mut stmt = conn.prepare(
        "SELECT skill_id FROM task WHERE id = ?1"
    )?;
    let skill_id = stmt.query_row(params![id], to_id)?;
    touch_skill(conn, skill_id, timestamp)?;

    let mut stmt = conn.prepare(
        "SELECT character_id FROM skill WHERE id = ?1"
    )?;
    let character_id = stmt.query_row(params![skill_id], to_id)?;
    touch_character(conn, character_id, timestamp)?;

    Ok(())
}

pub fn delete_task(conn: &Connection, id: IdType) -> Result<(), rusqlite::Error> {
    let timestamp = now();

    // get parents' ids before deleting for updating their updated_at attrs

    let mut stmt = conn.prepare(
        "SELECT skill_id FROM task WHERE id = ?1"
    )?;
    let skill_id = stmt.query_row(params![id], to_id)?;

    let mut stmt = conn.prepare(
        "SELECT character_id FROM skill WHERE id = ?1"
    )?;
    let character_id = stmt.query_row(params![skill_id], to_id)?;

    let num_rows_deleted = conn.execute(
        "DELETE FROM task WHERE id = ?1",
        params![id]
    )?;

    if num_rows_deleted == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    assert_eq!(num_rows_deleted, 1);

    touch_skill(conn, skill_id, timestamp)?;
    touch_character(conn, character_id, timestamp)?;

    Ok(())
}

fn to_id(row: &Row) -> Result<IdType, rusqlite::Error> {
    Ok(row.get(0)?)
}

fn to_task(row: &Row) -> Result<Task, rusqlite::Error> {
    Ok(Task {
        id: row.get(0)?,
        fields: TaskFields { name: row.get(1)?, description: row.get(2)?, completed: row.get(3)? },
        skill_id: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
