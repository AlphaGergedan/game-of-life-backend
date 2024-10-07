use std::{thread::sleep, time::Duration};
use actix_web::web;
use serde::Serialize;
use r2d2_sqlite::SqliteConnectionManager;
use crate::model::character::{Character, CharacterFields, CharacterList};
use crate::model::skill::{Skill, SkillFields, SkillList};
use crate::model::task::{Task, TaskFields, TaskList};
use crate::{ AppError, IdType};

pub mod character;
pub mod skill;
pub mod task;

use character::{get_character_list, get_character, create_character, update_character, delete_character};
use skill::{get_skill_list, get_skill, create_skill, update_skill, delete_skill};
use task::{get_task_list, get_task, create_task, update_task, delete_task};

pub type Pool = r2d2::Pool<SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

pub enum Query {
    GetCharacterList,
    GetCharacter(IdType),
    GetCharacterSkillList(IdType),
    CreateCharacterSkill(IdType, SkillFields),   // IdType: character_id
    GetCharacterTaskList(IdType),
    CreateCharacter(CharacterFields),
    UpdateCharacter(IdType, CharacterFields),
    DeleteCharacter(IdType),

    GetSkillList,
    GetSkill(IdType),
    GetSkillTaskList(IdType),
    CreateSkillTask(IdType, TaskFields),     // IdType: skill_id
    UpdateSkill(IdType, SkillFields),
    DeleteSkill(IdType),

    GetTaskList,
    GetTask(IdType),
    UpdateTask(IdType, TaskFields),
    DeleteTask(IdType),

    ResetDB,
}

#[derive(Serialize)]
pub enum QueryResult {
    CharacterList(Vec<Character>),
    Character(Character),
    SkillList(Vec<Skill>),
    Skill(Skill),
    TaskList(Vec<Task>),
    Task(Task),

    Success,
}

impl From::<CharacterList> for QueryResult {
    fn from(list: CharacterList) -> Self {
        QueryResult::CharacterList(list.0)
    }
}

impl From::<Character> for QueryResult {
    fn from(character: Character) -> Self {
        QueryResult::Character(character)
    }
}

impl From::<SkillList> for QueryResult {
    fn from(list: SkillList) -> Self {
        QueryResult::SkillList(list.0)
    }
}

impl From::<Skill> for QueryResult {
    fn from(skill: Skill) -> Self {
        QueryResult::Skill(skill)
    }
}

impl From::<TaskList> for QueryResult {
    fn from(list: TaskList) -> Self {
        QueryResult::TaskList(list.0)
    }
}

impl From::<Task> for QueryResult {
    fn from(task: Task) -> Self {
        QueryResult::Task(task)
    }
}

