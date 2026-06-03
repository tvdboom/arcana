use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    pub fn key(self) -> &'static str {
        match self {
            Language::English => "english",
            Language::Spanish => "spanish",
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Localizer {
    pub language: Language,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Default for Localizer {
    fn default() -> Self {
        Self {
            language: Language::English,
            translations: load_translations(),
        }
    }
}

impl Localizer {
    pub fn t(&self, key: &str) -> String {
        self.translations
            .get(self.language.key())
            .and_then(|language| language.get(key))
            .or_else(|| {
                self.translations
                    .get(Language::English.key())
                    .and_then(|language| language.get(key))
            })
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn format(&self, key: &str, replacements: &[(&str, String)]) -> String {
        let mut text = self.t(key);
        for (placeholder, value) in replacements {
            text = text.replace(&format!("{{{placeholder}}}"), value);
        }
        text
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }
}

fn load_translations() -> HashMap<String, HashMap<String, String>> {
    let mut path = PathBuf::from("assets");
    path.push("i18n");
    path.push("translations.json");

    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_else(fallback_translations)
}

fn fallback_translations() -> HashMap<String, HashMap<String, String>> {
    serde_json::from_str(include_str!("../../assets/i18n/translations.json"))
        .unwrap_or_default()
}
