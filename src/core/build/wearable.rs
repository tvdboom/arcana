use crate::core::build::effects::Effect;
use crate::core::build::equipment::Kind;
use crate::core::build::modifiers::Modifier;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use crate::utils::NameFromEnum;
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
    /// Name of the wearable (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the armor corresponds to
    pub image: String,

    /// Level or upgrade tier of the wearable
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
    pub fn description(&self, _language: Language, _localization: &Localization) -> String {
        let mut parts = vec![
            format!("[level]{}", self.level),
            format!("[{}]", self.kind.to_lowername()),
        ];
        if !self.modifiers.is_empty() {
            parts.push(format!("[modifier]{}", self.modifiers.len()));
        }
        if !self.effects.is_empty() {
            parts.push(format!("[ability]{}", self.effects.len()));
        }
        parts.join(" ")
    }

    pub fn full_description(&self, language: Language, localization: &Localization) -> Vec<String> {
        let mut lines = Vec::new();
        let level_label = localization.get("general.level", language);
        let kind_label = localization.get("general.kind", language);
        let modifiers_label = localization.get("general.modifiers", language);
        let abilities_label = localization.get("general.abilities", language);

        let kind_name = localization.get(format!("general.{}", self.kind.to_lowername()), language);

        lines.push(format!("[level] {}: {}", level_label, self.level));
        lines.push(format!("[{}] {}: {}", self.kind.to_lowername(), kind_label, kind_name));

        if !self.modifiers.is_empty() {
            lines.push(format!("[modifier] {}:", modifiers_label));
            for m in &self.modifiers {
                lines.push(format!("• {}", m.description(language, localization)));
            }
        }
        if !self.effects.is_empty() {
            lines.push(format!("[ability] {}:", abilities_label));
            for e in &self.effects {
                lines.push(format!("• {}", e.description(language, localization)));
            }
        }
        lines
    }
}
