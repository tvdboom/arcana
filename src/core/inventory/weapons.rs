use serde::Deserialize;
use crate::core::inventory::effects::Effect;
use crate::core::inventory::equipment::EquipmentKind;
use crate::core::inventory::modifiers::Modifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Hand {
    OneHand,
    TwoHand,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Weapon {
    /// Name of the weapon (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the weapon corresponds to
    pub image: String,

    /// Description key in the language files
    pub desc_key: String,

    /// Kind of weapon
    pub kind: EquipmentKind,

    /// Whether the weapon is carried in one or two hands
    pub hand: Hand,
    
    /// Level or upgrade tier of the weapon
    pub level: u32,

    /// Gold value for buying and selling
    pub price: u32,

    /// Flat raw attack rating added to offensive calculations
    pub base_attack: u32,

    /// Attack interval pacing (e.g., 1.5 base weapon swings per second)
    pub attack_speed: f32,

    /// Base critical strike chance (e.g., 0.05 for a +5% chance)
    pub crit_chance: f32,

    /// Static attribute modifiers applied directly to the player while equipped
    pub modifiers: Vec<Modifier>,

    /// Effects triggered on landing a successful hit
    pub effect: Vec<Effect>,
}