pub async fn execute(pool: &Pool, query: Query) -> Result<QueryResult, AppError> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get())
        .await
        .map_err(|_| AppError::InternalError)? // blocking error
        .map_err(|e| AppError::DBError {
            error_msg: format!("error getting db connection after initialization, {}", e.to_string())
        })?;

    conn.execute("PRAGMA foreign_keys = ON;", ()).expect("cannot set pragma foreign_keys to ON");

    web::block(move || -> Result<QueryResult, AppError> {
        // simulates expensive query
        sleep(Duration::from_secs(1));

        match query {
            Query::GetCharacterList => {
                let character_list = get_character_list(&conn)
                    .map_err(|e| {
                        match e {
                            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                            _ => AppError::DBError {
                                error_msg: format!("in get_character_list, {}", e.to_string())
                            }
                        }
                    })?;
                Ok(QueryResult::from(character_list))
            },
            Query::GetCharacter(id) => {
                let character = get_character(&conn, id)
                    .map_err(|e| {
                        match e {
                            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                            _ => AppError::DBError {
                                error_msg: format!("in get_character, {}", e.to_string())
                            }
                        }
                    })?;
                Ok(QueryResult::from(character))
            },
            Query::GetCharacterSkillList(id) => {
                let character = get_character(&conn, id)
                    .map_err(|e| {
                        match e {
                            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                            _ => AppError::DBError {
                                error_msg: format!("in get_character_skill_list, get_character, {}", e.to_string())
                            }
                        }
                    })?;
                let skill_list = get_skill_list(&conn, Some(character.id))
                    .map_err(|e| AppError::DBError {
                        error_msg: format!("in get_character_skill_list get_skill_list, {}", e.to_string())
                    })?;
                Ok(QueryResult::from(skill_list))
            },
            Query::GetCharacterTaskList(id) => {
                let character = get_character(&conn, id)
                    .map_err(|e| {
                        match e {
                            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                            _ => AppError::DBError {
                                error_msg: format!("in get_character_task_list, get_character, {}", e.to_string())
                            }
                        }
                    })?;
                let skill_ids = get_skill_list(&conn, Some(character.id))
                    .map_err(|e| AppError::DBError {
                        error_msg: format!("in get_character_task_list, get_skill_list, {}", e.to_string())
                    })?
                    .0
                    .into_iter().map(|skill| skill.id)
                    .collect::<Vec<IdType>>();
                let mut tasks = Vec::<Task>::new();
                for &skill_id in skill_ids.iter() {
                    let mut skill_task_list = get_task_list(&conn, Some(skill_id))
                        .map_err(|e| AppError::DBError {
                            error_msg: format!("in get_character_task_list, get_task_list, {}", e.to_string())
                        })?;
                    tasks.append(&mut skill_task_list.0);
                }

                let task_list = TaskList(tasks);
                Ok(QueryResult::from(task_list))
            },
            Query::CreateCharacter(fields) => {
                create_character(&conn, fields).map_err(|e| AppError::DBError {
                    error_msg: format!("in create_character, {}", e.to_string())
                })?;
                let id = conn.last_insert_rowid() as IdType;
                let created_character = get_character(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in create_character, get_character, {}", e.to_string())
                })?;
                Ok(QueryResult::from(created_character))
            },
            Query::UpdateCharacter(id, fields) => {
                update_character(&conn, id, fields).map_err(|e| AppError::DBError {
                    error_msg: format!("in update_character, {}", e.to_string())
                })?;
                let updated_character = get_character(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in update_character, get_character, {}", e.to_string())
                })?;
                Ok(QueryResult::from(updated_character))
            },
            Query::DeleteCharacter(id) => {
                delete_character(&conn, id).map_err(|e| {
                    match e {
                        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                        _ => AppError::DBError {
                            error_msg: format!("in delete_character, {}", e.to_string())
                        }
                    }
                }
                )?;
                Ok(QueryResult::Success)
            },
            Query::GetSkillList => {
                let skill_list = get_skill_list(&conn, None).map_err(|e| AppError::DBError {
                    error_msg:  format!("in get_skill_list, {}", e.to_string())
                })?;
                Ok(QueryResult::from(skill_list))
            },
            Query::GetSkill(id) => {
                let skill = get_skill(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in get_skill, {}", e.to_string())
                })?;
                Ok(QueryResult::from(skill))
            },
            Query::GetSkillTaskList(id) => {
                let skill = get_skill(&conn, id)
                    .map_err(|e| {
                        match e {
                            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                            _ => AppError::DBError {
                                error_msg: format!("in get_skill_task_list, in get_skill, {}", e.to_string())
                            }
                        }
                    })?;

                let task_list = get_task_list(&conn, Some(skill.id)).map_err(|e| AppError::DBError {
                    error_msg: format!("in get_skill_task_list, in get_task_list, {}", e.to_string())
                })?;
                Ok(QueryResult::from(task_list))
            },
            Query::CreateCharacterSkill(character_id, fields) => {
                create_skill(&conn, character_id, fields).map_err(|e| AppError::DBError {
                    error_msg: format!("in create_skill, {}", e.to_string())
                })?;
                let id = conn.last_insert_rowid() as IdType;
                let created_skill = get_skill(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in create_skill, get_skill, {}", e.to_string())
                })?;
                Ok(QueryResult::from(created_skill))
            },
            Query::UpdateSkill(id, fields) => {
                update_skill(&conn, id, fields).map_err(|e| {
                    match e {
                        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                        _ => AppError::DBError {
                            error_msg: format!("in update_skill, {}", e.to_string())
                        }
                    }
                })?;
                let updated_skill = get_skill(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in update_skill, get_skill, {}", e.to_string())
                })?;
                Ok(QueryResult::from(updated_skill))
            },
            Query::DeleteSkill(id) => {
                delete_skill(&conn, id).map_err(|e| {
                    match e {
                        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                        _ => AppError::DBError {
                            error_msg: format!("in delete_skill, {}", e.to_string())
                        }
                    }
                })?;
                Ok(QueryResult::Success)
            },
            Query::GetTaskList => {
                let task_list = get_task_list(&conn, None).map_err(|e| AppError::DBError {
                    error_msg: format!("in get_task_list, {}", e.to_string())
                })?;
                Ok(QueryResult::from(task_list))
            },
            Query::GetTask(id) => {
                let task = get_task(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in get_task, {}", e.to_string())
                })?;
                Ok(QueryResult::from(task))
            },
            Query::CreateSkillTask(skill_id, fields) => {
                create_task(&conn, skill_id, fields).map_err(|e| AppError::DBError {
                    error_msg: format!("in create_task, {}", e.to_string())
                })?;
                let id = conn.last_insert_rowid() as IdType;
                let created_task = get_task(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in create_task, get_task, {}", e.to_string())
                })?;
                Ok(QueryResult::from(created_task))
            },
            Query::UpdateTask(id, fields) => {
                update_task(&conn, id, fields).map_err(|e| AppError::DBError {
                    error_msg: format!("in update_task, {}", e.to_string())
                })?;
                let updated_task = get_task(&conn, id).map_err(|e| AppError::DBError {
                    error_msg: format!("in update_task, get_task, {}", e.to_string())
                })?;
                Ok(QueryResult::from(updated_task))
            },
            Query::DeleteTask(id) => {
                delete_task(&conn, id).map_err(|e| {
                    match e {
                        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
                        _ => AppError::DBError {
                            error_msg: format!("in delete_task, {}", e.to_string())
                        }
                    }
                }
                )?;
                Ok(QueryResult::Success)
            },
            Query::ResetDB => {
                clear_db(&conn).map_err(|e| AppError::DBError {
                    error_msg: format!("in reset_db:clear_db, {}", e.to_string())
                })?;
                create_db(&conn).map_err(|e| AppError::DBError {
                    error_msg: format!("in reset_db:create_db, {}", e.to_string())
                })?;

                Ok(QueryResult::Success)
            },
        }
    })
    .await
    .map_err(|_| AppError::InternalError)? // blocking error
}

