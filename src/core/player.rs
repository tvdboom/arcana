use crate::core::abilities::Ability;
use crate::core::classes::Class;
use crate::core::perks::Perk;
use crate::core::pets::Pet;
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
    pub perks: Vec<Perk>,
    pub pet: Option<Pet>,
}

const FANTASY_NAMES: &[&str] = &[
    "Eldrin",
    "Zephyrus",
    "Thorne",
    "Kaelen",
    "Valerius",
    "Sylas",
    "Baelor",
    "Garrick",
    "Cedric",
    "Gideon",
    "Alistair",
    "Ronan",
    "Dorian",
    "Lucian",
    "Tristan",
    "Percival",
    "Alaric",
    "Orpheus",
    "Tyrion",
    "Ignis",
    "Vaelen",
    "Elidor",
    "Malakor",
    "Rhaegar",
    "Viserys",
    "Jorah",
    "Daario",
    "Arthur",
    "Lancelot",
    "Merlin",
    "Kenneth",
    "Raymond",
    "Jonan",
    "Bran",
    "Sanson",
    "Loras",
    "Oberyn",
    "Theon",
    "Stannis",
    "Davos",
    "Barristan",
    "Joffrey",
    "Sandor",
    "Gregor",
    "Renly",
    "Eddard",
    "Robert",
    "Tywin",
    "Jaime",
    "Ramsay",
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
            perks: vec![Perk::IronSkin],
            pet: None,
        }
    }
}
