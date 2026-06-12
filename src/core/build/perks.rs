use crate::core::build::modifiers::Modifier;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Perk {
    /// Name of the perk (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the perk corresponds to
    pub image: String,

    /// Level of the perk
    pub level: u32,

    /// Passive modifiers that are always applied
    pub modifiers: Vec<Modifier>,
}

impl Perk {
    pub fn description(&self, _language: Language, _localization: &Localization) -> String {
        format!("[level]{} [modifier]{}", self.level, self.modifiers.len())
    }

    pub fn full_description(&self, language: Language, localization: &Localization) -> Vec<String> {
        let mut lines = Vec::new();
        let level_label = localization.get("general.level", language);
        let modifiers_label = localization.get("general.modifiers", language);

        lines.push(format!("[level] {}: {}", level_label, self.level));
        lines.push(format!("[modifier] {}:", modifiers_label));
        for m in &self.modifiers {
            lines.push(format!("• {}", m.description(language, localization)));
        }
        lines
    }
}
