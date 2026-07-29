//! Tactical combat roles, player stances, and telegraphed enemy move definitions.

use crate::core::catalog::abilities::Ability;
use crate::core::catalog::effects::Effect;
use crate::core::monsters::MonsterArchetype;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CombatStance {
    #[default]
    Aggressive,
    Defensive,
    Precise,
    Disruptive,
}

impl CombatStance {
    pub const ALL: [Self; 4] = [Self::Aggressive, Self::Defensive, Self::Precise, Self::Disruptive];

    /// Returns the localization key for this stance's visible name.
    pub const fn name_key(self) -> &'static str {
        match self {
            Self::Aggressive => "combat.stance_aggressive",
            Self::Defensive => "combat.stance_defensive",
            Self::Precise => "combat.stance_precise",
            Self::Disruptive => "combat.stance_disruptive",
        }
    }

    /// Returns the source asset key for this stance's combat card.
    pub const fn image_key(self) -> &'static str {
        match self {
            Self::Aggressive => "combat_stance_aggressive",
            Self::Defensive => "combat_stance_defensive",
            Self::Precise => "combat_stance_precise",
            Self::Disruptive => "combat_stance_disruptive",
        }
    }

    /// Returns the keyboard label used by this stance's combat card.
    pub const fn hotkey_label(self) -> &'static str {
        match self {
            Self::Aggressive => "1",
            Self::Defensive => "2",
            Self::Precise => "3",
            Self::Disruptive => "4",
        }
    }

    /// Returns the outgoing basic-attack damage multiplier.
    pub const fn attack_damage_multiplier(self) -> f32 {
        match self {
            Self::Aggressive => 1.20,
            Self::Defensive => 0.84,
            Self::Precise => 0.95,
            Self::Disruptive => 0.78,
        }
    }

    /// Returns the incoming damage multiplier while this stance is active.
    pub const fn incoming_damage_multiplier(self) -> f32 {
        match self {
            Self::Aggressive => 1.12,
            Self::Defensive => 0.84,
            Self::Precise | Self::Disruptive => 1.0,
        }
    }

    /// Returns the multiplier applied to basic-attack speed.
    pub const fn attack_speed_multiplier(self) -> f32 {
        match self {
            Self::Precise => 0.90,
            _ => 1.0,
        }
    }

    /// Returns the flat critical-strike chance added to basic attacks.
    pub const fn critical_chance_bonus(self) -> f32 {
        match self {
            Self::Precise => 0.18,
            _ => 0.0,
        }
    }

    /// Returns the Poise damage inflicted by one player basic attack.
    pub const fn poise_damage(self) -> f32 {
        match self {
            Self::Aggressive => 6.0,
            Self::Defensive => 4.0,
            Self::Precise => 7.0,
            Self::Disruptive => 16.0,
        }
    }

    /// Returns the perfect-parry window in seconds.
    pub const fn perfect_guard_window(self) -> f32 {
        match self {
            Self::Defensive => 0.34,
            _ => 0.24,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyMoveTarget {
    Player,
    Pet,
    SelfSide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyMoveKind {
    CrushingBlow,
    DarkRitual,
    VenomSpray,
    DefensiveShell,
    Execution,
    DevourCompanion,
    DrainLife,
    WitheringHex,
    GlacialBreath,
    SmokeVeil,
    SunderArmor,
    FlameBreath,
    EntanglingRoots,
    BloodFeast,
    ArcaneRupture,
    WarCry,
    Earthshatter,
    GraveChill,
    SoulSiphon,
    ShieldBash,
    IronBulwark,
    FanOfKnives,
    Shadowstep,
    CorrosiveBite,
    ParasiticBloom,
    Meteor,
    TemporalLock,
    SavagePounce,
    TerrifyingRoar,
}

impl EnemyMoveKind {
    /// Returns the monster level at which this move joins its archetype's rotation.
    pub const fn minimum_level(self) -> u32 {
        match self {
            Self::CrushingBlow
            | Self::VenomSpray
            | Self::DefensiveShell
            | Self::DrainLife
            | Self::WitheringHex
            | Self::GlacialBreath
            | Self::SmokeVeil
            | Self::SunderArmor
            | Self::EntanglingRoots
            | Self::BloodFeast
            | Self::ArcaneRupture
            | Self::WarCry
            | Self::SavagePounce => 1,
            Self::DarkRitual
            | Self::FlameBreath
            | Self::GraveChill
            | Self::ShieldBash
            | Self::FanOfKnives
            | Self::CorrosiveBite
            | Self::Meteor
            | Self::TerrifyingRoar => 5,
            Self::DevourCompanion
            | Self::Earthshatter
            | Self::SoulSiphon
            | Self::IronBulwark
            | Self::Shadowstep
            | Self::ParasiticBloom
            | Self::TemporalLock => 10,
            Self::Execution => 15,
        }
    }

    /// Returns the localization key for the move's name.
    pub const fn name_key(self) -> &'static str {
        match self {
            Self::CrushingBlow => "combat.move_crushing_blow",
            Self::DarkRitual => "combat.move_dark_ritual",
            Self::VenomSpray => "combat.move_venom_spray",
            Self::DefensiveShell => "combat.move_defensive_shell",
            Self::Execution => "combat.move_execution",
            Self::DevourCompanion => "combat.move_devour_companion",
            Self::DrainLife => "combat.move_drain_life",
            Self::WitheringHex => "combat.move_withering_hex",
            Self::GlacialBreath => "combat.move_glacial_breath",
            Self::SmokeVeil => "combat.move_smoke_veil",
            Self::SunderArmor => "combat.move_sunder_armor",
            Self::FlameBreath => "combat.move_flame_breath",
            Self::EntanglingRoots => "combat.move_entangling_roots",
            Self::BloodFeast => "combat.move_blood_feast",
            Self::ArcaneRupture => "combat.move_arcane_rupture",
            Self::WarCry => "combat.move_war_cry",
            Self::Earthshatter => "combat.move_earthshatter",
            Self::GraveChill => "combat.move_grave_chill",
            Self::SoulSiphon => "combat.move_soul_siphon",
            Self::ShieldBash => "combat.move_shield_bash",
            Self::IronBulwark => "combat.move_iron_bulwark",
            Self::FanOfKnives => "combat.move_fan_of_knives",
            Self::Shadowstep => "combat.move_shadowstep",
            Self::CorrosiveBite => "combat.move_corrosive_bite",
            Self::ParasiticBloom => "combat.move_parasitic_bloom",
            Self::Meteor => "combat.move_meteor",
            Self::TemporalLock => "combat.move_temporal_lock",
            Self::SavagePounce => "combat.move_savage_pounce",
            Self::TerrifyingRoar => "combat.move_terrifying_roar",
        }
    }

    /// Returns the localization key for the move's short tactical description.
    pub const fn description_key(self) -> &'static str {
        match self {
            Self::CrushingBlow => "combat.move_crushing_blow_desc",
            Self::DarkRitual => "combat.move_dark_ritual_desc",
            Self::VenomSpray => "combat.move_venom_spray_desc",
            Self::DefensiveShell => "combat.move_defensive_shell_desc",
            Self::Execution => "combat.move_execution_desc",
            Self::DevourCompanion => "combat.move_devour_companion_desc",
            Self::DrainLife => "combat.move_drain_life_desc",
            Self::WitheringHex => "combat.move_withering_hex_desc",
            Self::GlacialBreath => "combat.move_glacial_breath_desc",
            Self::SmokeVeil => "combat.move_smoke_veil_desc",
            Self::SunderArmor => "combat.move_sunder_armor_desc",
            Self::FlameBreath => "combat.move_flame_breath_desc",
            Self::EntanglingRoots => "combat.move_entangling_roots_desc",
            Self::BloodFeast => "combat.move_blood_feast_desc",
            Self::ArcaneRupture => "combat.move_arcane_rupture_desc",
            Self::WarCry => "combat.move_war_cry_desc",
            Self::Earthshatter => "combat.move_earthshatter_desc",
            Self::GraveChill => "combat.move_grave_chill_desc",
            Self::SoulSiphon => "combat.move_soul_siphon_desc",
            Self::ShieldBash => "combat.move_shield_bash_desc",
            Self::IronBulwark => "combat.move_iron_bulwark_desc",
            Self::FanOfKnives => "combat.move_fan_of_knives_desc",
            Self::Shadowstep => "combat.move_shadowstep_desc",
            Self::CorrosiveBite => "combat.move_corrosive_bite_desc",
            Self::ParasiticBloom => "combat.move_parasitic_bloom_desc",
            Self::Meteor => "combat.move_meteor_desc",
            Self::TemporalLock => "combat.move_temporal_lock_desc",
            Self::SavagePounce => "combat.move_savage_pounce_desc",
            Self::TerrifyingRoar => "combat.move_terrifying_roar_desc",
        }
    }

    /// Returns an existing ability-art key used to visualize this enemy intent.
    pub const fn image_key(self) -> &'static str {
        match self {
            Self::CrushingBlow => "images/catalog/abilities/skill_200_scullbreaker.webp",
            Self::DarkRitual => "images/catalog/abilities/DemonicFate.webp",
            Self::VenomSpray => "images/catalog/abilities/poison_claw.webp",
            Self::DefensiveShell => "images/catalog/abilities/Skill_DefStance.webp",
            Self::Execution => "images/catalog/abilities/Death_blow.webp",
            Self::DevourCompanion => "images/catalog/abilities/skill_143_devour.webp",
            Self::DrainLife => "images/catalog/abilities/drainlife.webp",
            Self::WitheringHex => "images/catalog/abilities/devil_mark.webp",
            Self::GlacialBreath => "images/catalog/abilities/dragon_coldbreath.webp",
            Self::SmokeVeil => "images/catalog/abilities/blackwater.webp",
            Self::SunderArmor => "images/catalog/abilities/skill_54_brakearmor.webp",
            Self::FlameBreath => "images/catalog/abilities/dragon_firebreath.webp",
            Self::EntanglingRoots => "images/catalog/abilities/skill_133_root.webp",
            Self::BloodFeast => "images/catalog/abilities/Bloodlust.webp",
            Self::ArcaneRupture => "images/catalog/abilities/skill_116_arcaneBlast.webp",
            Self::WarCry => "images/catalog/abilities/skill_177_rage.webp",
            Self::Earthshatter => "images/catalog/abilities/Skill_223_earthspirit.webp",
            Self::GraveChill => "images/catalog/abilities/summon_death.webp",
            Self::SoulSiphon => "images/catalog/abilities/soul_devouring.webp",
            Self::ShieldBash => "images/catalog/abilities/shield4.webp",
            Self::IronBulwark => "images/catalog/abilities/skill_245_holyshield.webp",
            Self::FanOfKnives => "images/catalog/abilities/skill_11_blade.webp",
            Self::Shadowstep => "images/catalog/abilities/Y_shadowstun.webp",
            Self::CorrosiveBite => "images/catalog/abilities/fishbite.webp",
            Self::ParasiticBloom => "images/catalog/abilities/skill_148_naturewarrior.webp",
            Self::Meteor => "images/catalog/abilities/skill_5_meteor.webp",
            Self::TemporalLock => "images/catalog/abilities/skill_115_timestop.webp",
            Self::SavagePounce => "images/catalog/abilities/Y_WarriorRuthlessness.webp",
            Self::TerrifyingRoar => "images/catalog/abilities/Fear.webp",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyMove {
    pub kind: EnemyMoveKind,
    pub cast_time: f32,
    pub recovery: f32,
    pub target: EnemyMoveTarget,
}

/// Builds the deterministic telegraphed move rotation for one enemy archetype.
pub fn enemy_move_rotation(archetype: MonsterArchetype, level: u32) -> Vec<EnemyMove> {
    let cast_scale = (1.0 - level.saturating_sub(1) as f32 * 0.008).max(0.82);
    let move_of = |kind, cast_time, recovery, target| EnemyMove {
        kind,
        cast_time: cast_time * cast_scale,
        recovery,
        target,
    };
    let mut rotation = match archetype {
        MonsterArchetype::Berserker => vec![
            move_of(EnemyMoveKind::CrushingBlow, 2.2, 3.5, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::SunderArmor, 2.4, 3.8, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::WarCry, 2.0, 3.5, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::Earthshatter, 2.8, 4.5, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::DarkRitual, 2.6, 4.0, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::Execution, 2.8, 5.0, EnemyMoveTarget::Player),
        ],
        MonsterArchetype::Necromancer => vec![
            move_of(EnemyMoveKind::WitheringHex, 2.4, 3.8, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::DarkRitual, 3.0, 4.5, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::DrainLife, 2.5, 4.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::BloodFeast, 2.8, 4.6, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::GraveChill, 2.6, 4.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::SoulSiphon, 3.0, 4.8, EnemyMoveTarget::Player),
        ],
        MonsterArchetype::Knight => vec![
            move_of(EnemyMoveKind::DefensiveShell, 2.0, 4.0, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::CrushingBlow, 2.7, 3.5, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::SunderArmor, 2.5, 4.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::ShieldBash, 2.1, 3.5, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::IronBulwark, 2.8, 5.0, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::Execution, 3.0, 5.0, EnemyMoveTarget::Player),
        ],
        MonsterArchetype::Assassin => vec![
            move_of(EnemyMoveKind::SmokeVeil, 1.8, 3.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::VenomSpray, 2.1, 3.8, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::EntanglingRoots, 2.0, 3.6, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::FanOfKnives, 2.0, 3.5, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::Shadowstep, 2.3, 4.2, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::Execution, 2.2, 4.5, EnemyMoveTarget::Player),
        ],
        MonsterArchetype::Leech => vec![
            move_of(EnemyMoveKind::DrainLife, 2.2, 3.2, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::VenomSpray, 2.4, 4.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::DarkRitual, 2.8, 4.5, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::BloodFeast, 2.6, 4.3, EnemyMoveTarget::SelfSide),
            move_of(EnemyMoveKind::CorrosiveBite, 2.2, 3.8, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::ParasiticBloom, 2.8, 4.6, EnemyMoveTarget::SelfSide),
        ],
        MonsterArchetype::Mage => vec![
            move_of(EnemyMoveKind::GlacialBreath, 2.8, 4.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::WitheringHex, 2.3, 3.5, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::ArcaneRupture, 2.5, 4.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::Meteor, 3.0, 4.8, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::TemporalLock, 2.8, 4.5, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::DarkRitual, 3.0, 4.5, EnemyMoveTarget::SelfSide),
        ],
        MonsterArchetype::Beast => vec![
            move_of(EnemyMoveKind::CrushingBlow, 2.0, 3.2, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::DevourCompanion, 2.7, 4.5, EnemyMoveTarget::Pet),
            move_of(EnemyMoveKind::FlameBreath, 2.8, 4.2, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::VenomSpray, 2.3, 4.0, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::SavagePounce, 2.1, 3.6, EnemyMoveTarget::Player),
            move_of(EnemyMoveKind::TerrifyingRoar, 2.5, 4.2, EnemyMoveTarget::Player),
        ],
    };
    rotation.retain(|movement| level >= movement.kind.minimum_level());
    rotation
}

/// Returns the Poise damage contributed by one ability cast.
pub fn ability_poise_damage(ability: &Ability) -> f32 {
    let mut damage: f32 = 0.0;
    for effect in &ability.effects {
        damage = damage.max(match effect {
            Effect::Stun {
                ..
            }
            | Effect::Silence {
                ..
            } => 34.0,
            Effect::Immobilize {
                ..
            }
            | Effect::Blind {
                ..
            }
            | Effect::Freeze {
                ..
            } => 18.0,
            Effect::Pierce {
                ..
            }
            | Effect::Cleave {
                ..
            }
            | Effect::Curse {
                ..
            } => 13.0,
            Effect::Burn {
                ..
            }
            | Effect::Poison {
                ..
            }
            | Effect::Vulnerability {
                ..
            } => 9.0,
            _ => 5.0,
        });
    }
    damage
}

#[cfg(test)]
mod tests {
    use super::{enemy_move_rotation, CombatStance, EnemyMoveKind};
    use crate::core::monsters::MonsterArchetype;

    #[test]
    /// Verifies every generated monster archetype has a varied telegraphed rotation.
    fn every_archetype_has_a_varied_rotation() {
        let archetypes = [
            MonsterArchetype::Berserker,
            MonsterArchetype::Necromancer,
            MonsterArchetype::Knight,
            MonsterArchetype::Assassin,
            MonsterArchetype::Leech,
            MonsterArchetype::Mage,
            MonsterArchetype::Beast,
        ];
        for archetype in archetypes {
            let rotation = enemy_move_rotation(archetype, 10);
            assert!(rotation.len() >= 4);
            assert!(rotation.iter().all(|movement| movement.cast_time > 1.0));
            assert!(rotation.windows(2).all(|moves| moves[0].kind != moves[1].kind));
        }
    }

    #[test]
    /// Verifies higher levels shorten telegraphs without making them unreadable.
    fn higher_level_telegraphs_remain_reactable() {
        let low = enemy_move_rotation(MonsterArchetype::Mage, 1);
        let high = enemy_move_rotation(MonsterArchetype::Mage, 20);
        for (low_move, high_move) in low.iter().zip(&high) {
            assert!(high_move.cast_time < low_move.cast_time);
            assert!(high_move.cast_time >= low_move.cast_time * 0.82);
        }
    }

    #[test]
    /// Verifies each stance owns a meaningful tactical advantage and tradeoff.
    fn stances_have_distinct_combat_jobs() {
        assert!(CombatStance::Aggressive.attack_damage_multiplier() > 1.0);
        assert!(CombatStance::Aggressive.incoming_damage_multiplier() > 1.0);
        assert!(CombatStance::Defensive.incoming_damage_multiplier() < 1.0);
        assert!(CombatStance::Precise.critical_chance_bonus() > 0.0);
        assert!(CombatStance::Disruptive.poise_damage() > CombatStance::Aggressive.poise_damage());
    }

    #[test]
    /// Verifies the most dangerous moves remain present in their intended archetypes.
    fn signature_moves_are_not_lost() {
        let berserker = enemy_move_rotation(MonsterArchetype::Berserker, 20);
        let beast = enemy_move_rotation(MonsterArchetype::Beast, 20);
        assert!(berserker.iter().any(|movement| movement.kind == EnemyMoveKind::Execution));
        assert!(beast.iter().any(|movement| movement.kind == EnemyMoveKind::DevourCompanion));
    }

    #[test]
    /// Verifies higher-level monsters unlock larger rotations containing gated moves.
    fn higher_levels_unlock_more_dangerous_moves() {
        let archetypes = [
            MonsterArchetype::Berserker,
            MonsterArchetype::Necromancer,
            MonsterArchetype::Knight,
            MonsterArchetype::Assassin,
            MonsterArchetype::Leech,
            MonsterArchetype::Mage,
            MonsterArchetype::Beast,
        ];
        for archetype in archetypes {
            let novice = enemy_move_rotation(archetype, 1);
            let veteran = enemy_move_rotation(archetype, 10);
            let apex = enemy_move_rotation(archetype, 20);

            assert!(veteran.len() > novice.len());
            assert!(apex.len() >= veteran.len());
            assert!(novice.iter().all(|movement| movement.kind.minimum_level() == 1));
            assert!(apex.iter().any(|movement| movement.kind.minimum_level() >= 10));
        }
    }
}
