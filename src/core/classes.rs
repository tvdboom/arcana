use crate::core::abilities::Ability;
use crate::core::perks::Perk;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    Druid,
    Mage(Ajah),
    Rogue,
    #[default]
    Warrior,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ajah {
    #[default]
    Black,
    Red,
    Green,
    White,
}

impl Ajah {
    pub fn special_ability(&self) -> Ability {
        match self {
            Ajah::Black => Ability::Shadowbolt,
            Ajah::Red => Ability::Firebolt,
            Ajah::Green => Ability::Regrowth,
            Ajah::White => Ability::Frostbolt,
        }
    }
}

impl Class {
    pub fn starting_ability(&self) -> Ability {
        match self {
            Class::Warrior => Ability::Slash,
            Class::Mage(_) => Ability::Heal,
            Class::Rogue => Ability::Backstab,
            Class::Druid => Ability::Regrowth,
        }
    }

    pub fn starting_perk(&self) -> Perk {
        match self {
            Class::Warrior => Perk::IronSkin,
            Class::Mage(_) => Perk::ArcaneFlow,
            Class::Rogue => Perk::FleetFooted,
            Class::Druid => Perk::WildBond,
        }
    }
}
