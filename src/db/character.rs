use actix_web::{
    body::BoxBody, http::header::ContentType, HttpResponse,
    Responder, Result
};
use rusqlite::{ params, Row };
use crate::{
    IdType, now, TimeType,
    db::Connection,
    model::character::{
        CharacterFields, Character, CharacterList,
    },
};

impl Responder for Character {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

impl Responder for CharacterList {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self.0).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

pub fn get_character_list(conn: &Connection) -> Result<CharacterList, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, avatar, notes, quote, created_at, updated_at FROM character"
    )?;
    let characters = stmt
        .query_map([], to_character)
        .and_then(Iterator::collect)?;
    Ok(CharacterList(characters))
}

pub fn get_character(conn: &Connection, id: IdType) -> Result<Character, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, avatar, notes, quote, created_at, updated_at FROM character WHERE id = ?1"
    )?;
    let character = stmt.query_row(params![id], to_character)?;
    Ok(character)
}

pub fn create_character(conn: &Connection, fields: CharacterFields) -> Result<(), rusqlite::Error> {
    let timestamp = now();
    let num_rows_inserted = conn.execute(
        "INSERT INTO character (name, avatar, notes, quote, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![fields.name, fields.avatar, fields.notes, fields.quote, timestamp, timestamp],
    )?;
    assert_eq!(num_rows_inserted, 1);
    Ok(())
}

pub fn update_character(conn: &Connection, id: IdType, fields: CharacterFields) -> Result<(), rusqlite::Error> {
    let timestamp = now();
    let num_rows_updated = conn.execute(
        "UPDATE character SET name = ?1, avatar = ?2, notes = ?3, quote = ?4, updated_at = ?5 WHERE id = ?6",
        params![fields.name, fields.avatar, fields.notes, fields.quote, timestamp, id],
    )?;
    assert_eq!(num_rows_updated, 1);
    Ok(())
}

pub fn touch(conn: &Connection, id: IdType, timestamp: TimeType) -> Result<(), rusqlite::Error> {
    let num_rows_updated = conn.execute(
        "UPDATE character SET updated_at = ?1 WHERE id = ?2",
        params![timestamp, id]
    )?;
    assert_eq!(num_rows_updated, 1);
    Ok(())
}

pub fn delete_character(conn: &Connection, id: IdType) -> Result<(), rusqlite::Error> {
    let num_rows_deleted = conn.execute(
        "DELETE FROM character WHERE id = ?1",
        params![id]
    )?;
    if num_rows_deleted == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

fn to_character(row: &Row) -> Result<Character, rusqlite::Error> {
    Ok(Character {
        id: row.get(0)?,
        fields: CharacterFields { name: row.get(1)?, avatar: row.get(2)?, notes: row.get(3)?, quote: row.get(4)? },
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
