use crate::core::inventory::armor::Armor;
use crate::core::inventory::effects::Effect;
use crate::core::inventory::modifiers::Modifier;
use crate::core::inventory::weapons::Weapon;
use crate::core::player::Player;
use serde::Deserialize;
use strum_macros::Display;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Deserialize)]
pub enum Kind {
    // Weapons
    Martial,
    Bulwark,
    Assassination,
    Skirmish,
    Tactic,
    Command,
    // Magic
    Fire,
    Frost,
    Lightning,
    Nature,
    Holy,
    Shadow,
    Cosmic,
}

impl Kind {
    pub fn is_magic(&self) -> bool {
        matches!(
            self,
            Kind::Fire
                | Kind::Frost
                | Kind::Lightning
                | Kind::Nature
                | Kind::Holy
                | Kind::Shadow
                | Kind::Cosmic
        )
    }
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

    pub fn kind(&self) -> Kind {
        match self {
            Equipment::Armor(a) => a.kind,
            Equipment::Weapon(w) => w.kind,
        }
    }

    pub fn attack(&self) -> i32 {
        let base = match self {
            Equipment::Weapon(w) => w.base_attack as i32,
            Equipment::Armor(_) => 0,
        };
        let mut bonus = 0;
        for modifier in self.modifiers() {
            if let Modifier::BonusAttack(val) = modifier {
                bonus += val;
            }
        }
        base + bonus
    }

    pub fn defense(&self) -> i32 {
        let base = match self {
            Equipment::Armor(a) => a.base_defense as i32,
            Equipment::Weapon(_) => 0,
        };
        let mut bonus = 0;
        for modifier in self.modifiers() {
            if let Modifier::BonusDefense(val) = modifier {
                bonus += val;
            }
        }
        base + bonus
    }

    pub fn initiative(&self) -> i32 {
        let mut bonus = 0;
        for modifier in self.modifiers() {
            if let Modifier::BonusInitiative(val) = modifier {
                bonus += val;
            }
        }
        bonus
    }

    pub fn attack_speed(&self) -> f32 {
        match self {
            Equipment::Weapon(w) => w.attack_speed,
            Equipment::Armor(_) => 1.0,
        }
    }

    pub fn crit(&self) -> i32 {
        match self {
            Equipment::Weapon(w) => (w.crit_chance * 100.0) as i32,
            Equipment::Armor(_) => 0,
        }
    }

    pub fn modifiers(&self) -> &[Modifier] {
        match self {
            Equipment::Armor(a) => &a.modifiers,
            Equipment::Weapon(w) => &w.modifiers,
        }
    }

    pub fn effect(&self) -> &[Effect] {
        match self {
            Equipment::Armor(a) => &a.effects,
            Equipment::Weapon(w) => &w.effects,
        }
    }
}
