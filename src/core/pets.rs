use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetKind {
    #[default]
    Bat,
    Bear,
    Crocodile,
    Eagle,
    Griffin,
    Hyena,
    InfernalCan,
    Lizard,
    Manticore,
    Pegasus,
    Puma,
    Rat,
    Snake,
    Spider,
    ThreeHeadedDog,
    Tiger,
    Unicorn,
    Vulture,
    Wolf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pet {
    pub name: String,
    pub kind: PetKind,
    pub health: i32,
    pub max_health: i32,
    pub attack: i32,
    pub defense: i32,
    pub initiative: i32,
}

impl Pet {
    pub fn new(name: impl Into<String>, kind: PetKind) -> Self {
        let (max_health, attack, defense, initiative) = match kind {
            PetKind::Bat => (40, 10, 10, 10),
            PetKind::Bear => (40, 10, 10, 10),
            PetKind::Crocodile => (40, 10, 10, 10),
            PetKind::Eagle => (40, 10, 10, 10),
            PetKind::Griffin => (40, 10, 10, 10),
            PetKind::Hyena => (40, 10, 10, 10),
            PetKind::InfernalCan => (40, 10, 10, 10),
            PetKind::Lizard => (40, 10, 10, 10),
            PetKind::Manticore => (40, 10, 10, 10),
            PetKind::Pegasus => (40, 10, 10, 10),
            PetKind::Puma => (40, 10, 10, 10),
            PetKind::Rat => (40, 10, 10, 10),
            PetKind::Snake => (40, 10, 10, 10),
            PetKind::Spider => (40, 10, 10, 10),
            PetKind::ThreeHeadedDog => (40, 10, 10, 10),
            PetKind::Tiger => (40, 10, 10, 10),
            PetKind::Unicorn => (40, 10, 10, 10),
            PetKind::Vulture => (40, 10, 10, 10),
            PetKind::Wolf => (40, 10, 10, 10),
        };

        Self {
            name: name.into(),
            kind,
            health: max_health,
            max_health,
            attack,
            defense,
            initiative,
        }
    }
}
