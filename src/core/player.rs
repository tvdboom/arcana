use crate::core::catalog::{get_equipment, GeneratedEquipment};
use crate::core::classes::Class;
use crate::core::constants::{NAMES, START_CHARACTERISTIC};
use crate::core::pets::Pet;
use crate::core::races::Race;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(EnumIter, Clone, Copy, Debug, Display, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Sex {
    #[default]
    Man,
    Woman,
}

impl Sex {
    pub fn characteristic_mod(&self, attr: Attribute) -> i32 {
        match attr {
            Attribute::Strength => match self {
                Sex::Man => 1,
                Sex::Woman => 0,
            },
            Attribute::Charisma => match self {
                Sex::Man => 0,
                Sex::Woman => 1,
            },
            _ => 0,
        }
    }
}

#[derive(EnumIter, Clone, Copy, Debug, Display, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AgeStage {
    Youth,
    YoungAdult,
    #[default]
    Adult,
    Senior,
    Elder,
}

impl AgeStage {
    pub fn from_u32(u: u32) -> Self {
        match u {
            0 => Self::Youth,
            1 => Self::YoungAdult,
            2 => Self::Adult,
            3 => Self::Senior,
            4 => Self::Elder,
            _ => panic!("invalid stage {u}"),
        }
    }

    pub fn index(&self) -> u32 {
        match self {
            AgeStage::Youth => 0,
            AgeStage::YoungAdult => 1,
            AgeStage::Adult => 2,
            AgeStage::Senior => 3,
            AgeStage::Elder => 4,
        }
    }

    pub fn frac(&self) -> f32 {
        self.index() as f32 / (Self::iter().len() - 1) as f32
    }

    pub fn characteristic_mod(&self, attr: Attribute) -> i32 {
        match attr {
            Attribute::Constitution => match self {
                AgeStage::Youth => 2,
                AgeStage::YoungAdult => 1,
                AgeStage::Adult => 0,
                AgeStage::Senior => -1,
                AgeStage::Elder => -2,
            },
            Attribute::Wisdom => match self {
                AgeStage::Youth => -2,
                AgeStage::YoungAdult => -1,
                AgeStage::Adult => 0,
                AgeStage::Senior => 1,
                AgeStage::Elder => 2,
            },
            _ => 0,
        }
    }
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
    pub stage: AgeStage,
    pub age: u32,
    pub level: u8,
    pub ap: u32,
    pub health: u32,
    pub mana: u32,
    pub bonus_max_health: u32,
    pub bonus_max_mana: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub wisdom: u32,
    pub charisma: u32,
    pub abilities: Vec<String>,
    pub perks: Vec<String>,
    pub pet: Option<Pet>,
    pub helmet: Option<String>,
    pub armor: Option<String>,
    pub boots: Option<String>,
    pub weapon_lh: Option<String>,
    pub weapon_rh: Option<String>,
    pub weapon_2h: Option<String>,
    pub accessory: Option<String>,
    pub gloves: Option<String>,
    pub accessory2: Option<String>,
    pub inventory: Vec<String>,
    pub gold: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: NAMES.choose(&mut rng()).unwrap().to_string(),
            sex: Sex::default(),
            race: Race::default(),
            class: Class::default(),
            stage: AgeStage::default(),
            age: 0,
            level: 1,
            ap: 10,
            health: 100,
            mana: 100,
            bonus_max_health: 0,
            bonus_max_mana: 0,
            strength: START_CHARACTERISTIC,
            dexterity: START_CHARACTERISTIC,
            constitution: START_CHARACTERISTIC,
            intelligence: START_CHARACTERISTIC,
            wisdom: START_CHARACTERISTIC,
            charisma: START_CHARACTERISTIC,
            abilities: vec![],
            perks: vec![],
            pet: None,
            helmet: None,
            armor: None,
            boots: None,
            weapon_lh: None,
            weapon_rh: None,
            weapon_2h: None,
            accessory: None,
            gloves: None,
            accessory2: None,
            inventory: vec![],
            gold: 100,
        }
    }
}

