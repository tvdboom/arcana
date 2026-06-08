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
    pub fn special_ability(&self) -> &'static str {
        match self {
            Ajah::Black => "novice ethereal cosmic flash",
            Ajah::Red => "novice infused pyro blast",
            Ajah::Green => "novice noble static nova",
            Ajah::White => "novice vile mana barrier",
        }
    }
}

impl Class {
    pub fn starting_ability(&self) -> &'static str {
        match self {
            Class::Warrior => "novice swift cleaving strike",
            Class::Mage(_) => "novice ashen lightning touch",
            Class::Rogue => "novice clandestine devious slash",
            Class::Druid => "novice sovereign sunfire howl",
        }
    }

    pub fn starting_perk(&self) -> &'static str {
        match self {
            Class::Warrior => "novice swift vanguard mastery",
            Class::Mage(_) => "novice infused acolyte resilience",
            Class::Rogue => "novice clandestine stalker reflexes",
            Class::Druid => "novice sovereign dryad harmony",
        }
    }

    pub fn starting_weapon(&self) -> &'static str {
        match self {
            Class::Warrior => "novice bronze mighty bow",
            Class::Mage(_) => "novice worn primal axe",
            Class::Rogue => "overlord mighty primal dagger",
            Class::Druid => "novice worn primal axe",
        }
    }
}
