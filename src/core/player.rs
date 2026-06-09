use crate::core::classes::Class;
use crate::core::constants::FANTASY_NAMES;
use crate::core::pets::Pet;
use crate::core::races::Race;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use strum_macros::{Display, EnumIter, EnumString};

#[derive(EnumIter, Clone, Copy, Debug, Display, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    pub bonus_max_health: f32,
    pub bonus_max_mana: f32,
}

impl Default for Player {
    fn default() -> Self {
        let name = FANTASY_NAMES.choose(&mut rng()).copied().unwrap().to_string();

        // Generate a random age in the Adult stage (stage 2) for default Human race
        let race = Race::default();
        let (min_age, max_age) = race.age_stage_range(2);
        use rand::RngExt;
        let age = rand::rng().random_range(min_age..=max_age);

        Self {
            name,
            sex: Sex::default(),
            race,
            class: Class::default(),
            age,
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
            bonus_max_health: 0.,
            bonus_max_mana: 0.,
        }
    }
}

impl Player {
    /// Get the age stage (0=Youth, 1=Young Adult, 2=Adult, 3=Senior, 4=Elder) from actual age
    pub fn age_stage(&self) -> u32 {
        let (min, max) = self.race.age_range();
        let span = max - min + 1;
        let age_offset = self.age.saturating_sub(min);
        let stage = (age_offset * 5) / span;
        stage.clamp(0, 4)
    }

    /// Age stage modifier: Youth=-2, Young Adult=-1, Adult=0, Senior=+1, Elder=+2.
    /// This is added to wisdom and subtracted from constitution.
    pub fn age_modifier(&self) -> i16 {
        (self.age_stage() as i16) - 2
    }

    pub fn strength(&self) -> u8 {
        let base = self.strength as i16;
        let modifier = self.race.modifier(Attribute::Strength) as i16;
        let sex_mod = if matches!(self.sex, Sex::Male) {
            1
        } else {
            0
        };
        (base + modifier + sex_mod).max(0) as u8
    }

    pub fn dexterity(&self) -> u8 {
        let base = self.dexterity as i16;
        let modifier = self.race.modifier(Attribute::Dexterity) as i16;
        (base + modifier).max(0) as u8
    }

    pub fn constitution(&self) -> u8 {
        let base = self.constitution as i16;
        let modifier = self.race.modifier(Attribute::Constitution) as i16;
        (base + modifier - self.age_modifier()).max(0) as u8
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
        let sex_mod = if matches!(self.sex, Sex::Female) {
            1
        } else {
            0
        };
        (base + modifier + sex_mod).max(0) as u8
    }

    /// All currently equipped pieces of gear.
    pub fn equipped_equipment(&self) -> Vec<crate::core::catalog::GeneratedEquipment> {
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
        .filter_map(|key| crate::core::catalog::get_equipment(key))
        .collect()
    }

    pub fn max_health(&self) -> f32 {
        let base_max = 100. + (self.constitution() as f32 - 10.) * 10.;
        let class_bonus = if matches!(self.class, Class::Warrior) {
            20.
        } else {
            0.
        };
        base_max + class_bonus + self.bonus_max_health
    }

    pub fn max_mana(&self) -> f32 {
        let mut base_max = 100.0_f32;
        match self.class {
            Class::Mage(_) => base_max += 30.,
            Class::Druid => base_max += 10.,
            _ => {},
        }
        let wisdom_bonus = (self.wisdom() as i32 - 10) * 10;
        (base_max + wisdom_bonus as f32 + self.bonus_max_mana).max(0.)
    }

    /// Total physical attack damage (base from strength plus weapon bonuses).
    pub fn attack_damage(&self) -> i32 {
        let str_bonus = (self.strength() as i32 - 10) + 5; // base 5 + 1 per str above 10
        str_bonus + self.equipped_equipment().iter().map(|w| w.attack).sum::<i32>()
    }

    pub fn weapon_attack_speed(&self, weapon_key: &str) -> f32 {
        let weapon_speed =
            crate::core::catalog::get_equipment(weapon_key).map(|w| w.attack_speed).unwrap_or(1.0);
        self.adjust_attack_speed(weapon_speed)
    }

    fn adjust_attack_speed(&self, weapon_speed: f32) -> f32 {
        let dex_bonus = (self.dexterity() as f32 - 10.) * 0.05;
        (weapon_speed + dex_bonus).max(0.3)
    }

    /// Total armor rating (base from constitution plus equipment bonuses).
    pub fn armor_value(&self) -> i32 {
        self.constitution() as i32 / 4
            + self.equipped_equipment().iter().map(|w| w.armor).sum::<i32>()
    }

    /// Initiative determines turn order (base from dexterity plus equipment bonuses).
    pub fn initiative(&self) -> i32 {
        let base_init = self.dexterity() as i32 / 2
            + self.equipped_equipment().iter().map(|w| w.initiative).sum::<i32>();
        if matches!(self.class, Class::Rogue) {
            base_init + 2
        } else {
            base_init
        }
    }

    /// (height_cm, weight_kg). Height and weight are derived deterministically from name and race.
    pub fn vitals(&self) -> (u32, u32) {
        let (_age_r, height_r, _weight_r) = self.race.vital_ranges();

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
