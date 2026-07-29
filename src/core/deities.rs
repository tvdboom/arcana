//! The nine deities, arranged across moral and ethical alignments.

use crate::core::identity::IdentityBonuses;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Deity {
    Aeloria,
    Serapha,
    Aurion,
    Vaelis,
    #[default]
    Tharos,
    Oryn,
    Kharos,
    Nyxara,
    Vhal,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MoralAlignment {
    Good,
    #[default]
    Neutral,
    Evil,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EthicalAlignment {
    Chaotic,
    #[default]
    Neutral,
    Lawful,
}

impl Deity {
    /// Returns the deity belonging to a moral and ethical alignment pair.
    pub fn from_alignment(moral: MoralAlignment, ethical: EthicalAlignment) -> Self {
        match (moral, ethical) {
            (MoralAlignment::Good, EthicalAlignment::Chaotic) => Deity::Aeloria,
            (MoralAlignment::Good, EthicalAlignment::Neutral) => Deity::Serapha,
            (MoralAlignment::Good, EthicalAlignment::Lawful) => Deity::Aurion,
            (MoralAlignment::Neutral, EthicalAlignment::Chaotic) => Deity::Vaelis,
            (MoralAlignment::Neutral, EthicalAlignment::Neutral) => Deity::Tharos,
            (MoralAlignment::Neutral, EthicalAlignment::Lawful) => Deity::Oryn,
            (MoralAlignment::Evil, EthicalAlignment::Chaotic) => Deity::Kharos,
            (MoralAlignment::Evil, EthicalAlignment::Neutral) => Deity::Nyxara,
            (MoralAlignment::Evil, EthicalAlignment::Lawful) => Deity::Vhal,
        }
    }

    /// Returns this deity's broad moral alignment.
    pub fn moral_alignment(self) -> MoralAlignment {
        match self {
            Deity::Aeloria | Deity::Serapha | Deity::Aurion => MoralAlignment::Good,
            Deity::Vaelis | Deity::Tharos | Deity::Oryn => MoralAlignment::Neutral,
            Deity::Kharos | Deity::Nyxara | Deity::Vhal => MoralAlignment::Evil,
        }
    }

    /// Returns this deity's relationship to law and freedom.
    pub fn ethical_alignment(self) -> EthicalAlignment {
        match self {
            Deity::Aeloria | Deity::Vaelis | Deity::Kharos => EthicalAlignment::Chaotic,
            Deity::Serapha | Deity::Tharos | Deity::Nyxara => EthicalAlignment::Neutral,
            Deity::Aurion | Deity::Oryn | Deity::Vhal => EthicalAlignment::Lawful,
        }
    }

    /// Returns the stable image key used by the UI asset map.
    pub fn image_key(self) -> &'static str {
        match self {
            Deity::Aeloria => "deity_aeloria",
            Deity::Serapha => "deity_serapha",
            Deity::Aurion => "deity_aurion",
            Deity::Vaelis => "deity_vaelis",
            Deity::Tharos => "deity_tharos",
            Deity::Oryn => "deity_oryn",
            Deity::Kharos => "deity_kharos",
            Deity::Nyxara => "deity_nyxara",
            Deity::Vhal => "deity_vhal",
        }
    }

    /// Returns the deity's permanent combat bonuses.
    pub fn bonuses(self) -> IdentityBonuses {
        match self {
            Deity::Aeloria => IdentityBonuses {
                crit_chance: 0.12,
                ..Default::default()
            },
            Deity::Serapha => IdentityBonuses {
                health_regen: 1,
                max_mana: 20,
                ..Default::default()
            },
            Deity::Aurion => IdentityBonuses {
                defense: 1,
                health_regen: 1,
                ..Default::default()
            },
            Deity::Vaelis => IdentityBonuses {
                initiative: 3,
                ..Default::default()
            },
            Deity::Tharos => IdentityBonuses {
                max_health: 20,
                max_mana: 15,
                ..Default::default()
            },
            Deity::Oryn => IdentityBonuses {
                defense: 2,
                max_health: 10,
                ..Default::default()
            },
            Deity::Kharos => IdentityBonuses {
                attack: 1,
                max_health: 5,
                ..Default::default()
            },
            Deity::Nyxara => IdentityBonuses {
                attack: 1,
                max_mana: 5,
                ..Default::default()
            },
            Deity::Vhal => IdentityBonuses {
                max_health: 25,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    /// Verifies that every alignment pair maps to one distinct deity.
    fn alignment_grid_contains_nine_unique_deities() {
        let mut deities = Vec::new();
        for moral in MoralAlignment::iter() {
            for ethical in EthicalAlignment::iter() {
                let deity = Deity::from_alignment(moral, ethical);
                assert_eq!(deity.moral_alignment(), moral);
                assert_eq!(deity.ethical_alignment(), ethical);
                deities.push(deity);
            }
        }
        deities.sort_by_key(|deity| *deity as u8);
        deities.dedup();

        assert_eq!(deities.len(), 9);
    }

    #[test]
    /// Verifies all divine packages fall within a narrow representative combat-value band.
    fn deity_bonuses_have_comparable_combat_value() {
        let ratings = Deity::iter()
            .map(|deity| deity.bonuses().representative_combat_rating())
            .collect::<Vec<_>>();
        let minimum = ratings.iter().copied().fold(f32::INFINITY, f32::min);
        let maximum = ratings.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        assert!(maximum / minimum <= 1.07, "deity ratings diverged: {ratings:?}");
    }

    /// Returns whether `left` is no worse in any field and better in at least one.
    fn strictly_dominates(left: IdentityBonuses, right: IdentityBonuses) -> bool {
        let no_worse = left.attack >= right.attack
            && left.defense >= right.defense
            && left.initiative >= right.initiative
            && left.max_health >= right.max_health
            && left.max_mana >= right.max_mana
            && left.health_regen >= right.health_regen
            && left.mana_regen >= right.mana_regen
            && left.crit_chance >= right.crit_chance
            && left.attack_speed >= right.attack_speed
            && left.melee_attack >= right.melee_attack
            && left.finesse_attack >= right.finesse_attack
            && left.ranged_attack >= right.ranged_attack;
        no_worse && left != right
    }

    #[test]
    /// Verifies every deity is a meaningful choice without penalties or strict domination.
    fn deity_bonuses_are_non_negative_and_not_dominated() {
        let deities = Deity::iter().collect::<Vec<_>>();
        for deity in &deities {
            let bonuses = deity.bonuses();
            assert!(
                bonuses.attack >= 0
                    && bonuses.defense >= 0
                    && bonuses.initiative >= 0
                    && bonuses.max_health >= 0
                    && bonuses.max_mana >= 0
                    && bonuses.health_regen >= 0
                    && bonuses.mana_regen >= 0
                    && bonuses.crit_chance >= 0.0
                    && bonuses.attack_speed >= 0.0,
                "{deity:?} has a punitive bonus package: {bonuses:?}"
            );
            for other in &deities {
                if deity != other {
                    assert!(
                        !strictly_dominates(other.bonuses(), bonuses),
                        "{deity:?} is strictly dominated by {other:?}"
                    );
                }
            }
        }
    }

    /// Counts the non-zero modifiers in a divine bonus package.
    fn active_modifier_count(bonuses: IdentityBonuses) -> usize {
        usize::from(bonuses.attack != 0)
            + usize::from(bonuses.defense != 0)
            + usize::from(bonuses.initiative != 0)
            + usize::from(bonuses.max_health != 0)
            + usize::from(bonuses.max_mana != 0)
            + usize::from(bonuses.health_regen != 0)
            + usize::from(bonuses.mana_regen != 0)
            + usize::from(bonuses.crit_chance != 0.0)
            + usize::from(bonuses.attack_speed != 0.0)
            + usize::from(bonuses.melee_attack != 0)
            + usize::from(bonuses.finesse_attack != 0)
            + usize::from(bonuses.ranged_attack != 0)
    }

    #[test]
    /// Verifies that every deity grants at most two numerical modifiers.
    fn deity_bonuses_have_at_most_two_modifiers() {
        for deity in Deity::iter() {
            assert!(
                active_modifier_count(deity.bonuses()) <= 2,
                "{deity:?} grants too many modifiers"
            );
        }
    }

    #[test]
    /// Verifies the Neutral Evil Veiled Queen grants the advertised attack and mana package.
    fn nyxara_grants_attack_and_maximum_mana() {
        let bonuses = Deity::Nyxara.bonuses();

        assert_eq!(bonuses.attack, 1);
        assert_eq!(bonuses.max_mana, 5);
    }
}
