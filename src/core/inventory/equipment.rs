use crate::core::inventory::armor::Armor;
use crate::core::inventory::effects::Effect;
use crate::core::inventory::modifiers::Modifier;
use crate::core::inventory::weapons::Weapon;
use crate::core::player::Player;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Debuff {
    /// Damage
    Burn,

    /// Reduces attack speed
    Freeze,

    /// Reduces initiative
    Paranoia,

    /// Damage
    Poison,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Equipment {
    Armor(Armor),
    Weapon(Weapon),
}

impl Equipment {
    pub fn description(&self, player: &Player) -> String {
        match self {
            Equipment::Armor(a) => a.description(player),
            Equipment::Weapon(w) => w.description(player),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Equipment::Armor(a) => &a.name,
            Equipment::Weapon(w) => &w.name,
        }
    }

    pub fn level(&self) -> u32 {
        match self {
            Equipment::Armor(a) => a.level,
            Equipment::Weapon(w) => w.level,
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            Equipment::Armor(a) => a.price,
            Equipment::Weapon(w) => w.price,
        }
    }

    pub fn attack(&self) -> i32 {
        let base = match self {
            Equipment::Weapon(w) => w.attack as i32,
            Equipment::Armor(_) => 0,
        };
        let mut bonus = 0;
        for modifier in self.modifiers() {
            if let Modifier::AttackModifier(val) = modifier {
                bonus += val;
            }
        }
        base + bonus
    }

    pub fn defense(&self) -> i32 {
        self.modifiers()
            .iter()
            .filter_map(|m| {
                if let Modifier::DefenseModifier(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn initiative(&self) -> i32 {
        self.modifiers()
            .iter()
            .filter_map(|m| {
                if let Modifier::InitiativeModifier(v) = m {
                    Some(v)
                } else {
                    None
                }
            })
            .sum()
    }

    pub fn modifiers(&self) -> &[Modifier] {
        match self {
            Equipment::Armor(a) => &a.modifiers,
            Equipment::Weapon(w) => &w.modifiers,
        }
    }

    pub fn effects(&self) -> &[Effect] {
        match self {
            Equipment::Armor(a) => &a.effects,
            Equipment::Weapon(w) => &w.effects,
        }
    }
}
