use crate::core::build::effects::Effect;
use crate::core::build::equipment::Kind;
use crate::core::build::modifiers::Modifier;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use crate::utils::NameFromEnum;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Category {
    Finesse,
    Magical,
    Melee,
    Range,
    Shield,
    Book,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Hand {
    OneHand,
    TwoHand,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Weapon {
    /// Name of the weapon (matches the English name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the weapon corresponds to
    pub image: String,

    /// Level or upgrade tier of the weapon
    pub level: u32,

    /// Kind of weapon
    pub kind: Kind,

    /// Weapon kind
    pub category: Category,

    /// Whether the weapon is carried in one or two hands
    pub hand: Hand,

    /// Gold value for buying and selling
    pub price: u32,

    /// Flat raw attack rating added to offensive calculations
    pub attack: u32,

    /// Attack interval pacing (e.g., 1.5 base weapon swings per second)
    pub attack_speed: f32,

    /// Base critical strike chance (e.g., 0.05 for a +5% chance)
    pub crit_chance: f32,

    /// Static attribute modifiers applied directly to the player while equipped
    pub modifiers: Vec<Modifier>,

    /// Effects triggered on landing a successful hit
    pub effects: Vec<Effect>,
}

impl Weapon {
    pub fn description(&self, _language: Language, _localization: &Localization) -> String {
        let mut parts = vec![format!("[level]{}", self.level)];
        if self.attack > 0 {
            parts.push(format!("[attack]{}", self.attack));
        }
        if self.attack_speed > 0.0 {
            parts.push(format!("[attack_speed]{:.1}", self.attack_speed));
        }
        if self.crit_chance > 0.0 {
            parts.push(format!("[crit_chance]{:.0}%", self.crit_chance * 100.0));
        }
        parts.push(format!("[{}]", self.kind.to_lowername()));
        parts.push(format!("[{}]", self.category.to_lowername()));
        if !self.modifiers.is_empty() {
            parts.push(format!("[modifier]{}", self.modifiers.len()));
        }
        if !self.effects.is_empty() {
            parts.push(format!("[ability]{}", self.effects.len()));
        }
        parts.join(" ")
    }

    pub fn full_description(&self, language: Language, localization: &Localization) -> Vec<String> {
        let mut lines = Vec::new();
        let level_label = localization.get("general.level", language);
        let attack_label = localization.get("general.attack", language);
        let attack_speed_label = localization.get("general.attack_speed", language);
        let crit_chance_label = localization.get("general.crit_chance", language);
        let kind_label = localization.get("general.kind", language);
        let category_label = localization.get("general.category", language);
        let modifiers_label = localization.get("general.modifiers", language);
        let abilities_label = localization.get("general.abilities", language);

        let kind_name = localization.get(format!("general.{}", self.kind.to_lowername()), language);
        let category_name = localization.get(format!("general.{}", self.category.to_lowername()), language);

        lines.push(format!("[level] {}: {}", level_label, self.level));
        if self.attack > 0 {
            lines.push(format!("[attack] {}: {}", attack_label, self.attack));
        }
        if self.attack_speed > 0.0 {
            lines.push(format!("[attack_speed] {}: {:.1}", attack_speed_label, self.attack_speed));
        }
        if self.crit_chance > 0.0 {
            lines.push(format!("[crit_chance] {}: {:.0}%", crit_chance_label, self.crit_chance * 100.0));
        }
        lines.push(format!("[{}] {}: {}", self.kind.to_lowername(), kind_label, kind_name));
        lines.push(format!("[{}] {}: {}", self.category.to_lowername(), category_label, category_name));

        if !self.modifiers.is_empty() {
            lines.push(format!("[modifier] {}:", modifiers_label));
            for m in &self.modifiers {
                lines.push(format!("• {}", m.description(language, localization)));
            }
        }
        if !self.effects.is_empty() {
            lines.push(format!("[ability] {}:", abilities_label));
            for e in &self.effects {
                lines.push(format!("• {}", e.description(language, localization)));
            }
        }
        lines
    }
}
