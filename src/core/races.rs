use crate::core::player::Attribute;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    Dwarf,
    Elf,
    #[default]
    Human,
    Orc,
}

impl Race {
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
