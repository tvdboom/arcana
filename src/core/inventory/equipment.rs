use serde::Deserialize;
use crate::core::inventory::armor::Armor;
use crate::core::inventory::weapons::Weapon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum EquipmentKind {
    Martial,
    Bulwark,
    Assassination,
    Skirmish,
    Tactic,
    Command,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Equipment {
    Armor(Armor),
    Weapon(Weapon),
}
