use crate::core::abilities::Ability;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    Druid,
    Mage,
    Rogue,
    #[default]
    Warrior,
}

impl Class {
    pub fn starting_ability(&self) -> Ability {
        match self {
            Class::Warrior => Ability::Slash,
            Class::Mage => Ability::Firebolt,
            Class::Rogue => Ability::Backstab,
            Class::Druid => Ability::Regrowth,
        }
    }
}
