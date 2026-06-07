use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use bevy::prelude::default;

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

/// Combat bonuses provided by a piece of equipment.
#[derive(Clone, Copy, Debug, Default)]
pub struct WeaponStats {
    pub attack: i32,
    pub armor: i32,
    pub crit: i32,
    pub initiative: i32,
}

impl Weapon {
    pub fn stats(&self) -> WeaponStats {        match self {
            // Helmets
            Weapon::IronHelmet => WeaponStats { armor: 2, ..default() },
            // Armors
            Weapon::IronChestplate => WeaponStats { armor: 5, initiative: -1, ..default() },
            Weapon::MageRobes => WeaponStats { armor: 1, ..default() },
            Weapon::LeatherArmor => WeaponStats { armor: 3, ..default() },
            Weapon::LeafyGarb => WeaponStats { armor: 2, initiative: 1, ..default() },
            // Boots
            Weapon::IronBoots => WeaponStats { armor: 1, ..default() },
            Weapon::ClothShoes => WeaponStats { initiative: 2, ..default() },
            Weapon::SilentBoots => WeaponStats { initiative: 3, crit: 2, ..default() },
            Weapon::LeatherBoots => WeaponStats { armor: 1, initiative: 2, ..default() },
            // One handed weapons
            Weapon::SteelSword => WeaponStats { attack: 6, ..default() },
            Weapon::IronShield => WeaponStats { armor: 4, ..default() },
            Weapon::AssassinDagger => WeaponStats { attack: 4, crit: 10, ..default() },
            Weapon::ThiefDagger => WeaponStats { attack: 3, crit: 6, ..default() },
            Weapon::OakWand => WeaponStats { attack: 3, ..default() },
            // Two handed weapons
            Weapon::WizardStaff => WeaponStats { attack: 5, crit: 3, ..default() },
        }
    }

    /// The icon asset key used to display this piece of equipment.
    pub fn image_key(&self) -> &'static str {
        match self {
            Weapon::IronHelmet => "helmet_icon",
            Weapon::IronChestplate | Weapon::MageRobes | Weapon::LeatherArmor | Weapon::LeafyGarb => {
                "armor_icon"
            },
            Weapon::IronBoots | Weapon::ClothShoes | Weapon::SilentBoots | Weapon::LeatherBoots => {
                "boots_icon"
            },
            Weapon::IronShield => "shield",
            Weapon::SteelSword | Weapon::AssassinDagger | Weapon::ThiefDagger | Weapon::OakWand
            | Weapon::WizardStaff => "sword",
        }
    }
}
