use std::collections::HashMap;

use crate::core::classes::{Ajah, Class};
use crate::core::pets::Pet;
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
    nl: HashMap<String, String>,
}

impl FromWorld for Localization {
    fn from_world(_world: &mut World) -> Self {
        let en = serde_json::from_str(include_str!("../../assets/language/en.json"))
            .expect("Failed to parse en.json");
        let es = serde_json::from_str(include_str!("../../assets/language/es.json"))
            .expect("Failed to parse es.json");
        let nl = serde_json::from_str(include_str!("../../assets/language/nl.json"))
            .expect("Failed to parse nl.json");

        Self {
            en,
            es,
            nl,
        }
    }
}

impl Localization {
    pub fn get(&self, key: &str, language: Language) -> String {
        let map = match language {
            Language::English => &self.en,
            Language::Spanish => &self.es,
            Language::Dutch => &self.nl,
        };
        map.get(key).cloned().unwrap_or_else(|| {
            warn!("Missing localization key: '{key}'");
            key.to_string()
        })
    }

    pub fn get_opt(&self, key: &str, language: Language) -> Option<String> {
        let map = match language {
            Language::English => &self.en,
            Language::Spanish => &self.es,
            Language::Dutch => &self.nl,
        };
        map.get(key).cloned()
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

/// Marks a text entity with the ajah description so it can be updated with modifiers on language change.
#[derive(Component)]
pub struct LocalizedAjahDesc(pub Ajah);

/// Marks a text entity with the pet description so it can be updated on language change.
#[derive(Component)]
pub struct LocalizedPetDesc(pub Pet);

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
    let ability_name = crate::core::catalog::get_ability(starting_ability)
        .map(|a| a.name)
        .unwrap_or("Ability");

    let starting_perk = class.starting_perk();
    let perk_name = crate::core::catalog::get_perk(starting_perk)
        .map(|p| p.name)
        .unwrap_or("Perk");

    let starting_weapon = class.starting_weapon();
    let weapon_name = crate::core::catalog::get_equipment(starting_weapon)
        .map(|w| w.name)
        .unwrap_or("Weapon");

    let starting_ability_label = localization.get("starting_ability", language);
    let starting_perk_label = localization.get("starting_perk", language);
    let starting_weapon_label = localization.get("starting_weapon", language);

    let bonus_desc = match class {
        Class::Warrior => {
            let hp_label = localization.get("health", language);
            format!("Max {}: +20", hp_label)
        },
        Class::Mage(_) => {
            let mana_label = localization.get("mana", language);
            format!("Max {}: +30", mana_label)
        },
        Class::Rogue => {
            let init_label = localization.get("initiative", language);
            format!("{}: +2", init_label)
        },
        Class::Druid => {
            let pet_label = localization.get("pet", language);
            let choose_pet_label = localization.get("choose pet", language);
            format!("{}: {}", pet_label, choose_pet_label)
        },
    };

    format!(
        "{}\n\n{}: {}\n{}: {}\n{}: {}\n\n{}",
        desc,
        starting_ability_label,
        ability_name,
        starting_perk_label,
        perk_name,
        starting_weapon_label,
        weapon_name,
        bonus_desc
    )
}

pub fn format_ajah_description(
    ajah: Ajah,
    language: Language,
    localization: &Localization,
) -> String {
    let ajah_key = ajah.to_lowername();
    let desc = localization.get(&format!("{}_desc", ajah_key), language);

    let special_ability = ajah.special_ability();
    let ability_name = crate::core::catalog::get_ability(special_ability)
        .map(|a| a.name)
        .unwrap_or("Ability");

    let special_ability_label = localization.get("special_ability", language);

    format!("{}\n\n{}: {}", desc, special_ability_label, ability_name)
}

pub fn format_pet_description(pet: Pet, language: Language, localization: &Localization) -> String {
    let pet_key = pet.to_lowername();
    localization.get(&format!("{}_desc", pet_key), language)
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
    mut ajah_desc_q: Query<
        (&mut Text, &LocalizedAjahDesc),
        (Without<LocalizedText>, Without<LocalizedRaceDesc>, Without<LocalizedClassDesc>),
    >,
    mut pet_desc_q: Query<
        (&mut Text, &LocalizedPetDesc),
        (
            Without<LocalizedText>,
            Without<LocalizedRaceDesc>,
            Without<LocalizedClassDesc>,
            Without<LocalizedAjahDesc>,
        ),
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

    for (mut text, desc) in &mut ajah_desc_q {
        text.0 = format_ajah_description(desc.0, settings.language, &localization);
    }

    for (mut text, desc) in &mut pet_desc_q {
        text.0 = format_pet_description(desc.0, settings.language, &localization);
    }
}