impl Player {
    pub fn strength(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Strength);
        let sex_mod = self.sex.characteristic_mod(Attribute::Strength);
        (self.strength as i32 + race_mod + sex_mod).max(0) as u32
    }

    pub fn strength_mod(&self) -> u32 {
        (self.strength() - START_CHARACTERISTIC).max(0)
    }

    pub fn dexterity(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Dexterity);
        (self.dexterity as i32 + race_mod).max(0) as u32
    }

    pub fn constitution(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Constitution);
        let age_mod = self.stage.characteristic_mod(Attribute::Constitution);
        (self.constitution as i32 + race_mod - age_mod).max(0) as u32
    }

    pub fn constitution_mod(&self) -> u32 {
        (self.constitution() - START_CHARACTERISTIC).max(0)
    }

    pub fn intelligence(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Intelligence);
        (self.intelligence as i32 + race_mod).max(0) as u32
    }

    pub fn wisdom(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Wisdom);
        let age_mod = self.stage.characteristic_mod(Attribute::Wisdom);
        (self.wisdom as i32 + race_mod + age_mod).max(0) as u32
    }

    pub fn wisdom_mod(&self) -> u32 {
        (self.wisdom() - START_CHARACTERISTIC).max(0)
    }

    pub fn charisma(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Charisma);
        let sex_mod = self.sex.characteristic_mod(Attribute::Charisma);
        (self.charisma as i32 + race_mod + sex_mod).max(0) as u32
    }

    /// All currently equipped pieces of gear.
    pub fn equipped_equipment(&self) -> Vec<GeneratedEquipment> {
        [
            &self.helmet,
            &self.armor,
            &self.boots,
            &self.weapon_lh,
            &self.weapon_rh,
            &self.weapon_2h,
            &self.accessory,
            &self.gloves,
            &self.accessory2,
        ]
        .into_iter()
        .flatten()
        .filter_map(|key| get_equipment(key))
        .collect()
    }

    pub fn max_health(&self) -> u32 {
        let base = 100 + 10 * self.constitution_mod();
        let class_mod = if self.class == Class::Warrior {
            20
        } else {
            0
        };
        base + class_mod + self.bonus_max_health
    }

    pub fn max_mana(&self) -> u32 {
        let base = 100 + 10 * self.wisdom_mod();
        let class_mod = match self.class {
            Class::Mage(_) => 30,
            Class::Druid => 10,
            _ => 0,
        };
        base + class_mod + self.bonus_max_mana
    }

    /// Total physical attack damage (base from strength plus weapon bonuses).
    pub fn attack_damage(&self) -> u32 {
        let equip_mod = self.equipped_equipment().iter().map(|w| w.attack).sum::<i32>();
        (5 + self.strength_mod() as i32 + equip_mod) as u32
    }

    pub fn weapon_attack_speed(&self, weapon_key: &str) -> f32 {
        let weapon_speed = get_equipment(weapon_key).map(|w| w.attack_speed).unwrap_or(1.0);
        self.adjust_attack_speed(weapon_speed)
    }

    fn adjust_attack_speed(&self, weapon_speed: f32) -> f32 {
        let dex_bonus = (self.dexterity() as f32 - 10.) * 0.05;
        (weapon_speed + dex_bonus).max(0.3)
    }

    pub fn defense_value(&self) -> i32 {
        self.constitution() as i32 / 4
            + self.equipped_equipment().iter().map(|w| w.defense).sum::<i32>()
    }

    pub fn initiative(&self) -> i32 {
        let base_init = self.dexterity() as i32 / 2
            + self.equipped_equipment().iter().map(|w| w.initiative).sum::<i32>();
        if matches!(self.class, Class::Assassin) {
            base_init + 2
        } else {
            base_init
        }
    }

    /// (height_cm, weight_kg). Height and weight are derived deterministically from name and race.
    pub fn vitals(&self) -> (u32, u32) {
        let (_, height_r, _) = self.race.vital_ranges();

        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        format!("{:?}", self.race).hash(&mut hasher);
        let seed = hasher.finish();

        let pick = |range: (u32, u32), salt: u64| -> u32 {
            let span = (range.1 - range.0 + 1) as u64;
            range.0 + ((seed.rotate_left(salt as u32 * 17) ^ salt) % span) as u32
        };

        let height = pick(height_r, 2);

        // Generate a random seed based on the race, name, and the generated height
        let mut weight_hasher = DefaultHasher::new();
        self.name.hash(&mut weight_hasher);
        format!("{:?}", self.race).hash(&mut weight_hasher);
        height.hash(&mut weight_hasher);
        let weight_seed = weight_hasher.finish();

        // Get a random float from 0.0 to 1.0 based on name, race, and height
        let rand_val = (weight_seed % 1000) as f32 / 1000.0;

        // Calculate weight based on height (using race-specific BMI ranges)
        let height_m = height as f32 / 100.0;
        let bmi = match self.race {
            Race::Elf => 16.5 + rand_val * 3.0,
            Race::Human => 21.0 + rand_val * 4.0,
            Race::Dwarf => 45.0 + rand_val * 10.0,
            Race::Orc => 31.0 + rand_val * 6.0,
        };

        let weight = (height_m * height_m * bmi).round() as u32;

        (height, weight)
    }
}
