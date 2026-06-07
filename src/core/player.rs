use crate::core::abilities::Ability;
use crate::core::classes::Class;
use crate::core::perks::Perk;
use crate::core::pets::Pet;
use crate::core::races::Race;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use strum_macros::{EnumIter, EnumString};
use crate::core::constants::FANTASY_NAMES;
use crate::core::consumables::Consumable;
use crate::core::weapons::Weapon;

#[derive(EnumIter, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Sex {
    #[default]
    Male,
    Female,
}

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
    pub sex: Sex,
    pub race: Race,
    pub class: Class,
    pub age: u32,
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
            sex: Sex::default(),
            race: Race::default(),
            class: Class::default(),
            age: 25,
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
    /// Wisdom bonus (and equal strength penalty) gained with age: +0 when young,
    /// up to +2 when at the upper end of the race's age range.
    pub fn age_modifier(&self) -> i16 {
        let (min, max) = self.race.age_range();
        if max <= min {
            return 0;
        }
        let fraction = (self.age.clamp(min, max) - min) as f32 / (max - min) as f32;
        (fraction * 2.).round() as i16
    }

    pub fn strength(&self) -> u8 {
        let base = self.strength as i16;
        let modifier = self.race.modifier(Attribute::Strength) as i16;
        let sex_mod = if matches!(self.sex, Sex::Male) { 1 } else { 0 };
        (base + modifier - self.age_modifier() + sex_mod).max(0) as u8
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
        (base + modifier + self.age_modifier()).max(0) as u8
    }

    pub fn charisma(&self) -> u8 {
        let base = self.charisma as i16;
        let modifier = self.race.modifier(Attribute::Charisma) as i16;
        let sex_mod = if matches!(self.sex, Sex::Female) { 1 } else { 0 };
        (base + modifier + sex_mod).max(0) as u8
    }

    /// All currently equipped pieces of gear.
    pub fn equipped_weapons(&self) -> Vec<&Weapon> {
        [
            &self.helmet,
            &self.armor,
            &self.boots,
            &self.weapon_lh,
            &self.weapon_rh,
            &self.weapon_2h,
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn max_health(&self) -> f32 {
        let mut base_max = self.constitution() as f32 * 5. + (self.level as f32 - 1.) * 10. + 50.;
        if matches!(self.class, Class::Warrior) {
            base_max += 20.;
        }
        base_max
    }

    pub fn max_mana(&self) -> f32 {
        let mut base_max = self.intelligence() as f32 * 3. + self.wisdom() as f32 * 2. + 50.;
        if matches!(self.class, Class::Mage(_)) {
            base_max += 30.;
        }
        base_max
    }

    /// Total physical attack damage (base from strength plus weapon bonuses).
    pub fn attack_damage(&self) -> i32 {
        self.strength() as i32 / 2 + self.equipped_weapons().iter().map(|w| w.stats().attack).sum::<i32>()
    }

    /// Total armor rating (base from constitution plus equipment bonuses).
    pub fn armor_value(&self) -> i32 {
        self.constitution() as i32 / 4 + self.equipped_weapons().iter().map(|w| w.stats().armor).sum::<i32>()
    }

    /// Initiative determines turn order (base from dexterity plus equipment bonuses).
    pub fn initiative(&self) -> i32 {
        let base_init = self.dexterity() as i32 / 2 + self.equipped_weapons().iter().map(|w| w.stats().initiative).sum::<i32>();
        if matches!(self.class, Class::Rogue) {
            base_init + 2
        } else {
            base_init
        }
    }

    /// (age, height_cm, weight_kg). Age is the value chosen during character
    /// creation; height and weight are derived deterministically from name and race.
    pub fn vitals(&self) -> (u32, u32, u32) {
        let (_age_r, height_r, weight_r) = self.race.vital_ranges();

        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        format!("{:?}", self.race).hash(&mut hasher);
        let seed = hasher.finish();

        let pick = |range: (u32, u32), salt: u64| -> u32 {
            let span = (range.1 - range.0 + 1) as u64;
            range.0 + ((seed.rotate_left(salt as u32 * 17) ^ salt) % span) as u32
        };

        (self.age, pick(height_r, 2), pick(weight_r, 3))
    }
}
