use crate::core::catalog::effects::Effect;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Consumable {
    /// Name of the consumable (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the armor corresponds to
    pub image: String,

    /// Level or upgrade tier of the wearable
    pub level: u32,

    /// Gold value for buying and selling at merchants
    pub price: u32,

    /// Optional passive effect triggered when struck by an enemy
    pub effects: Vec<Effect>,

    /// List of artifact names required to craft this consumable
    pub craft: Vec<String>,
}

impl Consumable {
    pub fn description(&self, _language: Language, _localization: &Localization) -> String {
        let mut parts = vec![format!("[level]{}", self.level)];
        if !self.effects.is_empty() {
            parts.push(format!("[ability]{}", self.effects.len()));
        }
        parts.join(" ")
    }

    pub fn full_description(&self, language: Language, localization: &Localization) -> Vec<String> {
        let mut lines = Vec::new();
        let level_label = localization.get("general.level", language);
        let abilities_label = localization.get("general.abilities", language);

        lines.push(format!("[level] {}: {}", level_label, self.level));

        if !self.effects.is_empty() {
            lines.push(format!("[ability] {}:", abilities_label));
            for e in &self.effects {
                lines.push(format!("• {}", e.description(language, localization)));
            }
        }
        lines
    }
}
