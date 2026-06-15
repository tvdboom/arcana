use crate::core::catalog::equipment::Kind;
use crate::core::catalog::weapons::Category;
use crate::core::localization::Localization;
use crate::core::player::Attribute;
use crate::core::settings::Language;
use crate::utils::NameFromEnum;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Modifier {
    /// Attributes
    AttributeModifier(Attribute, i32),

    /// Combat stats
    AttackModifier(i32),
    DefenseModifier(i32),
    InitiativeModifier(i32),
    AttackSpeedModifier(f32),
    CritChanceModifier(f32),

    /// Pets
    PetAttackModifier(i32),
    PetDefenseModifier(i32),
    PetInitiativeModifier(i32),
    PetAttackSpeedModifier(i32),

    /// Health & Mana
    MaxHealthModifier(i32),
    MaxManaModifier(i32),
    HealthRegen(i32),
    ManaRegen(i32),

    /// Ability and weapon effects
    KindPowerMultiplier(Kind, f32),
    KindResistanceMultiplier(Kind, f32),
    CategoryPowerMultiplier(Category, f32),
    CategoryResistanceMultiplier(Category, f32),

    /// Others
    LifeSteal(f32),
    HealingMultiplier(f32),
}

impl Modifier {
    pub fn description(&self, language: Language, localization: &Localization) -> String {
        match self {
            Self::AttributeModifier(attr, amount) => {
                let attr = localization.get(attr.to_lowername(), language).to_lowercase();
                format!("{amount:+} {attr}")
            },
            Self::AttackModifier(amount) => {
                let attack = localization.get("attack", language).to_lowercase();
                format!("{amount:+} {attack}")
            },
            Self::DefenseModifier(amount) => {
                let defense = localization.get("defense", language).to_lowercase();
                format!("{amount:+} {defense}")
            },
            Self::InitiativeModifier(amount) => {
                let initiative = localization.get("initiative", language).to_lowercase();
                format!("{amount:+} {initiative}")
            },
            Self::AttackSpeedModifier(percentage) => {
                let attack_speed = localization.get("attack_speed", language).to_lowercase();
                format!("{percentage:+.0}% {attack_speed}")
            },
            Self::CritChanceModifier(percentage) => {
                let crit_chance = localization.get("crit_chance", language).to_lowercase();
                format!("{percentage:+.0}% {crit_chance}")
            },
            Self::PetAttackModifier(amount) => {
                let pet_attack = localization.get("pet_attack", language).to_lowercase();
                format!("{amount:+} {pet_attack}")
            },
            Self::PetDefenseModifier(amount) => {
                let pet_defense = localization.get("pet_defense", language).to_lowercase();
                format!("{amount:+} {pet_defense}")
            },
            Self::PetInitiativeModifier(amount) => {
                let pet_initiative = localization.get("pet_initiative", language).to_lowercase();
                format!("{amount:+} {pet_initiative}")
            },
            Self::PetAttackSpeedModifier(amount) => {
                let pet_attack_speed =
                    localization.get("pet_attack_speed", language).to_lowercase();
                format!("{amount:+} {pet_attack_speed}")
            },
            Self::MaxHealthModifier(amount) => {
                let max_health = localization.get("general.max_health", language).to_lowercase();
                format!("{amount:+} {max_health}")
            },
            Self::MaxManaModifier(amount) => {
                let max_mana = localization.get("general.max_mana", language).to_lowercase();
                format!("{amount:+} {max_mana}")
            },
            Self::HealthRegen(amount) => {
                let health_regen =
                    localization.get("general.health_regen", language).to_lowercase();
                format!("{amount:+} {health_regen}")
            },
            Self::ManaRegen(amount) => {
                let mana_regen = localization.get("general.mana_regen", language).to_lowercase();
                format!("{amount:+} {mana_regen}")
            },
            Self::KindPowerMultiplier(kind, percentage) => {
                let kind_str = localization
                    .get(format!("general.{}", kind.to_lowername()), language)
                    .to_lowercase();
                let damage_str = localization.get("general.damage", language).to_lowercase();
                format!("{percentage:+.0}% {kind_str} {damage_str}")
            },
            Self::KindResistanceMultiplier(kind, percentage) => {
                let kind_str = localization
                    .get(format!("general.{}", kind.to_lowername()), language)
                    .to_lowercase();
                let resist_str = localization.get("general.resistance", language).to_lowercase();
                format!("{percentage:+.0}% {kind_str} {resist_str}")
            },
            Self::CategoryPowerMultiplier(category, percentage) => {
                let category_str = localization
                    .get(format!("general.{}", category.to_lowername()), language)
                    .to_lowercase();
                let damage_str = localization.get("general.damage", language).to_lowercase();
                format!("{percentage:+.0}% {category_str} {damage_str}")
            },
            Self::CategoryResistanceMultiplier(category, percentage) => {
                let category_str = localization
                    .get(format!("general.{}", category.to_lowername()), language)
                    .to_lowercase();
                let resist_str = localization.get("general.resistance", language).to_lowercase();
                format!("{percentage:+.0}% {category_str} {resist_str}")
            },
            Self::LifeSteal(percentage) => {
                let lifesteal_str = localization.get("general.life_steal", language).to_lowercase();
                format!("{percentage:+.0}% {lifesteal_str}")
            },
            Self::HealingMultiplier(percentage) => {
                let healing_str =
                    localization.get("general.healing_multiplier", language).to_lowercase();
                format!("{percentage:+.0}% {healing_str}")
            },
        }
    }
}
