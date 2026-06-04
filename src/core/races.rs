use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use crate::core::player::Attribute;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    Dwarf,
    Elf,
    #[default]
    Human,
    Orc,
}

impl Race {
    pub fn modifier(&self, attr: Attribute) -> f32 {
        match attr {
            Attribute::Strength => {
                match self {
                    Race::Dwarf => 1.0,
                    Race::Elf => -2.0,
                    Race::Human => 0.0,
                    Race::Orc => 2.0,
                }
            },
            Attribute::Dexterity => {
                match self {
                    Race::Dwarf => -1.0,
                    Race::Elf => 2.0,
                    Race::Human => 1.0,
                    Race::Orc => 0.0,
                }
            },
            Attribute::Constitution => {
                match self {
                    Race::Dwarf => 2.0,
                    Race::Elf => -1.0,
                    Race::Human => 0.0,
                    Race::Orc => 2.0,
                }
            },
            Attribute::Intelligence => {
                match self {
                    Race::Dwarf => 0.0,
                    Race::Elf => 1.0,
                    Race::Human => 0.0,
                    Race::Orc => -1.0,
                }
            },
            Attribute::Wisdom => {
                match self {
                    Race::Dwarf => 1.0,
                    Race::Elf => 1.0,
                    Race::Human => 0.0,
                    Race::Orc => 0.0,
                }
            },
            Attribute::Charisma => {
                match self {
                    Race::Dwarf => -1.0,
                    Race::Elf => 1.0,
                    Race::Human => 1.0,
                    Race::Orc => -1.0,
                }
            }
        }
    }
}
