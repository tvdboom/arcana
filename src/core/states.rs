use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(States, EnumIter, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Settings,
    Loading,
    Game,
}

#[derive(
    States, EnumIter, Debug, Clone, Copy, Eq, PartialEq, Hash, Default, Serialize, Deserialize,
)]
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
    Shop,
    Work,
    Study,
    Train,
    Rest,
    Craft,
    Hunt,
    Quest,
    Duel,
}

pub fn is_panel_state(state: GameState) -> bool {
    matches!(
        state,
        GameState::Shop
            | GameState::Work
            | GameState::Study
            | GameState::Train
            | GameState::Rest
            | GameState::Craft
            | GameState::Hunt
            | GameState::Quest
            | GameState::Duel
    )
}
