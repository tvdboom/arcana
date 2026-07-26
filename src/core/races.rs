//! Playable races and their attribute, aging, and descriptive properties.

use crate::core::identity::IdentityBonuses;
use crate::core::player::{AgeStage, Attribute};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    #[default]
    Human,
    Elf,
    Dwarf,
    Orc,
    Halfling,
    Dragonborn,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElfHeritage {
    #[default]
    High,
    Dark,
    Wood,
}

/// A supernatural form acquired after surviving an eligible monster encounter.
#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutation {
    Werewolf,
    Wererat,
    Werebear,
    Vampire,
    Undead,
}

impl Mutation {
    /// Returns the mutation's permanent attribute adjustment.
    pub fn characteristic_mod(self, attr: Attribute) -> i32 {
        match (self, attr) {
            (Self::Werewolf, Attribute::Constitution) => 4,
            (Self::Werewolf, Attribute::Wisdom) => -4,
            (Self::Wererat, Attribute::Dexterity) => 4,
            (Self::Wererat, Attribute::Charisma) => -4,
            (Self::Werebear, Attribute::Strength) => 4,
            (Self::Werebear, Attribute::Intelligence) => -4,
            (Self::Undead, Attribute::Constitution) => -4,
            _ => 0,
        }
    }

    /// Maps a mutation-bearing monster name to the form it can transmit.
    pub fn from_monster_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "werewolf" => Some(Self::Werewolf),
            "wererat" => Some(Self::Wererat),
            "werebear" => Some(Self::Werebear),
            "vampire" => Some(Self::Vampire),
            "lich" => Some(Self::Undead),
            _ => None,
        }
    }
}

impl Race {
    /// Plausible (min, max) age range in years for this race.
    pub fn age_range(&self) -> (u32, u32) {
        match self {
            Race::Human => (16, 80),
            Race::Elf => (100, 1350),
            Race::Dwarf => (60, 400),
            Race::Orc => (16, 60),
            Race::Halfling => (20, 160),
            Race::Dragonborn => (15, 110),
        }
    }

    /// Performs the age stage range operation.
    pub fn age_stage_range(&self, stage: AgeStage) -> (u32, u32) {
        let (min, max) = self.age_range();
        let span = max - min + 1;
        let start = min + (span * stage.index()) / 5;
        let end = min + (span * (stage.index() + 1)) / 5 - 1;
        (start, end.max(start))
    }

    /// Plausible (min, max) ranges for age (years), height (cm) and weight (kg).
    pub fn vital_ranges(&self) -> ((u32, u32), (u32, u32), (u32, u32)) {
        match self {
            Race::Dwarf => (self.age_range(), (120, 150), (60, 95)),
            Race::Elf => (self.age_range(), (170, 200), (50, 75)),
            Race::Human => (self.age_range(), (160, 190), (60, 95)),
            Race::Orc => (self.age_range(), (180, 220), (90, 145)),
            Race::Halfling => (self.age_range(), (95, 125), (22, 45)),
            Race::Dragonborn => (self.age_range(), (185, 225), (105, 165)),
        }
    }

    /// Performs the characteristic mod operation.
    pub fn characteristic_mod(&self, attr: Attribute) -> i32 {
        match attr {
            Attribute::Strength => match self {
                Race::Dwarf => 0,
                Race::Elf => 0,
                Race::Human => 0,
                Race::Orc => 1,
                Race::Halfling => -1,
                Race::Dragonborn => 1,
            },
            Attribute::Dexterity => match self {
                Race::Dwarf => -1,
                Race::Elf => 0,
                Race::Human => 1,
                Race::Orc => -1,
                Race::Halfling => 2,
                Race::Dragonborn => -1,
            },
            Attribute::Constitution => match self {
                Race::Dwarf => 1,
                Race::Elf => 0,
                Race::Human => 0,
                Race::Orc => 1,
                Race::Halfling => 0,
                Race::Dragonborn => 0,
            },
            Attribute::Intelligence => match self {
                Race::Dwarf => 0,
                Race::Elf => 1,
                Race::Human => 0,
                Race::Orc => -1,
                Race::Halfling => 0,
                Race::Dragonborn => -1,
            },
            Attribute::Wisdom => match self {
                Race::Dwarf => 1,
                Race::Elf => 1,
                Race::Human => 0,
                Race::Orc => 0,
                Race::Halfling => 0,
                Race::Dragonborn => 0,
            },
            Attribute::Charisma => match self {
                Race::Dwarf => 0,
                Race::Elf => 1,
                Race::Human => 1,
                Race::Orc => -1,
                Race::Halfling => 1,
                Race::Dragonborn => 1,
            },
        }
    }

