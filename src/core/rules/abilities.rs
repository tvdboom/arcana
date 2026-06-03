use crate::core::player::{Ability, AbilityEffect};

pub struct AbilityDatabase;

impl AbilityDatabase {
    pub fn get_all_abilities() -> Vec<Ability> {
        vec![
            Ability {
                name: "Heavy Strike".to_string(),
                mana_cost: 0,
                cooldown_seconds: 4.0,
                description: "A mighty physical strike scaling with Strength.".to_string(),
                effect: AbilityEffect::Damage { base: 12, strength_scale: 1.5, intel_scale: 0.0 },
            },
            Ability {
                name: "Shield Block".to_string(),
                mana_cost: 5,
                cooldown_seconds: 8.0,
                description: "Raise shield creating a defensive barrier scaling with Strength.".to_string(),
                effect: AbilityEffect::Shield { base: 15, intel_scale: 0.0, duration_seconds: 4.0 },
            },
            Ability {
                name: "Swift Slash".to_string(),
                mana_cost: 0,
                cooldown_seconds: 2.0,
                description: "Fast strike with high base critical rating potential.".to_string(),
                effect: AbilityEffect::Damage { base: 8, strength_scale: 0.5, intel_scale: 0.0 },
            },
            Ability {
                name: "Poison Blade".to_string(),
                mana_cost: 10,
                cooldown_seconds: 6.0,
                description: "Coats weapon in poison, dealing damage over time.".to_string(),
                effect: AbilityEffect::Debuff { duration_seconds: 5.0, damage_per_tick: 4 },
            },
            Ability {
                name: "Fireball".to_string(),
                mana_cost: 15,
                cooldown_seconds: 5.0,
                description: "Shoot a blazing fireball scaling with Intelligence.".to_string(),
                effect: AbilityEffect::Damage { base: 15, strength_scale: 0.0, intel_scale: 2.0 },
            },
            Ability {
                name: "Mana Shield".to_string(),
                mana_cost: 10,
                cooldown_seconds: 10.0,
                description: "A defensive bubble absorbing damage scaling with Intelligence.".to_string(),
                effect: AbilityEffect::Shield { base: 20, intel_scale: 1.5, duration_seconds: 5.0 },
            },
            Ability {
                name: "Heal".to_string(),
                mana_cost: 12,
                cooldown_seconds: 6.0,
                description: "Restores health scaling with Intelligence.".to_string(),
                effect: AbilityEffect::Heal { base: 20, intel_scale: 2.0 },
            },
            Ability {
                name: "Chain Lightning".to_string(),
                mana_cost: 20,
                cooldown_seconds: 7.0,
                description: "Electrifying arcs dealing high damage scaling with Intelligence.".to_string(),
                effect: AbilityEffect::Damage { base: 22, strength_scale: 0.0, intel_scale: 2.2 },
            },
            Ability {
                name: "Fire Blast".to_string(),
                mana_cost: 18,
                cooldown_seconds: 4.5,
                description: "Explosive flame wave scaling with Intelligence.".to_string(),
                effect: AbilityEffect::Damage { base: 18, strength_scale: 0.0, intel_scale: 2.1 },
            },
            Ability {
                name: "Frostbolt".to_string(),
                mana_cost: 10,
                cooldown_seconds: 5.5,
                description: "Launches a freezing projectile scaling with Intelligence.".to_string(),
                effect: AbilityEffect::Damage { base: 12, strength_scale: 0.0, intel_scale: 1.6 },
            },
            Ability {
                name: "Primal Strike".to_string(),
                mana_cost: 0,
                cooldown_seconds: 3.5,
                description: "Melee strike infused with nature's fury.".to_string(),
                effect: AbilityEffect::Damage { base: 10, strength_scale: 1.0, intel_scale: 0.0 },
            },
            Ability {
                name: "Regrowth".to_string(),
                mana_cost: 15,
                cooldown_seconds: 8.0,
                description: "Infuses the target with natural healing.".to_string(),
                effect: AbilityEffect::Heal { base: 15, intel_scale: 1.5 },
            },
            Ability {
                name: "Lesser Heal".to_string(),
                mana_cost: 8,
                cooldown_seconds: 5.0,
                description: "Generic spell that restores some health.".to_string(),
                effect: AbilityEffect::Heal { base: 10, intel_scale: 1.0 },
            },
            Ability {
                name: "Fire Bolt".to_string(),
                mana_cost: 6,
                cooldown_seconds: 4.0,
                description: "Generic spell that shoots minor fire magic.".to_string(),
                effect: AbilityEffect::Damage { base: 8, strength_scale: 0.0, intel_scale: 1.0 },
            },
            Ability {
                name: "Shield Slam".to_string(),
                mana_cost: 0,
                cooldown_seconds: 5.0,
                description: "Strikes with shield, dealing physical damage scaling with Strength.".to_string(),
                effect: AbilityEffect::Damage { base: 5, strength_scale: 1.2, intel_scale: 0.0 },
            },
            Ability {
                name: "Serrated Strike".to_string(),
                mana_cost: 0,
                cooldown_seconds: 6.0,
                description: "Physical strike that inflicts a 3-second bleeding debuff.".to_string(),
                effect: AbilityEffect::Debuff { duration_seconds: 3.0, damage_per_tick: 6 },
            },
            Ability {
                name: "Iron Will".to_string(),
                mana_cost: 5,
                cooldown_seconds: 12.0,
                description: "Temporarily increases physical attack power.".to_string(),
                effect: AbilityEffect::Buff { duration_seconds: 6.0, strength_bonus: 8 },
            },
            Ability {
                name: "Aspect of the Beast".to_string(),
                mana_cost: 10,
                cooldown_seconds: 15.0,
                description: "Temporarily increases animalistic power.".to_string(),
                effect: AbilityEffect::Buff { duration_seconds: 8.0, strength_bonus: 12 },
            },
        ]
    }

    pub fn find_ability(name: &str) -> Option<Ability> {
        Self::get_all_abilities().into_iter().find(|a| a.name == name)
    }
}
