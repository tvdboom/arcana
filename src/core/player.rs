use crate::core::build::equipment::Equipment;
use crate::core::build::modifiers::Modifier;
use crate::core::catalog::{get_equipment, get_perk};
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

#[derive(EnumIter, EnumString, Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub gloves: Option<String>,
    pub boots: Option<String>,
    pub weapon_lh: Option<String>,
    pub weapon_rh: Option<String>,
    pub accessory: Option<String>,
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
            accessory: None,
            gloves: None,
            accessory2: None,
            inventory: vec![],
            gold: 100,
        }
    }
}

impl Player {
    pub fn adjust_health_mana_after_change(&mut self, old_max_hp: u32, old_max_mp: u32) {
        let new_max_hp = self.max_health();
        let new_max_mp = self.max_mana();
        if new_max_hp > old_max_hp {
            self.health += new_max_hp - old_max_hp;
        }
        if new_max_mp > old_max_mp {
            self.mana += new_max_mp - old_max_mp;
        }
        self.health = self.health.min(new_max_hp);
        self.mana = self.mana.min(new_max_mp);
    }

    pub fn attribute_perk_mod(&self, attr: Attribute) -> i32 {
        let mut perk_mod = 0;
        for perk_key in &self.perks {
            if let Some(perk) = get_perk(perk_key) {
                for modifier in &perk.modifiers {
                    if let Modifier::AttributeModifier(target_attr, val) = modifier {
                        if *target_attr == attr {
                            perk_mod += val;
                        }
                    }
                }
            }
        }
        perk_mod
    }

    pub fn strength(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Strength);
        let sex_mod = self.sex.characteristic_mod(Attribute::Strength);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Strength, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Strength);
        (self.strength as i32 + race_mod + sex_mod + equip_mod + perk_mod).max(0) as u32
    }

    pub fn strength_mod(&self) -> u32 {
        self.strength().checked_sub(START_CHARACTERISTIC).unwrap_or_default()
    }

    pub fn dexterity(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Dexterity);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Dexterity, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Dexterity);
        (self.dexterity as i32 + race_mod + equip_mod + perk_mod).max(0) as u32
    }

    pub fn dexterity_mod(&self) -> u32 {
        self.dexterity().checked_sub(START_CHARACTERISTIC).unwrap_or_default()
    }

    pub fn constitution(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Constitution);
        let age_mod = self.stage.characteristic_mod(Attribute::Constitution);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Constitution, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Constitution);
        (self.constitution as i32 + race_mod - age_mod + equip_mod + perk_mod).max(0) as u32
    }

    pub fn constitution_mod(&self) -> u32 {
        self.constitution().checked_sub(START_CHARACTERISTIC).unwrap_or_default()
    }

    pub fn intelligence(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Intelligence);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Intelligence, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Intelligence);
        (self.intelligence as i32 + race_mod + equip_mod + perk_mod).max(0) as u32
    }

    pub fn wisdom(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Wisdom);
        let age_mod = self.stage.characteristic_mod(Attribute::Wisdom);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Wisdom, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Wisdom);
        (self.wisdom as i32 + race_mod + age_mod + equip_mod + perk_mod).max(0) as u32
    }

    pub fn wisdom_mod(&self) -> u32 {
        self.wisdom().checked_sub(START_CHARACTERISTIC).unwrap_or_default()
    }

    pub fn charisma(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Charisma);
        let sex_mod = self.sex.characteristic_mod(Attribute::Charisma);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Charisma, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Charisma);
        (self.charisma as i32 + race_mod + sex_mod + equip_mod + perk_mod).max(0) as u32
    }

    /// All currently equipped pieces of gear.
    pub fn equipped_equipment(&self) -> Vec<Equipment> {
        [
            &self.helmet,
            &self.armor,
            &self.boots,
            &self.weapon_lh,
            &self.weapon_rh,
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
        let base = 100 + 10 * self.constitution_mod() as i32;
        let class_mod = if self.class == Class::Warrior {
            20
        } else {
            0
        };
        let perk_health_mod: i32 = self.perks
            .iter()
            .filter_map(|key| get_perk(key))
            .flat_map(|perk| perk.modifiers.clone().into_iter())
            .filter_map(|m| {
                if let Modifier::MaxHealthModifier(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .sum();
        let equip_health_mod: i32 = self.equipped_equipment()
            .iter()
            .flat_map(|eq| eq.modifiers().iter())
            .filter_map(|m| {
                if let Modifier::MaxHealthModifier(v) = m {
                    Some(*v)
                } else {
                    None
                }
            })
            .sum();
        (base + class_mod + self.bonus_max_health as i32 + perk_health_mod + equip_health_mod).max(1) as u32
    }

    pub fn max_mana(&self) -> u32 {
        let base = 100 + 10 * self.wisdom_mod() as i32;
        let class_mod = match self.class {
            Class::Mage(_) => 30,
            Class::Druid => 10,
            _ => 0,
        };
        let perk_mana_mod: i32 = self.perks
            .iter()
            .filter_map(|key| get_perk(key))
            .flat_map(|perk| perk.modifiers.clone().into_iter())
            .filter_map(|m| {
                if let Modifier::MaxManaModifier(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .sum();
        let equip_mana_mod: i32 = self.equipped_equipment()
            .iter()
            .flat_map(|eq| eq.modifiers().iter())
            .filter_map(|m| {
                if let Modifier::MaxManaModifier(v) = m {
                    Some(*v)
                } else {
                    None
                }
            })
            .sum();
        (base + class_mod + self.bonus_max_mana as i32 + perk_mana_mod + equip_mana_mod).max(0) as u32
    }

    pub fn attack(&self) -> u32 {
        (5 + self.strength_mod() as i32
            + self.equipped_equipment().iter().map(|w| w.attack()).sum::<i32>()
            + self
                .perks
                .iter()
                .filter_map(|key| get_perk(key))
                .flat_map(|perk| perk.modifiers)
                .filter_map(|m| {
                    if let Modifier::AttackModifier(v) = m {
                        Some(v)
                    } else {
                        None
                    }
                })
                .sum::<i32>()
                .max(0)) as u32
    }

    pub fn defense(&self) -> u32 {
        (5 + self.constitution_mod() as i32
            + self.equipped_equipment().iter().map(|w| w.defense()).sum::<i32>()
            + self
                .perks
                .iter()
                .filter_map(|key| get_perk(key))
                .flat_map(|perk| perk.modifiers)
                .filter_map(|m| {
                    if let Modifier::DefenseModifier(v) = m {
                        Some(v)
                    } else {
                        None
                    }
                })
                .sum::<i32>()
                .max(0)) as u32
    }

    pub fn initiative(&self) -> u32 {
        (5 + self.dexterity_mod() as i32
            + self.equipped_equipment().iter().map(|w| w.initiative()).sum::<i32>()
            + self
                .perks
                .iter()
                .filter_map(|key| get_perk(key))
                .flat_map(|perk| perk.modifiers)
                .filter_map(|m| {
                    if let Modifier::InitiativeModifier(v) = m {
                        Some(v)
                    } else {
                        None
                    }
                })
                .sum::<i32>()
                .max(0)) as u32
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
