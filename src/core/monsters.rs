//! Monster kinds, generated combatants, and monster reward data.

use crate::core::catalog::effects::Effect;
use crate::core::catalog::modifiers::Modifier;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonsterArchetype {
    Berserker,
    Necromancer,
    Knight,
    Assassin,
    Leech,
    Mage,
    #[default]
    Beast,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonsterKind {
    #[default]
    Creature,
    Pet,
    Dragon,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Monster {
    pub name: String,
    pub image: String,
    pub level: u32,
    pub kind: MonsterKind,
    /// Governs the telegraphed move rotation used by this monster in PvE combat.
    #[serde(default)]
    pub archetype: MonsterArchetype,
    pub health: u32,
    pub max_health: u32,
    pub attack: u32,
    pub defense: u32,
    pub initiative: u32,
    pub attack_speed: f32,
    /// Health regenerated per second during combat.
    #[serde(default)]
    pub health_regen: i32,
    /// Reserved passive modifiers for imported or future monster definitions.
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    pub effects: Vec<Effect>,
}

impl Monster {
    /// Returns whether this monster's portrait comes from the named monster subdirectory.
    pub fn is_from_image_dir(&self, dir: &str) -> bool {
        self.image.contains(&format!("images/monsters/{dir}/"))
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ActiveMonster {
    pub monster: Monster,
}