    /// Returns the race's direct bonuses beyond its attribute adjustments.
    pub fn bonuses(self) -> IdentityBonuses {
        match self {
            Race::Halfling => IdentityBonuses {
                crit_chance: 0.12,
                ..Default::default()
            },
            Race::Dragonborn => IdentityBonuses {
                max_health: 10,
                ..Default::default()
            },
            Race::Human | Race::Elf | Race::Dwarf | Race::Orc => IdentityBonuses::default(),
        }
    }
}

impl ElfHeritage {
    /// Returns the attribute adjustment supplied by this elven heritage.
    pub fn characteristic_mod(self, attr: Attribute) -> i32 {
        match (self, attr) {
            (ElfHeritage::High, Attribute::Intelligence) => 1,
            _ => 0,
        }
    }

    /// Returns this heritage's direct bonuses beyond its attribute adjustments.
    pub fn bonuses(self) -> IdentityBonuses {
        match self {
            ElfHeritage::High => IdentityBonuses::default(),
            ElfHeritage::Dark => IdentityBonuses {
                max_mana: -10,
                crit_chance: 0.08,
                ..Default::default()
            },
            ElfHeritage::Wood => IdentityBonuses {
                melee_attack: -1,
                ranged_attack: 1,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Verifies that Halfling attributes favor agility and charm.
    fn halfling_has_agility_and_charisma_adjustments() {
        let race = Race::Halfling;

        assert_eq!(race.characteristic_mod(Attribute::Strength), -1);
        assert_eq!(race.characteristic_mod(Attribute::Dexterity), 2);
        assert_eq!(race.characteristic_mod(Attribute::Constitution), 0);
        assert_eq!(race.characteristic_mod(Attribute::Intelligence), 0);
        assert_eq!(race.characteristic_mod(Attribute::Wisdom), 0);
        assert_eq!(race.characteristic_mod(Attribute::Charisma), 1);
    }

    #[test]
    /// Verifies that Dragonborn trade agility and intellect for strength and endurance.
    fn dragonborn_has_draconic_attribute_adjustments() {
        let race = Race::Dragonborn;

        assert_eq!(race.characteristic_mod(Attribute::Strength), 1);
        assert_eq!(race.characteristic_mod(Attribute::Dexterity), -1);
        assert_eq!(race.characteristic_mod(Attribute::Constitution), 0);
        assert_eq!(race.characteristic_mod(Attribute::Intelligence), -1);
        assert_eq!(race.characteristic_mod(Attribute::Charisma), 1);
        assert_eq!(race.bonuses().max_health, 10);
    }

    #[test]
    /// Verifies that Elf ancestry no longer changes Strength or Dexterity.
    fn elf_has_no_base_strength_or_dexterity_adjustment() {
        let race = Race::Elf;

        assert_eq!(race.characteristic_mod(Attribute::Strength), 0);
        assert_eq!(race.characteristic_mod(Attribute::Dexterity), 0);
    }

    #[test]
    /// Verifies that elven heritage profiles remain restrained and role-specific.
    fn elf_heritages_have_balanced_tradeoffs() {
        assert_eq!(ElfHeritage::High.characteristic_mod(Attribute::Intelligence), 1);
        assert_eq!(ElfHeritage::High.characteristic_mod(Attribute::Strength), 0);

        assert!((ElfHeritage::Dark.bonuses().crit_chance - 0.08).abs() < f32::EPSILON);
        assert_eq!(ElfHeritage::Dark.bonuses().max_mana, -10);

        assert_eq!(ElfHeritage::Wood.bonuses().ranged_attack, 1);
        assert_eq!(ElfHeritage::Wood.bonuses().melee_attack, -1);
    }

    #[test]
    /// Verifies mutation encounters and their balanced attribute tradeoffs.
    fn mutations_map_from_monsters_and_adjust_attributes() {
        assert_eq!(Mutation::from_monster_name("Werewolf"), Some(Mutation::Werewolf));
        assert_eq!(Mutation::from_monster_name("Lich"), Some(Mutation::Undead));
        assert_eq!(Mutation::from_monster_name("Goblin"), None);

        assert_eq!(Mutation::Werebear.characteristic_mod(Attribute::Strength), 4);
        assert_eq!(Mutation::Werebear.characteristic_mod(Attribute::Intelligence), -4);
        assert_eq!(Mutation::Wererat.characteristic_mod(Attribute::Dexterity), 4);
        assert_eq!(Mutation::Wererat.characteristic_mod(Attribute::Charisma), -4);
    }
}
