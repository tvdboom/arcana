use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};
use crate::core::races::Race;

#[derive(EnumIter, Clone, Copy, Debug, EnumString, Serialize, Deserialize)]
pub enum Attribute {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub race: Race,
    pub health: f32,
    pub mana: f32,
    pub strength: f32,
    pub dexterity: f32,
    pub constitution: f32,
    pub intelligence: f32,
    pub wisdom: f32,
    pub charisma: f32,
}
