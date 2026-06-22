use crate::core::catalog::effects::Effect;
use crate::core::catalog::modifiers::Modifier;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragonColor {
    #[default]
    Black,
    Blue,
    Gold,
    Green,
    Red,
    Silver,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragonAge {
    #[default]
    Hatchling,
    Young,
    Juvenile,
    YoungAdult,
    Adult,
    Old,
    Ancient,
    Wyrm,
    GreatWyrm,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Monster {
    #[default]
    Bat,
    Bear,
    Crocodile,
    Dragon {
        color: DragonColor,
        age: DragonAge,
    },
    Eagle,
    Griffin,
    HellHound,
    Hyena,
    Lizard,
    Manticore,
    Owl,
    OwlBear,
    Pegasus,
    Puma,
    Rat,
    Snake,
    Spider,
    ThreeHeadedDog,
    Tiger,
    Unicorn,
    Vulture,
    Weasel,
    Wolf,
    Worg,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterStats {
    pub name: String,
    pub kind: Monster,
    pub health: u32,
    pub max_health: u32,
    pub attack: u32,
    pub defense: u32,
    pub initiative: u32,
    pub attack_speed: f32,
    pub modifiers: Vec<Modifier>,
    pub effects: Vec<Effect>,
}

impl MonsterStats {
    pub fn new(name: impl Into<String>, kind: Monster) -> Self {
        let (max_health, attack, defense, initiative) = match kind {
            Monster::Bat => (40, 10, 10, 10),
            Monster::Bear => (40, 10, 10, 10),
            Monster::Crocodile => (40, 10, 10, 10),
            Monster::Dragon {
                ..
            } => (40, 10, 10, 10),
            Monster::Eagle => (40, 10, 10, 10),
            Monster::Griffin => (40, 10, 10, 10),
            Monster::HellHound => (40, 10, 10, 10),
            Monster::Hyena => (40, 10, 10, 10),
            Monster::Lizard => (40, 10, 10, 10),
            Monster::Manticore => (40, 10, 10, 10),
            Monster::Owl => (40, 10, 10, 10),
            Monster::OwlBear => (40, 10, 10, 10),
            Monster::Pegasus => (40, 10, 10, 10),
            Monster::Puma => (40, 10, 10, 10),
            Monster::Rat => (40, 10, 10, 10),
            Monster::Snake => (40, 10, 10, 10),
            Monster::Spider => (40, 10, 10, 10),
            Monster::ThreeHeadedDog => (40, 10, 10, 10),
            Monster::Tiger => (40, 10, 10, 10),
            Monster::Unicorn => (40, 10, 10, 10),
            Monster::Vulture => (40, 10, 10, 10),
            Monster::Weasel => (40, 10, 10, 10),
            Monster::Wolf => (40, 10, 10, 10),
            Monster::Worg => (40, 10, 10, 10),
        };

        Self {
            name: name.into(),
            kind,
            health: max_health,
            max_health,
            attack,
            defense,
            initiative,
            attack_speed: 1.,
            modifiers: vec![],
            effects: vec![],
        }
    }
}
