use crate::core::abilities::Ability;
use crate::core::classes::Class;
use crate::core::perks::Perk;
use crate::core::pets::Pet;
use crate::core::races::Race;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};
use crate::core::constants::FANTASY_NAMES;
use crate::core::consumables::Consumable;
use crate::core::weapons::Weapon;

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
    pub level: u8,
    pub ap: u32,
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
    pub helmet: Option<Weapon>,
    pub armor: Option<Weapon>,
    pub boots: Option<Weapon>,
    pub weapon_lh: Option<Weapon>,
    pub weapon_rh: Option<Weapon>,
    pub weapon_2h: Option<Weapon>,
    pub consumables: Vec<Consumable>,
    pub money: u32,
}

impl Default for Player {
    fn default() -> Self {
        let name = FANTASY_NAMES.choose(&mut rng()).copied().unwrap_or("Arcana").to_string();
        Self {
            name,
            race: Race::default(),
            class: Class::default(),
            level: 1,
            ap: 10,
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
            helmet: None,
            armor: None,
            boots: None,
            weapon_lh: None,
            weapon_rh: None,
            weapon_2h: None,
            consumables: vec![],
            money: 100,
        }
    }
}

impl Player {
    pub fn strength(&self) -> u8 {
        let base = self.strength as i16;
        let modifier = self.race.modifier(Attribute::Strength) as i16;
        (base + modifier).max(0) as u8
    }

    pub fn dexterity(&self) -> u8 {
        let base = self.dexterity as i16;
        let modifier = self.race.modifier(Attribute::Dexterity) as i16;
        (base + modifier).max(0) as u8
    }

    pub fn constitution(&self) -> u8 {
        let base = self.constitution as i16;
        let modifier = self.race.modifier(Attribute::Constitution) as i16;
        (base + modifier).max(0) as u8
    }

    pub fn intelligence(&self) -> u8 {
        let base = self.intelligence as i16;
        let modifier = self.race.modifier(Attribute::Intelligence) as i16;
        (base + modifier).max(0) as u8
    }

    pub fn wisdom(&self) -> u8 {
        let base = self.wisdom as i16;
        let modifier = self.race.modifier(Attribute::Wisdom) as i16;
        (base + modifier).max(0) as u8
    }

    pub fn charisma(&self) -> u8 {
        let base = self.charisma as i16;
        let modifier = self.race.modifier(Attribute::Charisma) as i16;
        (base + modifier).max(0) as u8
    }
}
