use actix_web::{
    body::BoxBody, http::header::ContentType, HttpResponse,
    Responder, Result
};
use rusqlite::{ params, Row };
use crate::{
    IdType, now, TimeType,
    db::Connection,
    db::character::touch as touch_character,
    model::skill::{
        SkillFields, Skill, SkillList,
    },
};

impl Responder for Skill {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl Responder for SkillList {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self.0).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

pub fn get_skill_list(conn: &Connection, character_id: Option<IdType>) -> Result<SkillList, rusqlite::Error> {
    match character_id {
        Some(character_id) => {
            let mut stmt = conn.prepare(
                "SELECT id, name, progress, level, character_id, created_at, updated_at FROM skill WHERE character_id = ?1",
            )?;
            let skills = stmt
                .query_map(params![character_id], to_skill)
                .and_then(Iterator::collect)?;
            Ok(SkillList(skills))
        },
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, name, progress, level, character_id, created_at, updated_at FROM skill",
            )?;
            let skills = stmt
                .query_map(params![], to_skill)
                .and_then(Iterator::collect)?;
            Ok(SkillList(skills))
        },
    }
}

pub fn get_skill(conn: &Connection, id: IdType) -> Result<Skill, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, progress, level, character_id, created_at, updated_at FROM skill WHERE id = ?1",
    )?;
    let skill = stmt.query_row(params![id], to_skill)?;
    Ok(skill)
}

pub fn create_skill(conn: &Connection, character_id: IdType, fields: SkillFields) -> Result<(), rusqlite::Error> {
    let timestamp = now();
    let num_rows_inserted = conn.execute(
        "INSERT INTO skill (name, progress, level, character_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![fields.name, fields.progress, fields.level, character_id, timestamp, timestamp]
    )?;
    assert_eq!(num_rows_inserted, 1);

    // update parent's updated_at attribute
    touch_character(conn, character_id, timestamp)?;

    Ok(())
}

pub fn update_skill(conn: &Connection, id: IdType, fields: SkillFields) -> Result<(), rusqlite::Error> {
    let timestamp = now();
    let num_rows_updated = conn.execute(
        "UPDATE skill SET name = ?1, progress = ?2, level = ?3, updated_at = ?4 WHERE id = ?5",
        params![fields.name, fields.progress, fields.level, timestamp, id],
    )?;

    if num_rows_updated == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    assert_eq!(num_rows_updated, 1);

    // update parent's updated_at attribute
    let mut stmt = conn.prepare(
        "SELECT character_id FROM skill WHERE id = ?1",
    )?;
    let character_id = stmt.query_row(params![id], to_id)?;
    touch_character(&conn, character_id, timestamp)?;

    Ok(())
}

pub fn delete_skill(conn: &Connection, id: IdType) -> Result<(), rusqlite::Error> {
    let timestamp = now();

    // get parent ids before deleting
    let mut stmt = conn.prepare(
        "SELECT character_id FROM skill WHERE id = ?1",
    )?;
    let character_id = stmt.query_row(params![id], to_id)?;

    let num_rows_deleted = conn.execute(
        "DELETE FROM skill WHERE id = ?1",
        params![id]
    )?;
    if num_rows_deleted == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    assert_eq!(num_rows_deleted, 1);

    touch_character(&conn, character_id, timestamp)?;

    Ok(())
}

pub fn touch(conn: &Connection, id: IdType, timestamp: TimeType) -> Result<(), rusqlite::Error> {
    let num_rows_updated = conn.execute(
        "UPDATE skill SET updated_at = ?1 WHERE id = ?2",
        params![timestamp, id]
    )?;
    assert_eq!(num_rows_updated, 1);
    Ok(())
}

fn to_skill(row: &Row) -> Result<Skill, rusqlite::Error> {
    Ok(Skill {
        id: row.get(0)?,
        fields: SkillFields { name: row.get(1)?, progress: row.get(2)?, level: row.get(3)? },
        character_id: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn to_id(row: &Row) -> Result<IdType, rusqlite::Error> {
    Ok(row.get(0)?)
}
