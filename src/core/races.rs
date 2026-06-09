use crate::core::player::Attribute;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    #[default]
    Human,
    Elf,
    Dwarf,
    Orc,
}

impl Race {
    /// Plausible (min, max) age range in years for this race.
    pub fn age_range(&self) -> (u32, u32) {
        match self {
            Race::Human => (16, 80),
            Race::Elf => (100, 1350),
            Race::Dwarf => (60, 400),
            Race::Orc => (16, 60),
        }
    }

    pub fn age_stage_range(&self, stage: u32) -> (u32, u32) {
        let (min, max) = self.age_range();
        let span = max - min + 1;
        let stage = stage.clamp(0, 4);
        let start = min + (span * stage) / 5;
        let end = min + (span * (stage + 1)) / 5 - 1;
        (start, end.max(start))
    }

    /// Plausible (min, max) ranges for age (years), height (cm) and weight (kg).
    pub fn vital_ranges(&self) -> ((u32, u32), (u32, u32), (u32, u32)) {
        match self {
            Race::Dwarf => (self.age_range(), (120, 150), (60, 95)),
            Race::Elf => (self.age_range(), (170, 200), (50, 75)),
            Race::Human => (self.age_range(), (160, 190), (60, 95)),
            Race::Orc => (self.age_range(), (180, 220), (90, 145)),
        }
    }

    pub fn modifier(&self, attr: Attribute) -> i8 {
        match attr {
            Attribute::Strength => match self {
                Race::Dwarf => 1,
                Race::Elf => -2,
                Race::Human => 0,
                Race::Orc => 2,
            },
            Attribute::Dexterity => match self {
                Race::Dwarf => -1,
                Race::Elf => 2,
                Race::Human => 1,
                Race::Orc => 0,
            },
            Attribute::Constitution => match self {
                Race::Dwarf => 2,
                Race::Elf => -1,
                Race::Human => 0,
                Race::Orc => 2,
            },
            Attribute::Intelligence => match self {
                Race::Dwarf => 0,
                Race::Elf => 1,
                Race::Human => 0,
                Race::Orc => -1,
            },
            Attribute::Wisdom => match self {
                Race::Dwarf => 1,
                Race::Elf => 1,
                Race::Human => 0,
                Race::Orc => 0,
            },
            Attribute::Charisma => match self {
                Race::Dwarf => -1,
                Race::Elf => 1,
                Race::Human => 1,
                Race::Orc => -1,
            },
        }
    }
}
