use crate::core::build::effects::Effect;
use crate::core::build::equipment::Kind;
use crate::core::build::modifiers::Modifier;
use crate::core::localization::Localization;
use crate::core::settings::Language;
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
    pub fn description(&self, language: Language, localization: &Localization) -> String {
        let mut parts =
            vec![format!("+{} Atk", self.attack), format!("{:.1} AS", self.attack_speed)];
        if self.crit_chance > 0.0 {
            parts.push(format!("{:.0}% Crit", self.crit_chance * 100.0));
        }
        for m in &self.modifiers {
            parts.push(m.description(language, localization));
        }
        for e in &self.effects {
            parts.push(e.description(language, localization));
        }
        parts.join(" | ")
    }
}
