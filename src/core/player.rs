//! Player state, attributes, inventory, equipment, and derived statistics.

use crate::core::catalog::catalog::{get_equipment, get_perk};
use crate::core::catalog::equipment::Equipment;
use crate::core::catalog::modifiers::Modifier;
use crate::core::catalog::weapons::Category;
use crate::core::classes::{Class, ClassSpecialization};
use crate::core::constants::{NAMES, START_CHARACTERISTIC};
use crate::core::deities::Deity;
use crate::core::identity::IdentityBonuses;
use crate::core::monsters::Monster;
use crate::core::races::{ElfHeritage, Race};
use crate::utils::NameFromEnum;
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
    /// Returns the sex-specific Strength or Charisma adjustment.
    pub fn characteristic_mod(&self, attr: Attribute) -> i32 {
        match (self, attr) {
            (Sex::Man, Attribute::Strength) | (Sex::Woman, Attribute::Charisma) => 1,
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
    /// Performs the from u32 operation.
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

    /// Performs the index operation.
    pub fn index(&self) -> u32 {
        match self {
            AgeStage::Youth => 0,
            AgeStage::YoungAdult => 1,
            AgeStage::Adult => 2,
            AgeStage::Senior => 3,
            AgeStage::Elder => 4,
        }
    }

    /// Performs the frac operation.
    pub fn frac(&self) -> f32 {
        self.index() as f32 / (Self::iter().len() - 1) as f32
    }

    /// Returns the Constitution and Wisdom tradeoff for this age stage.
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

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Skill {
    pub attack: u32,
    pub defense: u32,
    pub initiative: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Training {
    pub melee: Skill,
    pub finesse: Skill,
    pub range: Skill,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub sex: Sex,
    pub race: Race,
    pub class: Class,
    pub stage: AgeStage,
    pub age: u32,
    pub xp: u32,
    pub ap: u32,
    pub missing_health: u32,
    pub missing_mana: u32,
    pub bonus_max_health: u32,
    pub bonus_max_mana: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub wisdom: u32,
    pub charisma: u32,
    pub abilities: Vec<String>,
    pub active_abilities: Vec<Option<String>>,
    pub perks: Vec<String>,
    pub pet: Option<Monster>,
    pub helmet: Option<String>,
    pub armor: Option<String>,
    pub gloves: Option<String>,
    pub boots: Option<String>,
    pub weapon_lh: Option<String>,
    pub weapon_rh: Option<String>,
    pub accessory: Option<String>,
    pub accessory2: Option<String>,
    #[serde(default)]
    pub equipped_consumables: Vec<String>,
    pub inventory: Vec<String>,
    pub gold: u32,
    pub training: Training,
    pub elf_heritage: ElfHeritage,
    pub specialization: ClassSpecialization,
    pub deity: Deity,
}

impl Default for Player {
    /// Returns the default value.
    fn default() -> Self {
        Self {
            name: NAMES.choose(&mut rng()).unwrap().to_string(),
            sex: Sex::default(),
            race: Race::default(),
            class: Class::default(),
            stage: AgeStage::default(),
            age: 0,
            xp: 10,
            ap: 0,
            missing_health: 0,
            missing_mana: 0,
            bonus_max_health: 0,
            bonus_max_mana: 0,
            strength: START_CHARACTERISTIC,
            dexterity: START_CHARACTERISTIC,
            constitution: START_CHARACTERISTIC,
            intelligence: START_CHARACTERISTIC,
            wisdom: START_CHARACTERISTIC,
            charisma: START_CHARACTERISTIC,
            abilities: vec![],
            active_abilities: vec![None; 5],
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
            equipped_consumables: vec![],
            inventory: vec![],
            gold: 100,
            training: Training::default(),
            elf_heritage: ElfHeritage::default(),
            specialization: ClassSpecialization::default(),
            deity: Deity::default(),
        }
    }
}

impl Player {
    /// Returns the race, sex, class, and specialization-specific character portrait key.
    pub fn portrait_key(&self) -> String {
        let race = self.race.to_lowername();
        let sex = self.sex.to_lowername();
        match self.class {
            Class::Assassin => match self.specialization {
                ClassSpecialization::Assassin(path) => {
                    let path = path.to_lowername().replace(' ', "_");
                    format!("assassin_{path}_{race}_{sex}")
                },
                _ => format!("assassin_{race}_{sex}"),
            },
            Class::Druid => format!("druid_{race}_{sex}"),
            Class::Mage(ajah) => {
                let ajah = ajah.to_lowername();
                format!("mage_{ajah}_{race}_{sex}")
            },
            Class::Warrior => match self.specialization {
                ClassSpecialization::Warrior(path) => {
                    let path = path.to_lowername().replace(' ', "_");
                    format!("warrior_{path}_{race}_{sex}")
                },
                _ => format!("warrior_{race}_{sex}"),
            },
            Class::Monk => match self.specialization {
                ClassSpecialization::Monk(school) => {
                    let school = school.to_lowername().replace(' ', "_");
                    format!("monk_{school}_{race}_{sex}")
                },
                _ => format!("monk_{race}_{sex}"),
            },
            Class::Bard => match self.specialization {
                ClassSpecialization::Bard(style) => {
                    let style = style.to_lowername().replace(' ', "_");
                    format!("bard_{style}_{race}_{sex}")
                },
                _ => format!("bard_{race}_{sex}"),
            },
        }
    }

    pub const MAX_EQUIPPED_CONSUMABLE_TYPES: usize = 8;

    /// Applies combat bonus.
    fn apply_combat_bonus(base: u32, bonus: i32) -> u32 {
        if bonus >= 0 {
            base.saturating_add(bonus as u32)
        } else {
            base.saturating_sub((-bonus) as u32)
        }
    }

    /// Performs the level operation.
    pub fn level(&self) -> u32 {
        self.xp / 10
    }

    /// Performs the health operation.
    pub fn health(&self) -> u32 {
        self.max_health().saturating_sub(self.missing_health)
    }

    /// Performs the set health operation.
    pub fn set_health(&mut self, val: u32) {
        let max_hp = self.max_health();
        let val = val.min(max_hp);
        self.missing_health = max_hp.saturating_sub(val);
    }

    /// Performs the mana operation.
    pub fn mana(&self) -> u32 {
        self.max_mana().saturating_sub(self.missing_mana)
    }

    /// Performs the set mana operation.
    pub fn set_mana(&mut self, val: u32) {
        let max_mp = self.max_mana();
        let val = val.min(max_mp);
        self.missing_mana = max_mp.saturating_sub(val);
    }

    /// Updates health mana.
    pub fn update_health_mana(&mut self, _old_max_hp: u32, _old_max_mp: u32) {
        self.missing_health = self.missing_health.min(self.max_health());
        self.missing_mana = self.missing_mana.min(self.max_mana());
    }

    /// Performs the attribute perk mod operation.
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

    /// Performs the strength operation.
    pub fn strength(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Strength);
        let heritage_mod = self.elf_heritage_mod(Attribute::Strength);
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
        (self.strength as i32 + race_mod + heritage_mod + sex_mod + equip_mod + perk_mod).max(0)
            as u32
    }

    /// Performs the strength mod operation.
    pub fn strength_mod(&self) -> i32 {
        self.strength() as i32 - START_CHARACTERISTIC as i32
    }

    /// Performs the dexterity operation.
    pub fn dexterity(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Dexterity);
        let heritage_mod = self.elf_heritage_mod(Attribute::Dexterity);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Dexterity, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Dexterity);
        (self.dexterity as i32 + race_mod + heritage_mod + equip_mod + perk_mod).max(0) as u32
    }

    /// Performs the dexterity mod operation.
    pub fn dexterity_mod(&self) -> i32 {
        self.dexterity() as i32 - START_CHARACTERISTIC as i32
    }

    /// Performs the constitution operation.
    pub fn constitution(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Constitution);
        let heritage_mod = self.elf_heritage_mod(Attribute::Constitution);
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
        (self.constitution as i32 + race_mod + heritage_mod + age_mod + equip_mod + perk_mod).max(0)
            as u32
    }

    /// Performs the constitution mod operation.
    pub fn constitution_mod(&self) -> i32 {
        self.constitution() as i32 - START_CHARACTERISTIC as i32
    }

    /// Performs the intelligence operation.
    pub fn intelligence(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Intelligence);
        let heritage_mod = self.elf_heritage_mod(Attribute::Intelligence);
        let mut equip_mod = 0;
        for eq in self.equipped_equipment() {
            for modifier in eq.modifiers() {
                if let Modifier::AttributeModifier(Attribute::Intelligence, val) = modifier {
                    equip_mod += val;
                }
            }
        }
        let perk_mod = self.attribute_perk_mod(Attribute::Intelligence);
        (self.intelligence as i32 + race_mod + heritage_mod + equip_mod + perk_mod).max(0) as u32
    }

    /// Performs the intelligence mod operation.
    pub fn intelligence_mod(&self) -> i32 {
        self.intelligence() as i32 - START_CHARACTERISTIC as i32
    }

    /// Performs the wisdom operation.
    pub fn wisdom(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Wisdom);
        let heritage_mod = self.elf_heritage_mod(Attribute::Wisdom);
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
        (self.wisdom as i32 + race_mod + heritage_mod + age_mod + equip_mod + perk_mod).max(0)
            as u32
    }

    /// Performs the wisdom mod operation.
    pub fn wisdom_mod(&self) -> i32 {
        self.wisdom() as i32 - START_CHARACTERISTIC as i32
    }

    /// Performs the charisma operation.
    pub fn charisma(&self) -> u32 {
        let race_mod = self.race.characteristic_mod(Attribute::Charisma);
        let heritage_mod = self.elf_heritage_mod(Attribute::Charisma);
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
        (self.charisma as i32 + race_mod + heritage_mod + sex_mod + equip_mod + perk_mod).max(0)
            as u32
    }

    /// Performs the charisma mod operation.
    pub fn charisma_mod(&self) -> i32 {
        self.charisma() as i32 - START_CHARACTERISTIC as i32
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

    /// Returns all currently active perk and equipment modifiers.
    pub fn active_modifiers(&self) -> Vec<Modifier> {
        let mut modifiers = self
            .perks
            .iter()
            .filter_map(|key| get_perk(key))
            .flat_map(|perk| perk.modifiers)
            .collect::<Vec<_>>();
        modifiers.extend(
            self.equipped_equipment()
                .iter()
                .flat_map(|equipment| equipment.modifiers().iter().cloned()),
        );
        modifiers
    }

    /// Returns whether consumable equipped.
    pub fn is_consumable_equipped(&self, key: &str) -> bool {
        self.equipped_consumables.iter().any(|k| k == key)
    }

    /// Toggles consumable equipped.
    pub fn toggle_consumable_equipped(&mut self, key: &str) -> bool {
        if self.is_consumable_equipped(key) {
            self.equipped_consumables.retain(|k| k != key);
            return true;
        }

        if !self.inventory.iter().any(|k| k == key) {
            return false;
        }
        if !matches!(get_equipment(key), Some(Equipment::Consumable(_))) {
            return false;
        }
        if self.equipped_consumables.len() >= Self::MAX_EQUIPPED_CONSUMABLE_TYPES {
            return false;
        }

        self.equipped_consumables.push(key.to_string());
        true
    }

    /// Adds inventory item.
    pub fn add_inventory_item(&mut self, key: String) {
        self.inventory.push(key.clone());
        self.auto_equip_consumable_if_possible(&key);
    }

    /// Performs the set pet operation.
    pub fn set_pet(&mut self, mut pet: Monster) {
        pet.health = pet.max_health;
        self.pet = Some(pet);
    }

    /// Performs the auto equip consumable if possible operation.
    fn auto_equip_consumable_if_possible(&mut self, key: &str) {
        if self.is_consumable_equipped(key) {
            return;
        }
        if self.equipped_consumables.len() >= Self::MAX_EQUIPPED_CONSUMABLE_TYPES {
            return;
        }
        if matches!(get_equipment(key), Some(Equipment::Consumable(_))) {
            self.equipped_consumables.push(key.to_string());
        }
    }

    /// Returns whether equipped melee.
    pub fn has_equipped_melee(&self) -> bool {
        self.equipped_equipment().iter().any(|eq| {
            if let Equipment::Weapon(w) = eq {
                w.category == Category::Melee
            } else {
                false
            }
        })
    }

    /// Returns whether equipped finesse.
    pub fn has_equipped_finesse(&self) -> bool {
        self.equipped_equipment().iter().any(|eq| {
            if let Equipment::Weapon(w) = eq {
                w.category == Category::Finesse
            } else {
                false
            }
        })
    }

    /// Returns whether equipped range.
    pub fn has_equipped_range(&self) -> bool {
        self.equipped_equipment().iter().any(|eq| {
            if let Equipment::Weapon(w) = eq {
                w.category == Category::Range
            } else {
                false
            }
        })
    }

    /// Performs the training bonus for skill operation.
    pub fn training_bonus_for_skill(&self, skill: &str) -> u32 {
        let mut total = 0;
        if self.has_equipped_melee() {
            total += match skill {
                "attack" => self.training.melee.attack,
                "defense" => self.training.melee.defense,
                "initiative" => self.training.melee.initiative,
                _ => 0,
            };
        }
        if self.has_equipped_finesse() {
            total += match skill {
                "attack" => self.training.finesse.attack,
                "defense" => self.training.finesse.defense,
                "initiative" => self.training.finesse.initiative,
                _ => 0,
            };
        }
        if self.has_equipped_range() {
            total += match skill {
                "attack" => self.training.range.attack,
                "defense" => self.training.range.defense,
                "initiative" => self.training.range.initiative,
                _ => 0,
            };
        }
        total
    }

    /// Returns the heritage attribute modifier when the player is an elf.
    fn elf_heritage_mod(&self, attr: Attribute) -> i32 {
        if self.race == Race::Elf {
            self.elf_heritage.characteristic_mod(attr)
        } else {
            0
        }
    }

    /// Returns the attribute modifier supplied by the player's race and heritage.
    pub fn race_attribute_mod(&self, attr: Attribute) -> i32 {
        self.race.characteristic_mod(attr) + self.elf_heritage_mod(attr)
    }

    /// Returns all applicable direct bonuses from race, heritage, class, specialization, and deity.
    pub fn identity_bonuses(&self) -> IdentityBonuses {
        let mut bonuses = self.race.bonuses() + self.class.bonuses() + self.deity.bonuses();
        if self.race == Race::Elf {
            bonuses += self.elf_heritage.bonuses();
        }
        if self.specialization_is_valid() {
            bonuses += self.specialization.bonuses();
        }

        if self.has_equipped_melee() {
            bonuses.attack += bonuses.melee_attack;
        }
        if self.has_equipped_finesse() {
            bonuses.attack += bonuses.finesse_attack;
        }
        if self.has_equipped_range() {
            bonuses.attack += bonuses.ranged_attack;
        }
        bonuses
    }

    /// Performs the max health operation.
    pub fn max_health(&self) -> u32 {
        let base = 100 + 10 * self.constitution_mod();
        let identity_mod = self.identity_bonuses().max_health;
        let perk_health_mod: i32 = self
            .perks
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
        let equip_health_mod: i32 = self
            .equipped_equipment()
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
        (base + identity_mod + self.bonus_max_health as i32 + perk_health_mod + equip_health_mod)
            .max(1) as u32
    }

    /// Performs the max mana operation.
    pub fn max_mana(&self) -> u32 {
        let base = 100 + 10 * self.wisdom_mod();
        let identity_mod = self.identity_bonuses().max_mana;
        let perk_mana_mod: i32 = self
            .perks
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
        let equip_mana_mod: i32 = self
            .equipped_equipment()
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
        (base + identity_mod + self.bonus_max_mana as i32 + perk_mana_mod + equip_mana_mod).max(0)
            as u32
    }

    /// Health regenerated per second (used during combat).
    pub fn health_regen(&self) -> i32 {
        let base = 2 + (self.constitution_mod() / 2);
        let perk_mod: i32 = self
            .perks
            .iter()
            .filter_map(|key| get_perk(key))
            .flat_map(|perk| perk.modifiers)
            .filter_map(|m| {
                if let Modifier::HealthRegen(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .sum();
        let equip_mod: i32 = self
            .equipped_equipment()
            .iter()
            .flat_map(|eq| eq.modifiers().iter())
            .filter_map(|m| {
                if let Modifier::HealthRegen(v) = m {
                    Some(*v)
                } else {
                    None
                }
            })
            .sum();
        (base + perk_mod + equip_mod + self.identity_bonuses().health_regen).max(0)
    }

    /// Mana regenerated per second (used during combat).
    pub fn mana_regen(&self) -> i32 {
        let base = 2 + (self.wisdom_mod() / 2);
        let perk_mod: i32 = self
            .perks
            .iter()
            .filter_map(|key| get_perk(key))
            .flat_map(|perk| perk.modifiers)
            .filter_map(|m| {
                if let Modifier::ManaRegen(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .sum();
        let equip_mod: i32 = self
            .equipped_equipment()
            .iter()
            .flat_map(|eq| eq.modifiers().iter())
            .filter_map(|m| {
                if let Modifier::ManaRegen(v) = m {
                    Some(*v)
                } else {
                    None
                }
            })
            .sum();
        (base + perk_mod + equip_mod + self.identity_bonuses().mana_regen).max(0)
    }

    /// Effective basic-attack speed (attacks per second), derived from equipped weapons.
    pub fn attack_speed(&self) -> f32 {
        let speeds: Vec<f32> = self
            .equipped_equipment()
            .iter()
            .filter_map(|eq| {
                if let Equipment::Weapon(w) = eq {
                    (!matches!(w.category, Category::Shield | Category::Book))
                        .then_some(w.attack_speed)
                } else {
                    None
                }
            })
            .filter(|s| *s > 0.0)
            .collect();
        let weapon_speed = if speeds.is_empty() {
            1.0
        } else {
            speeds.iter().sum::<f32>() / speeds.len() as f32
        };
        weapon_speed * self.attack_speed_multiplier()
    }

    /// Non-weapon attack-speed multiplier supplied by identity, perks, and equipment.
    pub fn attack_speed_multiplier(&self) -> f32 {
        let modifier_bonus = self
            .active_modifiers()
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::AttackSpeedModifier(percentage) => Some(percentage / 100.0),
                _ => None,
            })
            .sum::<f32>();
        (1.0 + self.identity_bonuses().attack_speed + modifier_bonus).max(0.1)
    }

    /// Non-weapon critical chance supplied by identity, perks, and equipment.
    pub fn non_weapon_crit_chance(&self) -> f32 {
        let modifier_bonus = self
            .active_modifiers()
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::CritChanceModifier(percentage) => Some(percentage / 100.0),
                _ => None,
            })
            .sum::<f32>();
        self.identity_bonuses().crit_chance + modifier_bonus
    }

    /// Flat attack bonus supplied to the player's active pet.
    pub fn pet_attack_bonus(&self) -> i32 {
        self.active_modifiers()
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::PetAttackModifier(value) => Some(*value),
                _ => None,
            })
            .sum()
    }

    /// Flat defense bonus supplied to the player's active pet.
    pub fn pet_defense_bonus(&self) -> i32 {
        self.active_modifiers()
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::PetDefenseModifier(value) => Some(*value),
                _ => None,
            })
            .sum()
    }

    /// Flat initiative bonus supplied to the player's active pet.
    pub fn pet_initiative_bonus(&self) -> i32 {
        self.active_modifiers()
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::PetInitiativeModifier(value) => Some(*value),
                _ => None,
            })
            .sum()
    }

    /// Percentage attack-speed bonus supplied to the player's active pet.
    pub fn pet_attack_speed_multiplier(&self) -> f32 {
        let percentage = self
            .active_modifiers()
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::PetAttackSpeedModifier(value) => Some(*value),
                _ => None,
            })
            .sum::<i32>();
        (1.0 + percentage as f32 / 100.0).max(0.1)
    }

    /// Returns the flat attack bonus supplied by identity choices.
    fn identity_attack_bonus(&self) -> i32 {
        self.identity_bonuses().attack
    }

    /// Returns the equipped-weapon attack modifier supplied by elven heritage.
    pub fn elf_heritage_attack_bonus(&self) -> i32 {
        if self.race == Race::Elf {
            let bonuses = self.elf_heritage.bonuses();
            let ranged_mod = if self.has_equipped_range() {
                bonuses.ranged_attack
            } else {
                0
            };
            let melee_mod = if self.has_equipped_melee() {
                bonuses.melee_attack
            } else {
                0
            };
            ranged_mod + melee_mod
        } else {
            0
        }
    }

    /// Returns the attack modifier supplied by the selected class and specialization.
    pub fn class_attack_bonus(&self) -> i32 {
        if !self.specialization_is_valid() {
            return 0;
        }
        let bonuses = self.specialization.bonuses();
        bonuses.attack
            + i32::from(self.has_equipped_melee()) * bonuses.melee_attack
            + i32::from(self.has_equipped_finesse()) * bonuses.finesse_attack
            + i32::from(self.has_equipped_range()) * bonuses.ranged_attack
    }

    /// Returns the attack modifier supplied by the selected deity.
    pub fn deity_attack_bonus(&self) -> i32 {
        self.deity.bonuses().attack
    }

    /// Returns the flat defense bonus supplied by identity choices.
    fn identity_defense_bonus(&self) -> i32 {
        self.identity_bonuses().defense
    }

    /// Returns the defense modifier supplied by the selected class and specialization.
    pub fn class_defense_bonus(&self) -> i32 {
        if self.specialization_is_valid() {
            self.specialization.bonuses().defense
        } else {
            0
        }
    }

    /// Returns the defense modifier supplied by the selected deity.
    pub fn deity_defense_bonus(&self) -> i32 {
        self.deity.bonuses().defense
    }

    /// Returns the flat initiative bonus supplied by identity choices.
    fn identity_initiative_bonus(&self) -> i32 {
        self.identity_bonuses().initiative
    }

    /// Returns the initiative modifier supplied by the selected deity.
    pub fn deity_initiative_bonus(&self) -> i32 {
        self.deity.bonuses().initiative
    }

    /// Returns whether the selected specialization belongs to the selected class.
    pub fn specialization_is_valid(&self) -> bool {
        match (self.class, self.specialization) {
            (Class::Assassin, ClassSpecialization::Assassin(_))
            | (Class::Druid, ClassSpecialization::Druid(_))
            | (Class::Warrior, ClassSpecialization::Warrior(_))
            | (Class::Monk, ClassSpecialization::Monk(_))
            | (Class::Bard, ClassSpecialization::Bard(_)) => true,
            (Class::Mage(class_ajah), ClassSpecialization::Mage(spec_ajah)) => {
                class_ajah == spec_ajah
            },
            _ => false,
        }
    }

    /// Combined critical-strike chance (0.0-1.0) from equipped weapons.
    pub fn crit_chance(&self) -> f32 {
        let chances: Vec<f32> = self
            .equipped_equipment()
            .iter()
            .filter_map(|eq| {
                if let Equipment::Weapon(w) = eq {
                    Some(w.crit_chance)
                } else {
                    None
                }
            })
            .collect();
        (chances.iter().cloned().fold(0.0_f32, f32::max) + self.non_weapon_crit_chance())
            .clamp(0.0, 1.0)
    }

    /// Performs the attack operation.
    pub fn attack(&self) -> u32 {
        let bonus = self.strength_mod()
            + self.identity_attack_bonus()
            + self.training_bonus_for_skill("attack") as i32
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
                .sum::<i32>();
        Self::apply_combat_bonus(5, bonus)
    }

    /// Performs the defense operation.
    pub fn defense(&self) -> u32 {
        let bonus = self.constitution_mod()
            + self.identity_defense_bonus()
            + self.training_bonus_for_skill("defense") as i32
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
                .sum::<i32>();
        Self::apply_combat_bonus(5, bonus)
    }

    /// Performs the initiative operation.
    pub fn initiative(&self) -> u32 {
        let bonus = self.dexterity_mod()
            + self.identity_initiative_bonus()
            + self.training_bonus_for_skill("initiative") as i32
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
                .sum::<i32>();
        Self::apply_combat_bonus(5, bonus)
    }

    /// Returns the initiative modifier supplied by the selected class and specialization.
    pub fn class_initiative_bonus(&self) -> i32 {
        let specialization_bonus = if self.specialization_is_valid() {
            self.specialization.bonuses().initiative
        } else {
            0
        };
        self.class.bonuses().initiative + specialization_bonus
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
            Race::Halfling => 20.0 + rand_val * 5.0,
            Race::Dragonborn => 32.0 + rand_val * 7.0,
        };

        let weight = (height_m * height_m * bmi).round() as u32;

        (height, weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::classes::{Ajah, AssassinPath, BardStyle, MonkSchool, PetChoice, WarriorPath};

    /// Verifies every field in a player's identity package reaches its derived-stat accessor.
    fn assert_identity_package_is_applied(player: &Player) {
        let bonuses = player.identity_bonuses();
        assert_eq!(
            player.attack(),
            Player::apply_combat_bonus(5, player.strength_mod() + bonuses.attack)
        );
        assert_eq!(
            player.defense(),
            Player::apply_combat_bonus(5, player.constitution_mod() + bonuses.defense)
        );
        assert_eq!(
            player.initiative(),
            Player::apply_combat_bonus(5, player.dexterity_mod() + bonuses.initiative)
        );
        assert_eq!(
            player.max_health(),
            (100 + 10 * player.constitution_mod() + bonuses.max_health).max(1) as u32
        );
        assert_eq!(
            player.max_mana(),
            (100 + 10 * player.wisdom_mod() + bonuses.max_mana).max(0) as u32
        );
        assert_eq!(
            player.health_regen(),
            (2 + player.constitution_mod() / 2 + bonuses.health_regen).max(0)
        );
        assert_eq!(player.mana_regen(), (2 + player.wisdom_mod() / 2 + bonuses.mana_regen).max(0));
        assert!((player.crit_chance() - bonuses.crit_chance).abs() < f32::EPSILON);
        assert!((player.attack_speed() - (1.0 + bonuses.attack_speed)).abs() < f32::EPSILON);
    }

    /// Estimates a complete player's basic-combat output and survival against a level-one foe.
    fn player_combat_rating(player: &Player) -> f32 {
        let attack = player.attack() as f32;
        let defense = player.defense() as f32;
        let initiative = player.initiative() as f32;
        let dodge =
            |attacker: f32, defender: f32| (0.18 + (defender - attacker) * 0.018).clamp(0.08, 0.70);
        let outgoing =
            player.attack_speed() / 2.0 * (1.0 - dodge(initiative, 8.0)) * attack * attack
                / (attack + 7.0)
                * (1.0 + player.crit_chance());
        let incoming = 0.5 * (1.0 - dodge(8.0, initiative)) * 64.0 / (8.0 + defense)
            - 0.3 * player.health_regen() as f32;
        let survival = player.max_health() as f32 / incoming.max(0.1);
        let mana_factor =
            ((player.max_mana() as f32 + 12.0 * player.mana_regen() as f32) / 124.0).powf(0.15);
        outgoing * survival.sqrt() * mana_factor
    }

    #[test]
    /// Verifies that Halfling luck contributes critical-strike chance.
    fn halfling_luck_increases_critical_chance() {
        let player = Player {
            race: Race::Halfling,
            ..default()
        };

        assert!((player.crit_chance() - 0.12).abs() < f32::EPSILON);
    }

    #[test]
    /// Verifies men favor Strength while women favor Charisma.
    fn sex_applies_strength_or_charisma_bonus() {
        assert_eq!(Sex::Man.characteristic_mod(Attribute::Strength), 1);
        assert_eq!(Sex::Man.characteristic_mod(Attribute::Charisma), 0);
        assert_eq!(Sex::Woman.characteristic_mod(Attribute::Strength), 0);
        assert_eq!(Sex::Woman.characteristic_mod(Attribute::Charisma), 1);
    }

    #[test]
    /// Verifies every deity package, including Nyxara's, reaches all derived stats.
    fn every_deity_bonus_is_applied() {
        for deity in Deity::iter() {
            let player = Player {
                class: Class::Druid,
                specialization: ClassSpecialization::Druid(PetChoice::Rat),
                deity,
                ..default()
            };
            assert_identity_package_is_applied(&player);
        }

        let nyxara = Player {
            class: Class::Druid,
            specialization: ClassSpecialization::Druid(PetChoice::Rat),
            deity: Deity::Nyxara,
            ..default()
        };
        assert_eq!(nyxara.attack(), 7);
        assert_eq!(nyxara.max_mana(), 125);
    }

    #[test]
    /// Verifies every race and Elf heritage package reaches all derived-stat accessors.
    fn every_ancestry_bonus_is_applied() {
        for race in Race::iter().filter(|race| *race != Race::Elf) {
            assert_identity_package_is_applied(&Player {
                race,
                class: Class::Druid,
                specialization: ClassSpecialization::Druid(PetChoice::Rat),
                ..default()
            });
        }
        for elf_heritage in ElfHeritage::iter() {
            assert_identity_package_is_applied(&Player {
                race: Race::Elf,
                elf_heritage,
                class: Class::Druid,
                specialization: ClassSpecialization::Druid(PetChoice::Rat),
                ..default()
            });
        }
    }

    #[test]
    /// Verifies every numerical specialization package reaches the derived-stat layer.
    fn every_specialization_bonus_is_applied() {
        let specializations = [
            (Class::Assassin, ClassSpecialization::Assassin(AssassinPath::Nightblade)),
            (Class::Assassin, ClassSpecialization::Assassin(AssassinPath::Venomhand)),
            (Class::Assassin, ClassSpecialization::Assassin(AssassinPath::Duelist)),
            (Class::Assassin, ClassSpecialization::Assassin(AssassinPath::Phantom)),
            (Class::Druid, ClassSpecialization::Druid(PetChoice::Rat)),
            (Class::Mage(Ajah::Black), ClassSpecialization::Mage(Ajah::Black)),
            (Class::Mage(Ajah::Red), ClassSpecialization::Mage(Ajah::Red)),
            (Class::Mage(Ajah::Green), ClassSpecialization::Mage(Ajah::Green)),
            (Class::Mage(Ajah::White), ClassSpecialization::Mage(Ajah::White)),
            (Class::Warrior, ClassSpecialization::Warrior(WarriorPath::Paladin)),
            (Class::Warrior, ClassSpecialization::Warrior(WarriorPath::Templar)),
            (Class::Warrior, ClassSpecialization::Warrior(WarriorPath::Berserker)),
            (Class::Warrior, ClassSpecialization::Warrior(WarriorPath::Warden)),
            (Class::Monk, ClassSpecialization::Monk(MonkSchool::OpenHand)),
            (Class::Monk, ClassSpecialization::Monk(MonkSchool::IronBody)),
            (Class::Monk, ClassSpecialization::Monk(MonkSchool::ShadowStep)),
            (Class::Monk, ClassSpecialization::Monk(MonkSchool::SpiritFist)),
            (Class::Bard, ClassSpecialization::Bard(BardStyle::WarChant)),
            (Class::Bard, ClassSpecialization::Bard(BardStyle::SilverBallad)),
            (Class::Bard, ClassSpecialization::Bard(BardStyle::GraveDirge)),
            (Class::Bard, ClassSpecialization::Bard(BardStyle::WildRhythm)),
        ];

        for (class, specialization) in specializations {
            let player = Player {
                class,
                specialization,
                ..default()
            };
            assert!(player.specialization_is_valid());
            assert_identity_package_is_applied(&player);
        }
    }

    #[test]
    /// Verifies invalid and mismatched specializations cannot leak bonuses into another class.
    fn invalid_specializations_do_not_apply() {
        let invalid = Player {
            class: Class::Warrior,
            specialization: ClassSpecialization::Mage(Ajah::Black),
            ..default()
        };
        let mismatched_ajah = Player {
            class: Class::Mage(Ajah::White),
            specialization: ClassSpecialization::Mage(Ajah::Black),
            ..default()
        };

        assert!(!invalid.specialization_is_valid());
        assert_eq!(invalid.class_attack_bonus(), 0);
        assert!(!mismatched_ajah.specialization_is_valid());
        assert_eq!(mismatched_ajah.class_attack_bonus(), 0);
        assert_eq!(mismatched_ajah.class_defense_bonus(), 0);
    }

    #[test]
    /// Verifies playable ancestry packages stay within a broad hybrid-role combat band.
    fn race_and_heritage_combat_values_are_comparable() {
        let weapon = crate::core::catalog::catalog::all_weapons()
            .iter()
            .find(|weapon| weapon.category == Category::Melee && weapon.level == 1)
            .expect("catalog contains a level-one melee weapon")
            .name
            .clone();
        let ranged_weapon = crate::core::catalog::catalog::all_weapons()
            .iter()
            .find(|weapon| weapon.category == Category::Range && weapon.level == 1)
            .expect("catalog contains a level-one ranged weapon")
            .name
            .clone();
        let mut players = Race::iter()
            .filter(|race| *race != Race::Elf)
            .map(|race| Player {
                race,
                class: Class::Druid,
                specialization: ClassSpecialization::Druid(PetChoice::Rat),
                deity: Deity::Serapha,
                weapon_lh: Some(weapon.clone()),
                ..default()
            })
            .collect::<Vec<_>>();
        players.extend(ElfHeritage::iter().map(|elf_heritage| {
            let weapon = if elf_heritage == ElfHeritage::Wood {
                ranged_weapon.clone()
            } else {
                weapon.clone()
            };
            Player {
                race: Race::Elf,
                elf_heritage,
                class: Class::Druid,
                specialization: ClassSpecialization::Druid(PetChoice::Rat),
                deity: Deity::Serapha,
                weapon_lh: Some(weapon),
                ..default()
            }
        }));
        let ratings = players.iter().map(player_combat_rating).collect::<Vec<_>>();
        let minimum = ratings.iter().copied().fold(f32::INFINITY, f32::min);
        let maximum = ratings.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        assert!(maximum / minimum <= 1.25, "race ratings diverged: {ratings:?}");
    }

    #[test]
    /// Verifies aging trades Constitution for an equal amount of Wisdom.
    fn age_stages_trade_constitution_for_wisdom() {
        for (stage, constitution, wisdom) in [
            (AgeStage::Youth, 2, -2),
            (AgeStage::YoungAdult, 1, -1),
            (AgeStage::Adult, 0, 0),
            (AgeStage::Senior, -1, 1),
            (AgeStage::Elder, -2, 2),
        ] {
            assert_eq!(stage.characteristic_mod(Attribute::Constitution), constitution);
            assert_eq!(stage.characteristic_mod(Attribute::Wisdom), wisdom);
            assert_eq!(constitution + wisdom, 0);
        }
    }

    #[test]
    /// Verifies that attack speed is the Monk's only derived-stat bonus.
    fn monk_has_only_an_attack_speed_bonus() {
        let player = Player {
            class: Class::Monk,
            wisdom: START_CHARACTERISTIC + 2,
            ..default()
        };
        let baseline = Player {
            class: Class::Warrior,
            wisdom: START_CHARACTERISTIC + 2,
            ..default()
        };

        assert_eq!(player.max_mana(), baseline.max_mana());
        assert_eq!(player.mana_regen(), baseline.mana_regen());
        assert_eq!(player.initiative(), baseline.initiative());
        assert_eq!(player.defense(), baseline.defense());
        assert!((player.attack_speed() - 1.1).abs() < f32::EPSILON);
        assert!((baseline.attack_speed() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    /// Verifies that High Elves gain intelligence without an overvalued attack penalty.
    fn high_elf_favors_intelligence() {
        let high_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::High,
            ..default()
        };
        let dark_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::Dark,
            ..default()
        };

        assert_eq!(high_elf.intelligence(), dark_elf.intelligence().saturating_add(1));
        assert_eq!(high_elf.strength(), dark_elf.strength());
    }

    #[test]
    /// Verifies that Dark Elves trade maximum mana for critical-strike chance.
    fn dark_elf_trades_mana_for_critical_chance() {
        let dark_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::Dark,
            ..default()
        };
        let wood_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::Wood,
            ..default()
        };

        assert_eq!(dark_elf.max_mana().saturating_add(10), wood_elf.max_mana());
        assert!((dark_elf.crit_chance() - wood_elf.crit_chance() - 0.08).abs() < f32::EPSILON);
    }

    #[test]
    /// Verifies that Wood Elves favor ranged weapons over melee weapons.
    fn wood_elf_trades_melee_attack_for_ranged_attack() {
        let ranged_weapon = crate::core::catalog::catalog::all_weapons()
            .iter()
            .find(|weapon| weapon.category == Category::Range)
            .expect("catalog contains a ranged weapon")
            .name
            .clone();
        let melee_weapon = crate::core::catalog::catalog::all_weapons()
            .iter()
            .find(|weapon| weapon.category == Category::Melee)
            .expect("catalog contains a melee weapon")
            .name
            .clone();
        let ranged_wood_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::Wood,
            weapon_lh: Some(ranged_weapon.clone()),
            ..default()
        };
        let ranged_dark_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::Dark,
            weapon_lh: Some(ranged_weapon),
            ..default()
        };
        let melee_wood_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::Wood,
            weapon_lh: Some(melee_weapon.clone()),
            ..default()
        };
        let melee_dark_elf = Player {
            race: Race::Elf,
            elf_heritage: ElfHeritage::Dark,
            weapon_lh: Some(melee_weapon),
            ..default()
        };

        assert_eq!(ranged_wood_elf.attack(), ranged_dark_elf.attack().saturating_add(1));
        assert_eq!(melee_wood_elf.attack().saturating_add(1), melee_dark_elf.attack());
    }

    #[test]
    /// Verifies that elven heritage effects never leak to another race.
    fn elf_heritage_effects_require_elf_ancestry() {
        let high_heritage_human = Player {
            race: Race::Human,
            elf_heritage: ElfHeritage::High,
            ..default()
        };
        let dark_heritage_human = Player {
            race: Race::Human,
            elf_heritage: ElfHeritage::Dark,
            ..default()
        };
        let wood_heritage_human = Player {
            race: Race::Human,
            elf_heritage: ElfHeritage::Wood,
            ..default()
        };

        assert_eq!(high_heritage_human.strength(), dark_heritage_human.strength());
        assert_eq!(high_heritage_human.intelligence(), dark_heritage_human.intelligence());
        assert_eq!(dark_heritage_human.max_mana(), wood_heritage_human.max_mana());
        assert_eq!(dark_heritage_human.crit_chance(), wood_heritage_human.crit_chance());
    }

    #[test]
    /// Verifies that Venomhand and Duelist bonuses require their distinct weapon categories.
    fn assassin_paths_favor_distinct_weapon_categories() {
        let ranged_weapon = crate::core::catalog::catalog::all_weapons()
            .iter()
            .find(|weapon| weapon.category == Category::Range)
            .expect("catalog contains a ranged weapon")
            .name
            .clone();
        let finesse_weapon = crate::core::catalog::catalog::all_weapons()
            .iter()
            .find(|weapon| weapon.category == Category::Finesse)
            .expect("catalog contains a finesse weapon")
            .name
            .clone();

        let ranged_baseline = Player {
            class: Class::Assassin,
            specialization: ClassSpecialization::Assassin(AssassinPath::Phantom),
            weapon_lh: Some(ranged_weapon),
            ..default()
        };
        let ranged_venomhand = Player {
            specialization: ClassSpecialization::Assassin(AssassinPath::Venomhand),
            ..ranged_baseline.clone()
        };
        let ranged_duelist = Player {
            specialization: ClassSpecialization::Assassin(AssassinPath::Duelist),
            ..ranged_baseline.clone()
        };

        assert_eq!(ranged_venomhand.attack(), ranged_baseline.attack().saturating_add(1));
        assert_eq!(ranged_duelist.attack(), ranged_baseline.attack());

        let finesse_baseline = Player {
            weapon_lh: Some(finesse_weapon),
            ..ranged_baseline
        };
        let finesse_venomhand = Player {
            specialization: ClassSpecialization::Assassin(AssassinPath::Venomhand),
            ..finesse_baseline.clone()
        };
        let finesse_duelist = Player {
            specialization: ClassSpecialization::Assassin(AssassinPath::Duelist),
            ..finesse_baseline.clone()
        };

        assert_eq!(finesse_venomhand.attack(), finesse_baseline.attack());
        assert_eq!(finesse_duelist.attack(), finesse_baseline.attack().saturating_add(1));
    }

    #[test]
    /// Verifies gameplay portraits retain their selected specialization.
    fn specialization_portraits_include_selected_path() {
        let assassin = Player {
            sex: Sex::Woman,
            race: Race::Elf,
            class: Class::Assassin,
            specialization: ClassSpecialization::Assassin(AssassinPath::Venomhand),
            ..default()
        };
        let bard = Player {
            race: Race::Dragonborn,
            class: Class::Bard,
            specialization: ClassSpecialization::Bard(BardStyle::GraveDirge),
            ..default()
        };
        let templar = Player {
            race: Race::Halfling,
            class: Class::Warrior,
            specialization: ClassSpecialization::Warrior(WarriorPath::Templar),
            ..default()
        };

        assert_eq!(assassin.portrait_key(), "assassin_venomhand_elf_woman");
        assert_eq!(bard.portrait_key(), "bard_grave_dirge_dragonborn_man");
        assert_eq!(templar.portrait_key(), "warrior_templar_halfling_man");
    }

    #[test]
    /// Verifies that Warrior callings apply their advertised tradeoffs.
    fn warrior_callings_change_combat_stats() {
        let paladin = Player {
            class: Class::Warrior,
            specialization: ClassSpecialization::Warrior(WarriorPath::Paladin),
            ..default()
        };
        let berserker = Player {
            specialization: ClassSpecialization::Warrior(WarriorPath::Berserker),
            ..paladin.clone()
        };
        let templar = Player {
            specialization: ClassSpecialization::Warrior(WarriorPath::Templar),
            ..paladin.clone()
        };

        assert_eq!(berserker.attack(), paladin.attack().saturating_add(1));
        assert_eq!(berserker.defense().saturating_add(1), paladin.defense());
        assert_eq!(templar.defense(), paladin.defense().saturating_add(3));
    }

    #[test]
    /// Verifies that divine patrons supply distinct derived-stat bonuses.
    fn deity_bonuses_affect_derived_stats() {
        let balanced = Player {
            deity: Deity::Tharos,
            ..default()
        };
        let tyrant = Player {
            deity: Deity::Kharos,
            ..balanced.clone()
        };
        let hearthmother = Player {
            deity: Deity::Serapha,
            ..balanced.clone()
        };

        assert_eq!(tyrant.attack(), balanced.attack().saturating_add(2));
        assert_eq!(tyrant.defense().saturating_add(3), balanced.defense());
        assert_eq!(hearthmother.health_regen(), balanced.health_regen().saturating_add(1));
    }
}
