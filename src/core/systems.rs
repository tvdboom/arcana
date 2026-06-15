use crate::core::audio::PlayAudioMsg;
use crate::core::classes::{Ajah, Class};
use crate::core::menu::systems::StartNewCharacterMsg;
use crate::core::pets::PetKind;
use crate::core::player::Player;
use crate::core::states::{AppState, GameState};
use crate::core::ui::creation::SelectionItem;
use crate::core::ui::level_up::{ApplyLevelUpMsg, LevelUpPending};
use crate::core::ui::modal::ActiveModal;
use bevy::prelude::*;

pub fn check_keys_menu(
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut start_new_char_msg: MessageWriter<StartNewCharacterMsg>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    level_up: Res<LevelUpPending>,
    mut apply_level_up_msg: MessageWriter<ApplyLevelUpMsg>,
    active_modal: Res<ActiveModal>,
) {
    if keyboard.just_released(KeyCode::Escape) {
        if active_modal.active {
            return;
        }
        if level_up.active {
            // Disable game menu / escape key when level up overlay is active
            return;
        }
        match app_state.get() {
            AppState::Settings => {
                play_audio_msg.write(PlayAudioMsg::new("button"));
                next_app_state.set(AppState::MainMenu);
            },
            AppState::Game => match game_state.get() {
                GameState::Playing => next_game_state.set(GameState::GameMenu),
                GameState::Shop
                | GameState::Work
                | GameState::Study
                | GameState::Train
                | GameState::Rest => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::Playing);
                },
                GameState::GameMenu => next_game_state.set(GameState::Playing),
                GameState::EndGame => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_app_state.set(AppState::MainMenu);
                },
                GameState::Settings => next_game_state.set(GameState::GameMenu),
                GameState::ChooseRace => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::CreateCharacter);
                },
                GameState::ChooseClass => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::ChooseRace);
                },
                GameState::ChooseSubClass => {
                    player.pet = None; // Reset pet selection
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::ChooseClass);
                },
                GameState::CreateCharacter => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_app_state.set(AppState::MainMenu);
                },
                _ => (),
            },
            _ => (),
        }
    }

    if keyboard.just_released(KeyCode::Enter) {
        if active_modal.active {
            return;
        }
        if level_up.active {
            let ability_ok =
                level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
            let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
            if level_up.points_remaining == 0 && ability_ok && perk_ok {
                apply_level_up_msg.write(ApplyLevelUpMsg);
            } else {
                play_audio_msg.write(PlayAudioMsg::new("error"));
            }
            return;
        }

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
                    let race = player.race;
                    race.on_select(&mut player, &mut next_game_state);
                },
                GameState::ChooseClass => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    let class = player.class;
                    class.on_select(&mut player, &mut next_game_state);
                },
                GameState::ChooseSubClass => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    match player.class {
                        Class::Mage(_) => {
                            let ajah = Ajah::default();
                            ajah.on_select(&mut player, &mut next_game_state);
                        },
                        Class::Druid => {
                            let kind =
                                player.pet.as_ref().map(|p| p.kind).unwrap_or(PetKind::default());
                            kind.on_select(&mut player, &mut next_game_state);
                        },
                        _ => {
                            next_game_state.set(GameState::CreateCharacter);
                        },
                    }
                },
                _ => (),
            },
        }
    }
}
