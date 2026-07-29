//! Ability catalog records, targeting requirements, and localized presentation.

use crate::core::catalog::effects::Effect;
use crate::core::catalog::equipment::Kind;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use crate::utils::NameFromEnum;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityRole {
    Primer,
    Payoff,
    Reaction,
    Sustain,
}

impl AbilityRole {
    /// Returns the localization key for this tactical role.
    pub const fn localization_key(self) -> &'static str {
        match self {
            Self::Primer => "combat.role_primer",
            Self::Payoff => "combat.role_payoff",
            Self::Reaction => "combat.role_reaction",
            Self::Sustain => "combat.role_sustain",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ability {
    /// Name of the ability (matches the english name)
    /// Lowercase with space -> underscore matches the language key for name
    pub name: String,

    /// Name of the image the ability corresponds to
    pub image: String,

    /// Level of the ability
    pub level: u32,

    /// Kind of ability
    pub kind: Kind,

    /// How much mana this ability costs
    pub mana_cost: u32,

    /// The ability cooldown (in seconds)
    pub cooldown: f32,

    /// Whether the ability applies to self or the enemy
    pub on_self: bool,

    /// Whether this ability applies only to the player or also his pet
    pub is_aoe: bool,

    /// Effects applied when hitting
    pub effects: Vec<Effect>,
}

impl Ability {
    /// Classifies this ability by how it participates in combat decisions.
    pub fn role(&self) -> AbilityRole {
        let has_reaction = self.effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::Heal { .. }
                    | Effect::Purge
                    | Effect::Fortify { .. }
                    | Effect::MonarchShield { .. }
                    | Effect::Stun { .. }
                    | Effect::Silence { .. }
                    | Effect::Taunt { .. }
            )
        });
        if has_reaction {
            return AbilityRole::Reaction;
        }
        let has_payoff = self.effects.iter().any(|effect| {
            matches!(effect, Effect::Pierce { .. } | Effect::Cleave { .. } | Effect::Bleed { .. })
        });
        if has_payoff {
            return AbilityRole::Payoff;
        }
        let has_primer = self.effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::Blind { .. }
                    | Effect::Burn { .. }
                    | Effect::Curse { .. }
                    | Effect::Freeze { .. }
                    | Effect::Immobilize { .. }
                    | Effect::Paranoia { .. }
                    | Effect::Poison { .. }
                    | Effect::Vulnerability { .. }
            )
        });
        if has_primer {
            AbilityRole::Primer
        } else {
            AbilityRole::Sustain
        }
    }

    /// Performs the description operation.
    pub fn description(&self, _language: Language, _localization: &Localization) -> String {
        let mut parts = vec![
            format!("[level]{}", self.level),
            format!("[mana]{}", self.mana_cost),
            format!("[cooldown]{:.1}s", self.cooldown),
            format!("[{}]", self.kind.to_lowername()),
        ];
        if !self.effects.is_empty() {
            parts.push(format!("[ability]{}", self.effects.len()));
        }
        parts.join(" ")
    }

    /// Performs the full description operation.
    pub fn full_description(&self, language: Language, localization: &Localization) -> Vec<String> {
        let mut lines = Vec::new();
        let level_label = localization.get("general.level", language);
        let mana_label = localization.get("general.mana", language);
        let cooldown_label = localization.get("general.cooldown", language);
        let kind_label = localization.get("general.kind", language);
        let role_label = localization.get("combat.role", language);
        let target_label = localization.get("general.target", language);
        let aoe_label = localization.get("general.aoe", language);
        let abilities_label = localization.get("general.abilities", language);

        let kind_name = localization.get(format!("general.{}", self.kind.to_lowername()), language);
        let mut target_val = if self.on_self {
            localization.get("general.self", language)
        } else {
            localization.get("general.enemy", language)
        };
        if let Some(first_char) = target_val.chars().next() {
            let capitalized =
                first_char.to_uppercase().to_string() + &target_val[first_char.len_utf8()..];
            target_val = capitalized;
        }
        let aoe_val = if self.is_aoe {
            localization.get("general.yes", language)
        } else {
            localization.get("general.no", language)
        };

        lines.push(format!("[level] {}: {}", level_label, self.level));
        lines.push(format!("[mana] {}: {}", mana_label, self.mana_cost));
        lines.push(format!("[cooldown] {}: {:.1}s", cooldown_label, self.cooldown));
        lines.push(format!("[{}] {}: {}", self.kind.to_lowername(), kind_label, kind_name));
        lines.push(format!(
            "[ability] {}: {}",
            role_label,
            localization.get(self.role().localization_key(), language)
        ));
        lines.push(format!("[target] {}: {}", target_label, target_val));
        lines.push(format!("[aoe] {}: {}", aoe_label, aoe_val));
        lines.push(format!("[ability] {}:", abilities_label));
        for e in &self.effects {
            lines.push(format!("• {}", e.description(language, localization)));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::{Ability, AbilityRole};
    use crate::core::catalog::effects::Effect;
    use crate::core::catalog::equipment::Kind;

    /// Builds a minimal ability around the supplied effects.
    fn ability_with(effects: Vec<Effect>) -> Ability {
        Ability {
            name: "Test Ability".to_string(),
            image: "test".to_string(),
            level: 1,
            kind: Kind::Physical,
            mana_cost: 1,
            cooldown: 2.0,
            on_self: false,
            is_aoe: false,
            effects,
        }
    }

    #[test]
    /// Verifies tactical roles prioritize reactions before damage classifications.
    fn reactions_take_role_precedence() {
        let ability = ability_with(vec![
            Effect::Pierce {
                damage: 5,
            },
            Effect::Stun {
                duration: 2.0,
            },
        ]);
        assert_eq!(ability.role(), AbilityRole::Reaction);
    }

    #[test]
    /// Verifies primers and payoffs retain distinct tactical classifications.
    fn primers_and_payoffs_have_distinct_roles() {
        let primer = ability_with(vec![Effect::Freeze {
            attack_speed_pct: -20.0,
            duration: 3.0,
        }]);
        let payoff = ability_with(vec![Effect::Pierce {
            damage: 8,
        }]);
        assert_eq!(primer.role(), AbilityRole::Primer);
        assert_eq!(payoff.role(), AbilityRole::Payoff);
    }
}
