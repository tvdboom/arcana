//! Localized strings and helpers for translating game concepts and interface text.

use std::collections::HashMap;

use crate::core::catalog::catalog::get_monster;
use crate::core::classes::{Ajah, Class, ClassSpecialization, PetChoice};
use crate::core::deities::{Deity, EthicalAlignment, MoralAlignment};
use crate::core::identity::IdentityBonuses;
use crate::core::monsters::MonsterKind;
use crate::core::player::Attribute;
use crate::core::races::{ElfHeritage, Race};
use crate::core::settings::{Language, Settings};
use crate::utils::capitalize_words;
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
    /// Performs the from world operation.
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

/// Performs the map localization key operation.
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
    if [
        "human",
        "human_desc",
        "elf",
        "elf_desc",
        "dwarf",
        "dwarf_desc",
        "orc",
        "orc_desc",
        "halfling",
        "halfling_desc",
        "halfling_luck",
        "dragonborn",
        "dragonborn_desc",
    ]
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
        "monk",
        "monk_desc",
        "bard",
        "bard_desc",
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
/// Returns custom localization.
fn get_custom_localization(_key: &str, _language: Language) -> Option<String> {
    None
}

impl Localization {
    /// Performs the get operation.
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

    /// Returns opt.
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

/// Marks a text entity with an Elf heritage description.
#[derive(Component)]
pub struct LocalizedElfHeritageDesc(pub ElfHeritage);

/// Marks a text entity with a class-specialization description.
#[derive(Component)]
pub struct LocalizedSpecializationDesc(pub ClassSpecialization);

/// Marks a text entity with the monster kind text so it can be updated on language change.
#[derive(Component)]
pub struct LocalizedMonsterKindDesc(pub MonsterKind);

/// Removes an old inline bonus sentence now rendered as structured bullet points.
fn description_prose(description: String) -> String {
    description.split_once(" +").map_or(description.clone(), |(prose, _)| prose.to_string())
}

/// Appends consistently formatted gameplay bonus bullets to descriptive prose.
fn description_with_bonuses(description: String, bonuses: Vec<String>) -> String {
    let description = description_prose(description);
    if bonuses.is_empty() {
        return description;
    }
    let bullets =
        bonuses.into_iter().map(|bonus| format!("• {bonus}")).collect::<Vec<_>>().join("\n");
    format!("{description}\n\n{bullets}")
}

/// Formats a signed modifier using a localized stat label.
fn stat_bonus(
    value: i32,
    stat_key: &str,
    language: Language,
    localization: &Localization,
) -> String {
    format!("{value:+} {}", localization.get(stat_key, language))
}

#[derive(Clone, Copy)]
enum MaximumPool {
    Health,
    Mana,
}

/// Formats a maximum-pool modifier using the localized max-stat label.
fn maximum_bonus(
    value: i32,
    pool: MaximumPool,
    language: Language,
    localization: &Localization,
) -> String {
    let max_stat_key = match pool {
        MaximumPool::Health => "general.max_health",
        MaximumPool::Mana => "general.max_mana",
    };
    format!("{value:+} {}", localization.get(max_stat_key, language))
}

/// Formats every nonzero field in a shared identity bonus package.
fn identity_bonus_descriptions(
    bonuses: IdentityBonuses,
    language: Language,
    localization: &Localization,
) -> Vec<String> {
    let mut descriptions = Vec::new();
    for (value, key) in [
        (bonuses.attack, "general.attack"),
        (bonuses.defense, "general.defense"),
        (bonuses.initiative, "general.initiative"),
    ] {
        if value != 0 {
            descriptions.push(stat_bonus(value, key, language, localization));
        }
    }
    for (value, category_key) in [
        (bonuses.melee_attack, "general.melee"),
        (bonuses.finesse_attack, "general.finesse"),
        (bonuses.ranged_attack, "general.range"),
    ] {
        if value != 0 {
            descriptions.push(format!(
                "{value:+} {} ({})",
                localization.get("general.attack", language),
                localization.get(category_key, language)
            ));
        }
    }
    for (value, pool) in
        [(bonuses.max_health, MaximumPool::Health), (bonuses.max_mana, MaximumPool::Mana)]
    {
        if value != 0 {
            descriptions.push(maximum_bonus(value, pool, language, localization));
        }
    }
    for (value, key) in
        [(bonuses.health_regen, "general.health_regen"), (bonuses.mana_regen, "general.mana_regen")]
    {
        if value != 0 {
            descriptions.push(stat_bonus(value, key, language, localization));
        }
    }
    if bonuses.crit_chance != 0.0 {
        descriptions.push(format!(
            "{:+.0}% {}",
            bonuses.crit_chance * 100.0,
            localization.get("general.crit_chance", language)
        ));
    }
    if bonuses.attack_speed != 0.0 {
        descriptions.push(format!(
            "{:+.0}% {}",
            bonuses.attack_speed * 100.0,
            localization.get("general.attack_speed", language)
        ));
    }
    descriptions
}

/// Formats race description.
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
            modifier_strs.push(format!("{val:+} {attr_name}"));
        }
    }
    modifier_strs.extend(identity_bonus_descriptions(race.bonuses(), language, localization));

    description_with_bonuses(desc, modifier_strs)
}

