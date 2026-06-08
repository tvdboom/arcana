use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub struct PetStats {
    pub health: i32,
    pub armor: i32,
    pub attack: i32,
    pub initiative: i32,
}

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pet {
    #[default]
    Bat,
    Bear,
    Crocodile,
    Eagle,
    Hyena,
    InfernalCan,
    Lizard,
    Pegasus,
    Rat,
    Snake,
    Spider,
    ThreeHeadedDog,
    Tiger,
    Unicorn,
    Vulture,
    Wolf,
}

impl Pet {
    pub fn stats(&self) -> PetStats {
        match self {
            Pet::Rat            => PetStats { health: 20,  armor: 1, attack: 2,  initiative: 3 },
            Pet::Bat            => PetStats { health: 25,  armor: 2, attack: 3,  initiative: 5 },
            Pet::Lizard         => PetStats { health: 30,  armor: 3, attack: 4,  initiative: 2 },
            Pet::Snake          => PetStats { health: 40,  armor: 2, attack: 5,  initiative: 3 },
            Pet::Spider         => PetStats { health: 35,  armor: 3, attack: 4,  initiative: 4 },
            Pet::Vulture        => PetStats { health: 45,  armor: 2, attack: 5,  initiative: 4 },
            Pet::Hyena          => PetStats { health: 60,  armor: 4, attack: 7,  initiative: 4 },
            Pet::Eagle          => PetStats { health: 55,  armor: 2, attack: 7,  initiative: 7 },
            Pet::Crocodile      => PetStats { health: 80,  armor: 8, attack: 8,  initiative: 1 },
            Pet::Wolf           => PetStats { health: 70,  armor: 4, attack: 9,  initiative: 6 },
            Pet::Tiger          => PetStats { health: 75,  armor: 4, attack: 11, initiative: 7 },
            Pet::Bear           => PetStats { health: 100, armor: 8, attack: 10, initiative: 2 },
            Pet::Pegasus        => PetStats { health: 80,  armor: 5, attack: 7,  initiative: 8 },
            Pet::InfernalCan    => PetStats { health: 70,  armor: 5, attack: 9,  initiative: 5 },
            Pet::ThreeHeadedDog => PetStats { health: 90,  armor: 7, attack: 12, initiative: 3 },
            Pet::Unicorn        => PetStats { health: 90,  armor: 6, attack: 8,  initiative: 6 },
        }
    }
}
