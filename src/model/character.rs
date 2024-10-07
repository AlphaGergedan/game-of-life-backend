use serde::{ Deserialize, Serialize, };
use crate::{ IdType, TimeType, };

#[derive(Serialize)]
pub struct Character {
    pub id: IdType,
    pub fields: CharacterFields,
    pub created_at: TimeType,
    pub updated_at: TimeType,
}

#[derive(Serialize, Deserialize)]
pub struct CharacterFields {
    pub name: String,
    pub avatar: String,
    pub notes: String,
    pub quote: String,
}

pub struct CharacterList(pub Vec<Character>);
