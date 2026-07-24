//! Playable races and their attribute, aging, and descriptive properties.

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
        }
    }

    /// Performs the characteristic mod operation.
    pub fn characteristic_mod(&self, attr: Attribute) -> i32 {
        match attr {
            Attribute::Strength => match self {
                Race::Dwarf => 1,
                Race::Elf => -2,
                Race::Human => 0,
                Race::Orc => 2,
                Race::Halfling => -1,
            },
            Attribute::Dexterity => match self {
                Race::Dwarf => -1,
                Race::Elf => 2,
                Race::Human => 1,
                Race::Orc => 0,
                Race::Halfling => 2,
            },
            Attribute::Constitution => match self {
                Race::Dwarf => 2,
                Race::Elf => -1,
                Race::Human => 0,
                Race::Orc => 2,
                Race::Halfling => 0,
            },
            Attribute::Intelligence => match self {
                Race::Dwarf => 0,
                Race::Elf => 1,
                Race::Human => 0,
                Race::Orc => -1,
                Race::Halfling => 0,
            },
            Attribute::Wisdom => match self {
                Race::Dwarf => 1,
                Race::Elf => 1,
                Race::Human => 0,
                Race::Orc => 0,
                Race::Halfling => 0,
            },
            Attribute::Charisma => match self {
                Race::Dwarf => -1,
                Race::Elf => 1,
                Race::Human => 1,
                Race::Orc => -1,
                Race::Halfling => 1,
            },
        }
    }

    /// Additional critical-strike chance granted by this race.
    pub fn crit_chance_bonus(&self) -> f32 {
        match self {
            Race::Halfling => 0.03,
            _ => 0.0,
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
}
