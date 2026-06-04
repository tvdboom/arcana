use std::collections::HashMap;

use bevy::prelude::*;
use serde_json;

use crate::core::settings::{Language, Settings};

#[derive(Resource)]
pub struct Localization {
    en: HashMap<String, String>,
    es: HashMap<String, String>,
}

impl FromWorld for Localization {
    fn from_world(_world: &mut World) -> Self {
        let en = serde_json::from_str(include_str!("../../assets/language/en.json"))
            .expect("Failed to parse en.json");
        let es = serde_json::from_str(include_str!("../../assets/language/es.json"))
            .expect("Failed to parse es.json");
        Self { en, es }
    }
}

impl Localization {
    pub fn get(&self, key: &str, language: Language) -> String {
        let map = match language {
            Language::English => &self.en,
            Language::Spanish => &self.es,
        };
        map.get(key).cloned().unwrap_or_else(|| {
            warn!("Missing localization key: '{key}'");
            key.to_string()
        })
    }
}

/// Marks a text entity with the localization key so it can be updated on language change.
#[derive(Component)]
pub struct LocalizedText(pub String);

/// Updates all LocalizedText entities whenever the Settings resource changes.
pub fn update_localized_text(
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut text_q: Query<(&mut Text, &LocalizedText)>,
) {
    for (mut text, loc) in &mut text_q {
        text.0 = localization.get(&loc.0, settings.language);
    }
}
