use std::collections::HashMap;

use crate::core::classes::{Ajah, Class};
use crate::core::monsters::MonsterKind;
use crate::core::ui::creation::PetChoice;
use crate::core::player::Attribute;
use crate::core::races::Race;
use crate::core::settings::{Language, Settings};
use crate::utils::NameFromEnum;
use crate::utils::capitalize_words;
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

fn map_localization_key(key: &str) -> String {
    let lower = key.to_lowercase();
    if lower.contains('.') {
        let parts: Vec<&str> = lower.splitn(2, '.').collect();
        return format!("{}.{}", parts[0], parts[1].replace(" ", "_"));
    }

    // Check attributes
    if ["strength", "dexterity", "constitution", "intelligence", "wisdom", "charisma"]
        .contains(&lower.as_str())
    {
        return format!("attribute.{}", lower);
    }
    // Check races
    if ["human", "human_desc", "elf", "elf_desc", "dwarf", "dwarf_desc", "orc", "orc_desc"]
        .contains(&lower.as_str())
    {
        return format!("race.{}", lower);
    }
    // Check classes
    if [
        "warrior",
        "warrior_desc",
        "mage",
        "mage_desc",
        "assassin",
        "assassin_desc",
        "druid",
        "druid_desc",
    ]
    .contains(&lower.as_str())
    {
        return format!("class.{}", lower);
    }
    // Check ajahs
    if ["black", "black_desc", "red", "red_desc", "green", "green_desc", "white", "white_desc"]
        .contains(&lower.as_str())
    {
        return format!("ajah.{}", lower);
    }
    // Check pets
    let pets = [
        "wolf",
        "bear",
        "snake",
        "eagle",
        "bat",
        "crocodile",
        "hell hound",
        "hyena",
        "lizard",
        "pegasus",
        "rat",
        "spider",
        "three headed dog",
        "tiger",
        "unicorn",
        "vulture",
        "puma",
        "griffin",
        "manticore",
    ];
    if pets.iter().any(|&p| lower == p || lower == format!("{}_desc", p)) {
        let normalized = lower.replace(" ", "_");
        return format!("pet.{}", normalized);
    }

    // Default to general
    let normalized = lower.replace(" ", "_");
    format!("general.{}", normalized)
}

#[allow(dead_code)]
fn get_custom_localization(_key: &str, _language: Language) -> Option<String> {
    None
}

impl Localization {
    pub fn get(&self, key: impl Into<String>, language: Language) -> String {
        let key = key.into();
        let mapped_key = map_localization_key(&key);
        let map = match language {
            Language::English => &self.en,
            Language::Spanish => &self.es,
            Language::Dutch => &self.nl,
        };
        if let Some(val) = map.get(&mapped_key) {
            return val.clone();
        }
        panic!("Missing localization key: '{}' (mapped from '{}')", mapped_key, key)
    }

    pub fn get_opt(&self, key: &str, language: Language) -> Option<String> {
        let mapped_key = map_localization_key(key);
        let map = match language {
            Language::English => &self.en,
            Language::Spanish => &self.es,
            Language::Dutch => &self.nl,
        };
        map.get(&mapped_key).cloned()
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
pub struct LocalizedPetDesc(pub PetChoice);

/// Marks a text entity with the monster kind text so it can be updated on language change.
#[derive(Component)]
pub struct LocalizedMonsterKindDesc(pub MonsterKind);

pub fn format_race_description(
    race: Race,
    language: Language,
    localization: &Localization,
) -> String {
    let race_key = race.to_lowername();
    let desc = localization.get(format!("race.{}_desc", race_key), language);

    let mut modifier_strs = Vec::new();
    for attr in Attribute::iter() {
        let val = race.characteristic_mod(attr);
        if val != 0 {
            let attr_name =
                localization.get(format!("attribute.{}", attr.to_lowername()), language);
            modifier_strs.push(format!("  {val:+} {attr_name}"));
        }
    }

    if modifier_strs.is_empty() {
        desc
    } else {
        format!("{}\n\n{}", desc, modifier_strs.join("\n"))
    }
}

pub fn format_class_description(
    class: Class,
    language: Language,
    localization: &Localization,
) -> String {
    let desc = localization.get(format!("class.{}_desc", class.to_lowername()), language);

    let physical_label = localization.get("general.physical", language);
    let magical_label = localization.get("general.magical", language);
    let ability_label = localization.get("general.ability", language);
    let perk_label = localization.get("general.perk", language);
    let weapon_label = localization.get("general.weapon", language);
    let bonus_desc = match class {
        Class::Assassin => {
            let finesse_label = localization.get("general.finesse", language);
            let init_label = localization.get("general.initiative", language);
            format!(" +1 {physical_label} {ability_label}\n +1 {finesse_label} {weapon_label}\n +1 {perk_label}\n +2 {init_label}")
        },
        Class::Druid => {
            let nature_label = localization.get("general.nature", language);
            let pet_label = localization.get("general.pet", language);
            format!(" +1 {magical_label} {ability_label} ({nature_label})\n +1 {magical_label} {weapon_label}\n +1 {perk_label}\n +1 {pet_label}")
        },
        Class::Mage(_) => {
            let mp_label = localization.get("general.mana", language);
            format!(" +1 {magical_label} {ability_label}\n +1 {magical_label} {weapon_label}\n +1 {perk_label}\n +30 max {mp_label}")
        },
        Class::Warrior => {
            let melee_label = localization.get("general.melee", language);
            let hp_label = localization.get("general.health", language);
            format!(" +1 {physical_label} {ability_label}\n +1 {melee_label} {weapon_label}\n +1 {perk_label}\n +20 max {hp_label}")
        },
    };

    format!("{desc}\n\n{}", bonus_desc.to_lowercase())
}

pub fn format_ajah_description(
    ajah: Ajah,
    language: Language,
    localization: &Localization,
) -> String {
    let desc = localization.get(format!("ajah.{}_desc", ajah.to_lowername()), language);

    let ability_label = localization.get("general.ability", language);
    let damage_label = localization.get("general.damage", language);
    let kind_label = localization.get(format!("general.{}", ajah.kind().to_lowername()), language);
    let bonus_desc = format!(" +1 {kind_label} {ability_label}\n +20% {kind_label} {damage_label}");

    format!("{desc}\n\n{}", bonus_desc.to_lowercase())
}

pub fn format_pet_description(
    pet: PetChoice,
    language: Language,
    localization: &Localization,
) -> String {
    let pet_key = pet.to_lowername();
    localization.get(format!("pet.{}_desc", pet_key), language)
}

pub fn format_monster_kind_description(
    kind: MonsterKind,
    _language: Language,
    _localization: &Localization,
) -> String {
    capitalize_words(&kind.to_lowername())
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
    mut monster_kind_desc_q: Query<
        (&mut Text, &LocalizedMonsterKindDesc),
        (
            Without<LocalizedText>,
            Without<LocalizedRaceDesc>,
            Without<LocalizedClassDesc>,
            Without<LocalizedAjahDesc>,
            Without<LocalizedPetDesc>,
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

    for (mut text, desc) in &mut monster_kind_desc_q {
        text.0 = format_monster_kind_description(desc.0, settings.language, &localization);
    }
}
