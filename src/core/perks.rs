use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perk {
    IronSkin,
    ArcaneFlow,
    FleetFooted,
    WildBond,
}

impl Perk {
    pub fn level(&self) -> u8 {
        match self {
            Perk::IronSkin => 1,
            Perk::ArcaneFlow => 1,
            Perk::FleetFooted => 1,
            Perk::WildBond => 1,
        }
    }
}