/// Formats class description.
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
    let mut bonuses = match class {
        Class::Assassin => {
            let finesse_label = localization.get("general.finesse", language);
            vec![
                format!("+1 {physical_label} {ability_label}"),
                format!("+1 {finesse_label} {weapon_label}"),
                format!("+1 {perk_label}"),
            ]
        },
        Class::Druid => {
            let nature_label = localization.get("general.nature", language);
            let pet_label = localization.get("general.pet", language);
            vec![
                format!("+1 {magical_label} {ability_label} ({nature_label})"),
                format!("+1 {magical_label} {weapon_label}"),
                format!("+1 {perk_label}"),
                format!("+1 {pet_label}"),
            ]
        },
        Class::Mage(_) => {
            vec![
                format!("+1 {magical_label} {ability_label}"),
                format!("+1 {magical_label} {weapon_label}"),
                format!("+1 {perk_label}"),
            ]
        },
        Class::Warrior => {
            let melee_label = localization.get("general.melee", language);
            vec![
                format!("+1 {physical_label} {ability_label}"),
                format!("+1 {melee_label} {weapon_label}"),
                format!("+1 {perk_label}"),
            ]
        },
        Class::Monk => {
            let finesse_label = localization.get("general.finesse", language);
            vec![
                format!("+1 {physical_label} {ability_label}"),
                format!("+1 {finesse_label} {weapon_label}"),
                format!("+1 {perk_label}"),
            ]
        },
        Class::Bard => {
            vec![
                format!("+1 {magical_label} {ability_label}"),
                format!("+1 {magical_label} {weapon_label}"),
                format!("+1 {perk_label}"),
            ]
        },
    };
    bonuses.extend(identity_bonus_descriptions(class.bonuses(), language, localization));

    description_with_bonuses(desc, bonuses)
}

/// Formats an Elf heritage's description and bonuses.
pub fn format_elf_heritage_description(
    heritage: ElfHeritage,
    language: Language,
    localization: &Localization,
) -> String {
    let description =
        localization.get(format!("heritage.{}_desc", heritage.to_lowername()), language);
    let mut bonuses = Attribute::iter()
        .filter_map(|attribute| {
            let value = heritage.characteristic_mod(attribute);
            (value != 0).then(|| {
                stat_bonus(
                    value,
                    &format!("attribute.{}", attribute.to_lowername()),
                    language,
                    localization,
                )
            })
        })
        .collect::<Vec<_>>();
    bonuses.extend(identity_bonus_descriptions(heritage.bonuses(), language, localization));
    description_with_bonuses(description, bonuses)
}

/// Formats a class specialization's description and bonuses.
pub fn format_specialization_description(
    specialization: ClassSpecialization,
    language: Language,
    localization: &Localization,
) -> String {
    let key = match specialization {
        ClassSpecialization::Assassin(path) => path.to_lowername(),
        ClassSpecialization::Druid(pet) => {
            return format_pet_description(pet, language, localization);
        },
        ClassSpecialization::Mage(ajah) => {
            return format_ajah_description(ajah, language, localization);
        },
        ClassSpecialization::Warrior(path) => path.to_lowername(),
        ClassSpecialization::Monk(school) => school.to_lowername(),
        ClassSpecialization::Bard(style) => style.to_lowername(),
    };
    let bonuses = identity_bonus_descriptions(specialization.bonuses(), language, localization);
    let description = localization.get(format!("specialization.{}_desc", key), language);
    description_with_bonuses(description, bonuses)
}

/// Formats ajah description.
pub fn format_ajah_description(
    ajah: Ajah,
    language: Language,
    localization: &Localization,
) -> String {
    let desc = localization.get(format!("ajah.{}_desc", ajah.to_lowername()), language);

    let ability_label = localization.get("general.ability", language);
    let kind_label = localization.get(format!("general.{}", ajah.kind().to_lowername()), language);
    let mut bonuses = vec![format!("+1 {kind_label} {ability_label}")];
    bonuses.extend(identity_bonus_descriptions(ajah.bonuses(), language, localization));
    description_with_bonuses(desc, bonuses)
}

/// Formats pet description.
pub fn format_pet_description(
    pet: PetChoice,
    language: Language,
    localization: &Localization,
) -> String {
    let pet_key = pet.to_lowername();
    let description = localization.get(format!("pet.{}_desc", pet_key), language);
    let bonuses = get_monster(pet.monster_name()).map_or_else(Vec::new, |monster| {
        vec![
            format!("{}: {}", localization.get("general.health", language), monster.max_health),
            format!("{}: {}", localization.get("general.attack", language), monster.attack),
            format!("{}: {}", localization.get("general.defense", language), monster.defense),
            format!("{}: {}", localization.get("general.initiative", language), monster.initiative),
        ]
    });
    description_with_bonuses(description, bonuses)
}

