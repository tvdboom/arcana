use crate::core::menu::utils::TextSize;
use crate::core::states::{AppState, GameState};
use bevy::prelude::*;
use bevy::window::WindowResized;

pub fn on_resize_message(
    mut resize_msg: MessageReader<WindowResized>,
    mut text: Query<(&mut TextFont, &TextSize)>,
) {
}

pub fn check_keys_menu(
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_released(KeyCode::Escape) {
        match app_state.get() {
            AppState::Settings => next_app_state.set(AppState::MainMenu),
            AppState::Game => match game_state.get() {
                GameState::Playing => next_game_state.set(GameState::GameMenu),
                GameState::GameMenu => next_game_state.set(GameState::Playing),
                GameState::EndGame => next_app_state.set(AppState::MainMenu),
                GameState::Settings => next_game_state.set(GameState::GameMenu),
                _ => (),
            },
            _ => (),
        }
    }

    if keyboard.just_released(KeyCode::Enter) {
        match app_state.get() {
            AppState::MainMenu => next_app_state.set(AppState::Game),
            AppState::Settings => next_app_state.set(AppState::MainMenu),
            AppState::Game if *game_state.get() == GameState::EndGame => {
                next_app_state.set(AppState::MainMenu)
            },
            _ => (),
        }
    }
}

pub fn check_keys_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_released(KeyCode::Space) {
        match game_state.get() {
            GameState::Combat => next_game_state.set(GameState::CombatPaused),
            GameState::CombatPaused => next_game_state.set(GameState::Combat),
            _ => (),
        }
    }
}
