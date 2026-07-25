//! Shared numerical bonuses granted by character-creation identity choices.

use std::ops::{Add, AddAssign};

/// Numerical combat bonuses supplied by a race, class, specialization, or deity.
///
/// Critical chance and attack speed are stored as fractions (`0.05` means five percent).
/// Category-specific attack fields are flat bonuses that apply only while a matching weapon is
/// equipped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IdentityBonuses {
    pub attack: i32,
    pub defense: i32,
    pub initiative: i32,
    pub max_health: i32,
    pub max_mana: i32,
    pub health_regen: i32,
    pub mana_regen: i32,
    pub crit_chance: f32,
    pub attack_speed: f32,
    pub melee_attack: i32,
    pub finesse_attack: i32,
    pub ranged_attack: i32,
}

impl Add for IdentityBonuses {
    type Output = Self;

    /// Combines two independent identity bonus packages.
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for IdentityBonuses {
    /// Adds every numerical field so identity sources can be composed safely.
    fn add_assign(&mut self, rhs: Self) {
        self.attack += rhs.attack;
        self.defense += rhs.defense;
        self.initiative += rhs.initiative;
        self.max_health += rhs.max_health;
        self.max_mana += rhs.max_mana;
        self.health_regen += rhs.health_regen;
        self.mana_regen += rhs.mana_regen;
        self.crit_chance += rhs.crit_chance;
        self.attack_speed += rhs.attack_speed;
        self.melee_attack += rhs.melee_attack;
        self.finesse_attack += rhs.finesse_attack;
        self.ranged_attack += rhs.ranged_attack;
    }
}

#[cfg(test)]
impl IdentityBonuses {
    /// Estimates a package using Arcana's early-game damage, dodge, and regeneration formulas.
    pub fn representative_combat_rating(self) -> f32 {
        let attack = 10.0
            + self.attack as f32
            + self.melee_attack.max(self.finesse_attack).max(self.ranged_attack).max(0) as f32;
        let defense = 7.0 + self.defense as f32;
        let initiative = 8.0 + self.initiative as f32;
        let max_health = 100.0 + self.max_health as f32;
        let max_mana = 100.0 + self.max_mana as f32;
        let health_regen = 2.0 + self.health_regen as f32;
        let mana_regen = 2.0 + self.mana_regen as f32;
        let attack_speed = 1.0 + self.attack_speed;
        let crit_chance = 0.05 + self.crit_chance;
        let dodge =
            |attacker: f32, defender: f32| (0.18 + (defender - attacker) * 0.018).clamp(0.08, 0.70);

        let damage_per_hit = attack * attack / (attack + 7.0);
        let damage_per_second = attack_speed / 2.0
            * (1.0 - dodge(initiative, 8.0))
            * damage_per_hit
            * (1.0 + crit_chance);
        let incoming_per_second =
            0.5 * (1.0 - dodge(8.0, initiative)) * 100.0 / (10.0 + defense) - 0.3 * health_regen;
        let survival = max_health / incoming_per_second.max(0.1);
        let mana_factor = ((max_mana + 12.0 * mana_regen) / 124.0).powf(0.15);
        damage_per_second * survival.sqrt() * mana_factor
    }
}
