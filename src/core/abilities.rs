use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MagicType {
    Physical,
    Fire,
    Ice,
    Dark,
    Nature,
    Holy,
}

#[derive(Clone, Copy, Debug)]
pub struct AbilityStats {
    pub level: u8,
    pub magic_type: MagicType,
    pub mana_cost: u32,
    pub cooldown: u32, // turns
}

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ability {
    Slash,
    Firebolt,
    Backstab,
    Regrowth,
    Heal,
    ShieldBash,
    Shadowbolt,
    Frostbolt,
    Bite,
    Maul,
    PoisonBite,
    Swoop,
}

impl Ability {
    pub fn stats(&self) -> AbilityStats {
        match self {
            Ability::Slash => AbilityStats {
                level: 1,
                magic_type: MagicType::Physical,
                mana_cost: 0,
                cooldown: 0,
            },
            Ability::Firebolt => AbilityStats {
                level: 1,
                magic_type: MagicType::Fire,
                mana_cost: 15,
                cooldown: 1,
            },
            Ability::Backstab => AbilityStats {
                level: 1,
                magic_type: MagicType::Physical,
                mana_cost: 10,
                cooldown: 2,
            },
            Ability::Regrowth => AbilityStats {
                level: 1,
                magic_type: MagicType::Nature,
                mana_cost: 20,
                cooldown: 3,
            },
            Ability::Heal => AbilityStats {
                level: 1,
                magic_type: MagicType::Holy,
                mana_cost: 25,
                cooldown: 2,
            },
            Ability::ShieldBash => AbilityStats {
                level: 2,
                magic_type: MagicType::Physical,
                mana_cost: 5,
                cooldown: 1,
            },
            Ability::Shadowbolt => AbilityStats {
                level: 1,
                magic_type: MagicType::Dark,
                mana_cost: 30,
                cooldown: 2,
            },
            Ability::Frostbolt => AbilityStats {
                level: 1,
                magic_type: MagicType::Ice,
                mana_cost: 20,
                cooldown: 1,
            },
            Ability::Bite => AbilityStats {
                level: 1,
                magic_type: MagicType::Physical,
                mana_cost: 0,
                cooldown: 1,
            },
            Ability::Maul => AbilityStats {
                level: 2,
                magic_type: MagicType::Physical,
                mana_cost: 0,
                cooldown: 2,
            },
            Ability::PoisonBite => AbilityStats {
                level: 3,
                magic_type: MagicType::Nature,
                mana_cost: 10,
                cooldown: 2,
            },
            Ability::Swoop => AbilityStats {
                level: 2,
                magic_type: MagicType::Physical,
                mana_cost: 5,
                cooldown: 1,
            },
        }
    }

    pub fn level(&self) -> u8 {
        self.stats().level
    }
}
