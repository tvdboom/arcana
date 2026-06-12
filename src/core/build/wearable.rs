use crate::core::build::effects::Effect;
use crate::core::build::equipment::Kind;
use crate::core::build::modifiers::Modifier;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum WearableSlot {
    Accessory,
    Helmet,
    Chestplate,
    Gloves,
    Boots,
    Consumable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Wearable {
    /// Name of the armor piece (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the armor corresponds to
    pub image: String,

    /// Level or upgrade tier of the armor piece
    pub level: u32,

    /// Kind of armor
    pub kind: Kind,

    /// Gold value for buying and selling at merchants
    pub price: u32,

    /// Determines which slot this protective piece occupies on the character sheet
    pub slot: WearableSlot,

    /// Static attribute modifiers applied directly to the player while equipped
    pub modifiers: Vec<Modifier>,

    /// Optional passive effect triggered when struck by an enemy
    pub effects: Vec<Effect>,
}

impl Wearable {
    pub fn description(&self, language: Language, localization: &Localization) -> String {
        let mut parts = Vec::new();
        for m in &self.modifiers {
            parts.push(m.description(language, localization));
        }
        for e in &self.effects {
            parts.push(e.description(language, localization));
        }
        if parts.is_empty() {
            "Protective gear".to_string()
        } else {
            parts.join(" | ")
        }
    }
}
