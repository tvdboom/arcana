use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perk {
    IronSkin,
    ArcaneFlow,
    FleetFooted,
    WildBond,
}
