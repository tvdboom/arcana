use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(States, EnumIter, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Settings,
    Game,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub enum GameState {
    #[default]
    CreateCharacter,
    ChooseRace,
    ChooseClass,
    ChooseSubClass,
    Playing,
    Combat,
    CombatPaused,
    GameMenu,
    Settings,
    EndGame,
}
