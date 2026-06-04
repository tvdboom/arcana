use std::collections::HashMap;

use crate::core::classes::Class;
use crate::core::player::Attribute;
use crate::core::races::Race;
use crate::core::settings::{Language, Settings};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use serde_json;
use strum::IntoEnumIterator;

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
        Self {
            en,
            es,
        }
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

/// Marks a text entity with the race description so it can be updated with modifiers on language change.
#[derive(Component)]
pub struct LocalizedRaceDesc(pub Race);

/// Marks a text entity with the class description so it can be updated with modifiers on language change.
#[derive(Component)]
pub struct LocalizedClassDesc(pub Class);

pub fn format_race_description(
    race: Race,
    language: Language,
    localization: &Localization,
) -> String {
    let race_key = race.to_lowername();
    let desc = localization.get(&format!("{}_desc", race_key), language);

    let mut modifier_strs = Vec::new();
    for attr in Attribute::iter() {
        let val = race.modifier(attr);
        if val != 0 {
            let sign = if val > 0 {
                "+"
            } else {
                ""
            };
            let attr_name = localization.get(attr.to_lowername().as_str(), language);
            modifier_strs.push(format!("{} {}{}", attr_name, sign, val));
        }
    }

    if modifier_strs.is_empty() {
        desc
    } else {
        let modifiers_label = localization.get("modifiers", language);
        format!("{}\n\n{}: {}", desc, modifiers_label, modifier_strs.join(", "))
    }
}

pub fn format_class_description(
    class: Class,
    language: Language,
    localization: &Localization,
) -> String {
    let class_key = class.to_lowername();
    let desc = localization.get(&format!("{}_desc", class_key), language);

    let starting_ability = class.starting_ability();
    let ability_key = starting_ability.to_lowername();
    let ability_name = localization.get(&ability_key, language);
    let ability_desc = localization.get(&format!("{}_desc", ability_key), language);

    let starting_ability_label = localization.get("starting_ability", language);

    format!("{}\n\n{}: {} - {}", desc, starting_ability_label, ability_name, ability_desc)
}

/// Updates all LocalizedText and LocalizedRaceDesc entities whenever the Settings resource changes.
pub fn update_localized_text(
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut text_q: Query<(&mut Text, &LocalizedText)>,
    mut desc_q: Query<(&mut Text, &LocalizedRaceDesc), Without<LocalizedText>>,
    mut class_desc_q: Query<
        (&mut Text, &LocalizedClassDesc),
        (Without<LocalizedText>, Without<LocalizedRaceDesc>),
    >,
) {
    for (mut text, loc) in &mut text_q {
        text.0 = localization.get(&loc.0, settings.language);
    }

    for (mut text, desc) in &mut desc_q {
        text.0 = format_race_description(desc.0, settings.language, &localization);
    }

    for (mut text, desc) in &mut class_desc_q {
        text.0 = format_class_description(desc.0, settings.language, &localization);
    }
}
