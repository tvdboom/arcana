use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ability {
    Slash,
    Firebolt,
    Backstab,
    Regrowth,
    Heal,
    ShieldBash,
    Shadowbolt,
    Frostbolt,
    Bite,
    Maul,
    PoisonBite,
    Swoop,
}
