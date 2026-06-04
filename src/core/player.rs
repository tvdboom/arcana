use crate::core::abilities::Ability;
use crate::core::classes::Class;
use crate::core::races::Race;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};

#[derive(EnumIter, Clone, Copy, Debug, EnumString, Serialize, Deserialize)]
pub enum Attribute {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub health: f32,
    pub mana: f32,
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
    pub abilities: Vec<Ability>,
}

const FANTASY_NAMES: &[&str] = &[
    "Eldrin",
    "Zephyrus",
    "Thorne",
    "Lyra",
    "Kaelen",
    "Seraphina",
    "Valerius",
    "Sylas",
    "Thalia",
    "Morwenna",
    "Baelor",
    "Elara",
    "Garrick",
    "Freya",
    "Cedric",
    "Rowena",
    "Gideon",
    "Isolde",
    "Alistair",
    "Evadne",
    "Ronan",
    "Cassandra",
    "Dorian",
    "Genevieve",
    "Lucian",
    "Maeve",
    "Tristan",
    "Aurelia",
    "Percival",
    "Gwendolyn",
    "Alaric",
    "Fiona",
    "Orpheus",
    "Guinevere",
    "Morgana",
    "Ygritte",
    "Daenerys",
    "Tyrion",
    "Arya",
    "Ignis",
    "Vaelen",
    "Elidor",
    "Zephyra",
    "Katherine",
    "Malakor",
    "Rhaegar",
    "Viserys",
    "Melisandre",
    "Jorah",
    "Daario",
];

impl Default for Player {
    fn default() -> Self {
        use rand::seq::IndexedRandom;
        let mut rng = rand::rng();
        let name = FANTASY_NAMES.choose(&mut rng).copied().unwrap_or("Arcana").to_string();
        Self {
            name,
            race: Race::default(),
            class: Class::default(),
            health: 100.,
            mana: 100.,
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            abilities: vec![],
        }
    }
}
