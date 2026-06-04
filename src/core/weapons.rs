use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weapon {
    // Helmets
    IronHelmet,
    // Armors
    IronChestplate,
    MageRobes,
    LeatherArmor,
    LeafyGarb,
    // Boots
    IronBoots,
    ClothShoes,
    SilentBoots,
    LeatherBoots,
    // One Handed Weapons
    SteelSword,
    IronShield,
    AssassinDagger,
    ThiefDagger,
    OakWand,
    // Two Handed Weapons
    WizardStaff,
}