/// Formats a deity's lore and gameplay bonuses as a card description.
pub fn format_deity_description(
    deity: Deity,
    language: Language,
    localization: &Localization,
) -> String {
    let description = localization.get(format!("deity.{}_desc", deity.to_lowername()), language);
    let description = description
        .split_once('•')
        .map_or(description.clone(), |(_, lore)| lore.trim_start().to_string());
    let bonuses = identity_bonus_descriptions(deity.bonuses(), language, localization);
    description_with_bonuses(description, bonuses)
}

/// Formats a deity's localized ethical and moral alignment without its name.
pub fn format_deity_alignment(
    deity: Deity,
    language: Language,
    localization: &Localization,
) -> String {
    if deity.ethical_alignment() == EthicalAlignment::Neutral
        && deity.moral_alignment() == MoralAlignment::Neutral
    {
        return localization.get("alignment.true_neutral", language);
    }

    let ethical = localization
        .get(format!("alignment.{}", deity.ethical_alignment().to_lowername()), language);
    let moral =
        localization.get(format!("alignment.{}", deity.moral_alignment().to_lowername()), language);
    format!("{ethical} {moral}")
}

/// Formats monster kind description.
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
    mut heritage_desc_q: Query<
        (&mut Text, &LocalizedElfHeritageDesc),
        (
            Without<LocalizedText>,
            Without<LocalizedRaceDesc>,
            Without<LocalizedClassDesc>,
            Without<LocalizedAjahDesc>,
            Without<LocalizedPetDesc>,
            Without<LocalizedMonsterKindDesc>,
        ),
    >,
    mut specialization_desc_q: Query<
        (&mut Text, &LocalizedSpecializationDesc),
        (
            Without<LocalizedText>,
            Without<LocalizedRaceDesc>,
            Without<LocalizedClassDesc>,
            Without<LocalizedAjahDesc>,
            Without<LocalizedPetDesc>,
            Without<LocalizedMonsterKindDesc>,
            Without<LocalizedElfHeritageDesc>,
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

    for (mut text, desc) in &mut heritage_desc_q {
        text.0 = format_elf_heritage_description(desc.0, settings.language, &localization);
    }

    for (mut text, desc) in &mut specialization_desc_q {
        text.0 = format_specialization_description(desc.0, settings.language, &localization);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies alignment formatting omits deity names and handles true neutral naturally.
    #[test]
    fn deity_alignment_uses_only_the_localized_alignment() {
        let localization = Localization::from_world(&mut World::new());

        assert_eq!(
            format_deity_alignment(Deity::Kharos, Language::English, &localization),
            "Chaotic Evil"
        );
        assert_eq!(
            format_deity_alignment(Deity::Tharos, Language::English, &localization),
            "True Neutral"
        );
    }

    /// Verifies deity card prose does not repeat the separately displayed alignment title.
    #[test]
    fn deity_description_starts_with_lore() {
        let localization = Localization::from_world(&mut World::new());
        let description = format_deity_description(Deity::Kharos, Language::English, &localization);

        assert!(description.starts_with("Exalts ruin, fury, and conquest."));
        assert!(!description.contains("Chaotic Evil"));
    }

    /// Verifies creator pool bonuses use max-stat wording without parenthesized qualifiers.
    #[test]
    fn maximum_pool_bonuses_use_max_stat_labels() {
        let localization = Localization::from_world(&mut World::new());

        assert_eq!(
            maximum_bonus(15, MaximumPool::Health, Language::English, &localization),
            "+15 max health"
        );
        assert_eq!(
            maximum_bonus(5, MaximumPool::Mana, Language::English, &localization),
            "+5 max mana"
        );
    }

    /// Verifies Elf heritage cards are rendered from their shared gameplay profiles.
    #[test]
    fn elf_heritage_descriptions_show_benefits_and_drawbacks() {
        let localization = Localization::from_world(&mut World::new());
        let high =
            format_elf_heritage_description(ElfHeritage::High, Language::English, &localization);
        let dark =
            format_elf_heritage_description(ElfHeritage::Dark, Language::English, &localization);
        let wood =
            format_elf_heritage_description(ElfHeritage::Wood, Language::English, &localization);

        assert!(high.contains("+1 Intelligence"));
        assert!(!high.contains("Strength"));
        assert!(dark.contains("+8% Crit. chance"));
        assert!(dark.contains("-10 max mana"));
        assert!(wood.contains("+1 Attack (Range)"));
        assert!(wood.contains("-1 Attack (Melee)"));
    }
}
