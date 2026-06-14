use crate::core::build::modifiers::Modifier;
use crate::core::build::weapons::Weapon;
use crate::core::build::wearables::Wearable;
use crate::core::localization::Localization;
use crate::core::settings::Language;
use serde::Deserialize;
use strum_macros::Display;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Deserialize)]
pub enum Kind {
    Physical,
    Fire,
    Ice,
    Nature,
    Holy,
    Shadow,
}

impl Kind {
    pub fn is_magic(&self) -> bool {
        self != &Kind::Physical
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum Equipment {
    Wearable(Wearable),
    Weapon(Weapon),
}

impl Equipment {
    pub fn description(&self, language: Language, localization: &Localization) -> String {
        match self {
            Equipment::Wearable(a) => a.description(language, localization),
            Equipment::Weapon(w) => w.description(language, localization),
        }
    }

    pub fn full_description(&self, language: Language, localization: &Localization) -> Vec<String> {
        match self {
            Equipment::Wearable(a) => a.full_description(language, localization),
            Equipment::Weapon(w) => w.full_description(language, localization),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Equipment::Wearable(a) => &a.name,
            Equipment::Weapon(w) => &w.name,
        }
    }

    pub fn level(&self) -> u32 {
        match self {
            Equipment::Wearable(a) => a.level,
            Equipment::Weapon(w) => w.level,
        }
    }

    pub fn price(&self) -> u32 {
        match self {
            Equipment::Wearable(a) => a.price,
            Equipment::Weapon(w) => w.price,
        }
    }

    pub fn sell_price(&self, modifier: i32) -> u32 {
        match self {
            Equipment::Wearable(a) => (a.price as f32 * (0.5 + 0.01 * modifier as f32)) as u32,
            Equipment::Weapon(w) => (w.price as f32 * (0.5 + 0.01 * modifier as f32)) as u32,
        }
    }

    pub fn attack(&self) -> i32 {
        let base = match self {
            Equipment::Weapon(w) => w.attack as i32,
            Equipment::Wearable(_) => 0,
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
            Equipment::Wearable(a) => &a.modifiers,
            Equipment::Weapon(w) => &w.modifiers,
        }
    }
}
