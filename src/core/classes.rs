//! Playable class and Ajah definitions together with their gameplay properties.

use crate::core::catalog::equipment::Kind;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    Assassin,
    Druid,
    Mage(Ajah),
    #[default]
    Warrior,
    Monk,
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
    /// Performs the kind operation.
    pub fn kind(&self) -> Kind {
        match self {
            Ajah::Black => Kind::Shadow,
            Ajah::Green => Kind::Nature,
            Ajah::Red => Kind::Fire,
            Ajah::White => Kind::Ice,
        }
    }
}
