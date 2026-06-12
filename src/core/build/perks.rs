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
    pub fn description(&self, language: Language, localization: &Localization) -> String {
        let mut parts = Vec::new();
        for m in &self.modifiers {
            parts.push(m.description(language, localization));
        }
        if parts.is_empty() {
            "Passive boost".to_string()
        } else {
            parts.join(" | ")
        }
    }
}
