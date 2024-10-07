use serde::{ Serialize, Deserialize };
use crate::{ IdType, TimeType, };

#[derive(Serialize, Deserialize)]
pub struct TaskFields {
    pub name: String,
    pub description: String,
    pub completed: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: IdType,
    pub fields: TaskFields,
    pub skill_id: IdType,
    pub created_at: TimeType,
    pub updated_at: TimeType,
}

pub struct TaskList(pub Vec<Task>);