/// Delete all entries from the databse.
fn clear_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM task", ())?;
    conn.execute("DELETE FROM skill", ())?;
    conn.execute("DELETE FROM character", ())?;

    Ok(())
}

pub fn create_db(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    // character table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS character (
            id          INTEGER PRIMARY KEY,
            name        TEXT,
            avatar      TEXT,
            notes       TEXT,
            quote       TEXT,

            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        )",
        (),
    )?;

    // skill table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS skill (
            id              INTEGER PRIMARY KEY,
            name            TEXT,
            progress        INTEGER,
            level           INTEGER,
            character_id    INTEGER NOT NULL,
            created_at      INTEGER NOT NULL,
            updated_at      INTEGER NOT NULL,

            FOREIGN KEY(character_id) REFERENCES character(id) ON DELETE CASCADE
        )",
        (),
    )?;

    // task table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task (
            id                      INTEGER PRIMARY KEY,
            name                    TEXT,
            description             TEXT,
            completed               INTEGER NOT NULL CHECK (completed IN (0, 1)),
            skill_id                INTEGER NOT NULL,
            created_at              INTEGER NOT NULL,
            updated_at              INTEGER NOT NULL,

            FOREIGN KEY(skill_id)   REFERENCES skill(id) ON DELETE CASCADE
        )",
        (),
    )?;

    Ok(())
}
