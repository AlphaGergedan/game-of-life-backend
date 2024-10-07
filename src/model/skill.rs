use serde::{ Serialize, Deserialize };
use crate::{ IdType, TimeType, };

#[derive(Serialize, Deserialize)]
pub struct SkillFields {
    pub name: String,
    pub progress: u8,
    pub level: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Skill {
    pub id: IdType,
    pub fields: SkillFields,
    pub character_id: IdType,
    pub created_at: TimeType,
    pub updated_at: TimeType,
}

pub struct SkillList(pub Vec<Skill>);
