use crate::core::inventory::effects::Effect;
use crate::core::inventory::equipment::Kind;
use crate::core::inventory::modifiers::Modifier;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum EquipmentSlot {
    Accessory,
    Helmet,
    Chestplate,
    Gloves,
    Boots,
    Consumable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Armor {
    /// Name of the armor piece (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the armor corresponds to
    pub image: String,

    /// Kind of armor
    pub kind: Kind,

    /// Level or upgrade tier of the armor piece
    pub level: u32,

    /// Gold value for buying and selling at merchants
    pub price: u32,

    /// Determines which slot this protective piece occupies on the character sheet
    pub slot: EquipmentSlot,

    /// Flat raw mitigation rating added to defensive calculations
    pub base_defense: u32,

    /// Static attribute modifiers applied directly to the player while equipped
    pub modifiers: Vec<Modifier>,

    /// Optional passive effect triggered when struck by an enemy
    pub effects: Vec<Effect>,
}
