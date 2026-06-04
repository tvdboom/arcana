use crate::core::audio::PlayAudioMsg;
use crate::core::menu::systems::StartNewCharacterMsg;
use crate::core::player::Player;
use crate::core::states::{AppState, GameState};
use bevy::prelude::*;

pub fn check_keys_menu(
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut start_new_char_msg: MessageWriter<StartNewCharacterMsg>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Res<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if keyboard.just_released(KeyCode::Escape) {
        match app_state.get() {
            AppState::Settings => {
                play_audio_msg.write(PlayAudioMsg::new("button"));
                next_app_state.set(AppState::MainMenu);
            },
            AppState::Game => match game_state.get() {
                GameState::Playing => next_game_state.set(GameState::GameMenu),
                GameState::GameMenu => next_game_state.set(GameState::Playing),
                GameState::EndGame => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_app_state.set(AppState::MainMenu);
                },
                GameState::Settings => next_game_state.set(GameState::GameMenu),
                GameState::CreateCharacter => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_app_state.set(AppState::MainMenu);
                },
                GameState::ChooseRace => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::CreateCharacter);
                },
                GameState::ChooseClass => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::ChooseRace);
                },
                GameState::ChooseSubClass => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::ChooseClass);
                },
                _ => (),
            },
            _ => (),
        }
    }

    if keyboard.just_released(KeyCode::Enter) {
        match app_state.get() {
            AppState::MainMenu => {
                play_audio_msg.write(PlayAudioMsg::new("button"));
                start_new_char_msg.write(StartNewCharacterMsg);
            },
            AppState::Settings => {
                play_audio_msg.write(PlayAudioMsg::new("button"));
                next_app_state.set(AppState::MainMenu);
            },
            AppState::Game => match game_state.get() {
                GameState::EndGame => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_app_state.set(AppState::MainMenu);
                },
                GameState::CreateCharacter => {
                    let current_sum = (player.strength
                        + player.dexterity
                        + player.constitution
                        + player.intelligence
                        + player.wisdom
                        + player.charisma) as i32;

                    if !player.name.trim().is_empty() && current_sum == 60 {
                        play_audio_msg.write(PlayAudioMsg::new("button"));
                        next_game_state.set(GameState::ChooseRace);
                    } else {
                        play_audio_msg.write(PlayAudioMsg::new("error"));
                    }
                },
                GameState::ChooseRace => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::ChooseClass);
                },
                GameState::ChooseClass => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::ChooseSubClass);
                },
                _ => (),
            },
        }
    }
}
