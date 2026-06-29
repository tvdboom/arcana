use crate::core::actions::gain_xp;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::{Ajah, Class};
use crate::core::menu::systems::{CombatMenuSuspended, GameMenuOrigin, StartNewCharacterMsg};
use crate::core::player::Player;
use crate::core::states::{AppState, GameState};
use crate::core::ui::creation::SelectionItem;
use crate::core::ui::level_up::{ApplyLevelUpMsg, LevelUpPending};
use crate::core::ui::modal::ActiveModal;
use bevy::prelude::*;
use rand::{rng, RngExt};

pub fn check_keys_menu(
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut start_new_char_msg: MessageWriter<StartNewCharacterMsg>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    mut apply_level_up_msg: MessageWriter<ApplyLevelUpMsg>,
    active_modal: Res<ActiveModal>,
    mut game_menu_origin: ResMut<GameMenuOrigin>,
    mut combat_menu_suspended: ResMut<CombatMenuSuspended>,
    mut state: Option<ResMut<crate::core::combat::mechanics::CombatState>>,
    duel_active: Option<Res<crate::core::combat::mechanics::DuelActive>>,
) {
    let cheat_level_up = *app_state.get() == AppState::Game
        && keyboard.just_released(KeyCode::ArrowUp)
        && (keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight))
        && (keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight));
    if cheat_level_up {
        let old_level = player.level();
        gain_xp(&mut player, 10, &mut level_up, &mut play_audio_msg, &mut next_game_state, true);
        if player.level() > old_level && level_up.active {
            let mut rng = rng();
            while level_up.points_remaining > 0 {
                let idx = rng.random_range(0..level_up.attr_gains.len());
                if level_up.attr_gains[idx] < 2 {
                    level_up.attr_gains[idx] += 1;
                    level_up.points_remaining -= 1;
                }
            }
            apply_level_up_msg.write(ApplyLevelUpMsg);
        }
        player.gold = player.gold.saturating_add(1000);
        return;
    }

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
            AppState::Loading => {},
            AppState::Game => match game_state.get() {
                GameState::Playing => {
                    game_menu_origin.0 = Some(GameState::Playing);
                    combat_menu_suspended.0 = false;
                    next_game_state.set(GameState::GameMenu);
                },
                GameState::Combat => {
                    game_menu_origin.0 = Some(GameState::Combat);
                    combat_menu_suspended.0 = true;
                    if duel_active.is_none() {
                        if let Some(s) = state.as_mut() {
                            s.paused = true;
                        }
                    }
                    next_game_state.set(GameState::GameMenu);
                },
                GameState::Shop
                | GameState::Work
                | GameState::Study
                | GameState::Train
                | GameState::Rest
                | GameState::Craft
                | GameState::Hunt
                | GameState::Quest
                | GameState::Duel => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    next_game_state.set(GameState::Playing);
                },
                GameState::GameMenu => {
                    let target = game_menu_origin.0.unwrap_or(GameState::Playing);
                    combat_menu_suspended.0 = false;
                    game_menu_origin.0 = None;
                    if duel_active.is_none() {
                        if let Some(s) = state.as_mut() {
                            s.paused = false;
                        }
                    }
                    next_game_state.set(target);
                },
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
            AppState::Loading => {},
            AppState::Game => match game_state.get() {
                GameState::Combat => {
                    game_menu_origin.0 = Some(GameState::Combat);
                    combat_menu_suspended.0 = true;
                    if duel_active.is_none() {
                        if let Some(s) = state.as_mut() {
                            s.paused = true;
                        }
                    }
                    next_game_state.set(GameState::GameMenu);
                },
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
                            let kind = player.pet.as_ref().map(|p| p.kind).unwrap_or_default();
                            kind.on_select(&mut player, &mut next_game_state);
                        },
                        _ => {
                            next_game_state.set(GameState::CreateCharacter);
                        },
                    }
                },
                GameState::GameMenu | GameState::Settings => {
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                    let target = game_menu_origin.0.unwrap_or(GameState::Playing);
                    combat_menu_suspended.0 = false;
                    game_menu_origin.0 = None;
                    if duel_active.is_none() {
                        if let Some(s) = state.as_mut() {
                            s.paused = false;
                        }
                    }
                    next_game_state.set(target);
                },
                _ => (),
            },
        }
    }
}
