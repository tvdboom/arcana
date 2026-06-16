pub mod craft;
pub mod duel;
pub mod hunt;
pub mod quest;
pub mod rest;
pub mod shop;
pub mod study;
pub mod train;
pub mod work;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::abilities::Ability;
use crate::core::catalog::equipment::Kind;
use crate::core::catalog::catalog::{all_abilities, all_perks};
use crate::core::classes::Class;
use crate::core::localization::Localization;
use crate::core::menu::buttons::DisabledButton;
use crate::core::player::{Attribute, Player};
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::utils::{capitalize_words, NameFromEnum};
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{rng, RngExt};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(
    EnumString, Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize,
)]
pub enum Action {
    Rest,
    Study,
    Work,
    Train,
    Craft,
    Shop,
    Hunt,
    Quest,
    Duel,
}

impl Action {
    pub fn ap_cost(&self) -> u32 {
        match self {
            Action::Shop
            | Action::Duel
            | Action::Work
            | Action::Study
            | Action::Train
            | Action::Rest => 0,
            Action::Craft | Action::Hunt => 2,
            Action::Quest => 3,
        }
    }
}

#[derive(Component)]
pub struct ActionButton(pub Action);

// Reusable level up helper
pub fn trigger_level_up(
    player: &mut Player,
    level_up: &mut LevelUpPending,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    next_game_state: &mut NextState<GameState>,
) {
    next_game_state.set(GameState::Playing);
    let mut rng = rng();
    player.level += 1;
    player.ap = 10;
    player.bonus_max_health += 10;
    player.bonus_max_mana += 10;

    if let Some(pet) = &mut player.pet {
        pet.health += 10;
        pet.max_health += 10;
    }

    let mut ability_choices = Vec::new();
    let ability_pool: Vec<_> = all_abilities()
        .iter()
        .filter(|ab| {
            ab.level == player.level as u32 && !player.abilities.contains(&ab.name.to_string())
        })
        .collect();

    let mut weighted_pool: Vec<(&Ability, f64)> = ability_pool
        .iter()
        .map(|ab| {
            let mut weight = 1.0;
            let is_magical = ab.kind.is_magic();
            if matches!(player.class, Class::Druid | Class::Mage(_)) && is_magical {
                weight *= 2.0;
            }
            if let Class::Mage(ajah) = player.class {
                if ab.kind == ajah.kind() {
                    weight *= 3.0;
                }
            }
            if matches!(player.class, Class::Warrior | Class::Assassin) && ab.kind == Kind::Physical
            {
                weight *= 2.0;
            }
            (*ab, weight)
        })
        .collect();

    for _ in 0..3 {
        if weighted_pool.is_empty() {
            break;
        }
        let total_weight: f64 = weighted_pool.iter().map(|(_, w)| *w).sum();
        if total_weight <= 0.0 {
            let idx = rng.random_range(0..weighted_pool.len());
            let (ab, _) = weighted_pool.remove(idx);
            ability_choices.push(ab.name.to_string());
        } else {
            let mut r = rng.random_range(0.0..total_weight);
            let mut chosen_idx = 0;
            for (idx, (_, w)) in weighted_pool.iter().enumerate() {
                r -= *w;
                if r <= 0.0 {
                    chosen_idx = idx;
                    break;
                }
            }
            let (ab, _) = weighted_pool.remove(chosen_idx);
            ability_choices.push(ab.name.to_string());
        }
    }

    let mut perk_choices = Vec::new();
    let mut perk_pool: Vec<_> = all_perks()
        .iter()
        .filter(|pk| {
            pk.level == player.level as u32 && !player.perks.contains(&pk.name.to_string())
        })
        .collect();
    for _ in 0..3 {
        if perk_pool.is_empty() {
            break;
        }
        let idx = rng.random_range(0..perk_pool.len());
        perk_choices.push(perk_pool[idx].name.to_string());
        perk_pool.remove(idx);
    }

    let ability_chosen = if !ability_choices.is_empty() {
        Some(0)
    } else {
        None
    };
    let perk_chosen = if !perk_choices.is_empty() {
        Some(0)
    } else {
        None
    };

    *level_up = LevelUpPending {
        active: true,
        new_level: player.level,
        points_remaining: 2,
        attr_gains: [0; 6],
        ability_choices,
        perk_choices,
        ability_chosen,
        perk_chosen,
    };

    play_audio_msg.write(PlayAudioMsg::new("levelup").volume(-10.));
}

pub fn handle_playing_action_clicks(
    event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    action_btn_q: Query<&ActionButton, Without<DisabledButton>>,
    _game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(action_btn) = action_btn_q.get(event.entity) {
        let action = action_btn.0;

        let current_state = _game_state.get();

        // Toggle behavior: if clicking the button of the action that is currently open, close it.
        let is_currently_open = match (action, current_state) {
            (Action::Shop, GameState::Shop) => true,
            (Action::Work, GameState::Work) => true,
            (Action::Study, GameState::Study) => true,
            (Action::Train, GameState::Train) => true,
            (Action::Rest, GameState::Rest) => true,
            _ => false,
        };

        if is_currently_open {
            next_game_state.set(GameState::Playing);
            play_audio_msg.write(PlayAudioMsg::new("button"));
            return;
        }

        // Close any open panel if clicking a non-panel action
        if *current_state != GameState::Playing
            && matches!(action, Action::Hunt | Action::Craft | Action::Quest | Action::Duel)
        {
            next_game_state.set(GameState::Playing);
        }

        match action {
            Action::Shop => {
                next_game_state.set(GameState::Shop);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Work => {
                next_game_state.set(GameState::Work);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Study => {
                next_game_state.set(GameState::Study);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Train => {
                next_game_state.set(GameState::Train);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Rest => {
                next_game_state.set(GameState::Rest);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Hunt => {
                hunt::handle_hunt(&mut player);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Craft => {
                craft::handle_craft(&mut player);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Quest => {
                quest::handle_quest(&mut player);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Duel => {
                duel::handle_duel(&mut player);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
        }

        // Deduct action points for standard non-panel actions
        if matches!(action, Action::Hunt | Action::Craft | Action::Quest) {
            let cost = action.ap_cost();
            if player.ap <= cost {
                trigger_level_up(
                    &mut player,
                    &mut level_up,
                    &mut play_audio_msg,
                    &mut next_game_state,
                );
            } else {
                player.ap -= cost;
            }
        }
    }
}

// System to handle click on work cards
pub fn handle_work_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    card_q: Query<&work::WorkCardMarker>,
    slider_state: Res<work::WorkSliderState>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(marker) = card_q.get(event.entity) {
        let slider_val = slider_state.0;
        let ap_cost = slider_val + 1;

        let lang = settings.language;
        let toast = toast_container_q.single().unwrap();

        if player.ap < ap_cost {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            spawn_toast(
                &mut commands,
                &assets,
                localization.get("not_enough_ap", lang),
                Color::srgba(0.20, 0.05, 0.05, 0.93),
                Color::srgb(0.85, 0.20, 0.20),
                Color::srgb(1.0, 0.80, 0.80),
                toast,
            );
            return;
        }

        let slider_mult = [1.0, 2.5, 4.0][slider_val as usize];

        // Fixed costs calculations:
        let craft_percentage =
            (10.0 + player.level as f32 * 0.5 - player.charisma_mod() as f32).max(1.0);
        let craft_cost =
            ((craft_percentage / 100.0) * player.max_mana() as f32 * slider_mult).max(1.0) as u32;

        let manual_percentage =
            (14.0 + player.level as f32 * 0.5 - player.charisma_mod() as f32).max(1.0);
        let manual_cost = ((manual_percentage / 100.0) * player.max_health() as f32 * slider_mult)
            .max(1.0) as u32;

        match marker.0 {
            0 => {
                // Clerical Labor has no health/mana costs
            },
            1 => {
                if player.mana() < craft_cost {
                    play_audio_msg.write(PlayAudioMsg::new("error"));
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization.get("not_enough_mana", lang),
                        Color::srgba(0.20, 0.05, 0.05, 0.93),
                        Color::srgb(0.85, 0.20, 0.20),
                        Color::srgb(1.0, 0.80, 0.80),
                        toast,
                    );
                    return;
                }
            },
            2 => {
                if player.health() <= manual_cost {
                    play_audio_msg.write(PlayAudioMsg::new("error"));
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization.get("not_enough_health", lang),
                        Color::srgba(0.20, 0.05, 0.05, 0.93),
                        Color::srgb(0.85, 0.20, 0.20),
                        Color::srgb(1.0, 0.80, 0.80),
                        toast,
                    );
                    return;
                }
            },
            _ => {},
        }

        let mut rng = rng();

        match marker.0 {
            0 => {
                // Clerical Labor
                let base = (1.0 + player.charisma_mod() as f32)
                    * (player.level as f32).powf(1.2)
                    * 4.0
                    * slider_mult;
                let min_gold = (base * 0.8).max(1.0) as u32;
                let max_gold = (base * 1.2).max(2.0) as u32;
                let gold_earned = rng.random_range(min_gold..=max_gold);

                player.gold += gold_earned;

                spawn_toast(
                    &mut commands,
                    &assets,
                    localization
                        .get("toast_gold_earned", lang)
                        .replace("{gold}", &gold_earned.to_string()),
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    toast,
                );
            },
            1 => {
                // Craft Labor
                let base = (1.0 + player.charisma_mod() as f32)
                    * (player.level as f32).powf(1.2)
                    * 5.0
                    * slider_mult;
                let min_gold = (base * 0.8).max(1.0) as u32;
                let max_gold = (base * 1.2).max(2.0) as u32;
                let gold_earned = rng.random_range(min_gold..=max_gold);

                player.gold += gold_earned;
                let next_mana = player.mana().saturating_sub(craft_cost);
                player.set_mana(next_mana);

                spawn_toast(
                    &mut commands,
                    &assets,
                    localization
                        .get("earned_gold_lost_mana", lang)
                        .replace("{gold}", &gold_earned.to_string())
                        .replace("{mana}", &craft_cost.to_string()),
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    toast,
                );
            },
            2 => {
                // Manual Labor
                let base = (1.0 + player.charisma_mod() as f32)
                    * (player.level as f32).powf(1.2)
                    * 7.0
                    * slider_mult;
                let min_gold = (base * 0.8).max(1.0) as u32;
                let max_gold = (base * 1.2).max(2.0) as u32;
                let gold_earned = rng.random_range(min_gold..=max_gold);

                player.gold += gold_earned;
                let next_health = player.health().saturating_sub(manual_cost).max(1);
                player.set_health(next_health);

                spawn_toast(
                    &mut commands,
                    &assets,
                    localization
                        .get("earned_gold_lost_health", lang)
                        .replace("{gold}", &gold_earned.to_string())
                        .replace("{health}", &manual_cost.to_string()),
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    toast,
                );
            },
            _ => {},
        }

        play_audio_msg.write(PlayAudioMsg::new("work"));

        if player.ap <= ap_cost {
            trigger_level_up(&mut player, &mut level_up, &mut play_audio_msg, &mut next_game_state);
        } else {
            player.ap -= ap_cost;
        }
    }
}

// System to handle click on study cards
pub fn handle_study_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    card_q: Query<&study::StudyCardMarker>,
    slider_state: Res<study::StudySliderState>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(marker) = card_q.get(event.entity) {
        let slider_val = slider_state.0;
        let ap_cost = slider_val + 1;

        let lang = settings.language;
        let toast = toast_container_q.single().unwrap();

        if player.ap < ap_cost {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            spawn_toast(
                &mut commands,
                &assets,
                localization.get("not_enough_ap", lang),
                Color::srgba(0.20, 0.05, 0.05, 0.93),
                Color::srgb(0.85, 0.20, 0.20),
                Color::srgb(1.0, 0.80, 0.80),
                toast,
            );
            return;
        }

        let mut rng = rng();
        let chance = 40 + player.intelligence_mod() * 5;

        // Determine target level based on slider value
        let offset: i32 = match slider_val {
            0 => {
                // Light: heavily weighted lower
                let r = rng.random_range(0..100);
                if r < 40 {
                    -2
                } else if r < 70 {
                    -1
                } else if r < 90 {
                    0
                } else if r < 98 {
                    1
                } else {
                    2
                }
            },
            1 => {
                // Regular: symmetric
                let r = rng.random_range(0..100);
                if r < 15 {
                    -2
                } else if r < 35 {
                    -1
                } else if r < 65 {
                    0
                } else if r < 85 {
                    1
                } else {
                    2
                }
            },
            2 => {
                // Heavy: heavily weighted higher
                let r = rng.random_range(0..100);
                if r < 2 {
                    -2
                } else if r < 10 {
                    -1
                } else if r < 30 {
                    0
                } else if r < 60 {
                    1
                } else {
                    2
                }
            },
            _ => 0,
        };

        let target_level = (player.level as i32 + offset).clamp(1, 20) as u32;

        match marker.0 {
            0 => {
                // Apprenticeship (learn ability)
                let roll = rng.random_range(0..100);
                if roll < chance {
                    let candidates: Vec<_> = all_abilities()
                        .iter()
                        .filter(|ab| {
                            ab.level == target_level
                                && !player.abilities.contains(&ab.name.to_string())
                        })
                        .collect();

                    if let Some(ability) = candidates.choose(&mut rng) {
                        let name = capitalize_words(&ability.name.to_string());
                        player.abilities.push(ability.name.to_string());
                        spawn_toast(
                            &mut commands,
                            &assets,
                            localization
                                .get("toast_study_ability", lang)
                                .replace("{ability}", &name),
                            Color::srgba(0.08, 0.10, 0.20, 0.93),
                            Color::srgb(0.35, 0.55, 0.90),
                            Color::srgb(0.75, 0.90, 1.0),
                            toast,
                        );
                    } else {
                        // Fallback: search range -2..=+2
                        let candidates_any: Vec<_> = all_abilities()
                            .iter()
                            .filter(|ab| {
                                let diff = (ab.level as i32 - player.level as i32).abs();
                                diff <= 2 && !player.abilities.contains(&ab.name.to_string())
                            })
                            .collect();

                        if let Some(ability) = candidates_any.choose(&mut rng) {
                            let name = capitalize_words(&ability.name.to_string());
                            player.abilities.push(ability.name.to_string());
                            spawn_toast(
                                &mut commands,
                                &assets,
                                localization
                                    .get("toast_study_ability", lang)
                                    .replace("{ability}", &name),
                                Color::srgba(0.08, 0.10, 0.20, 0.93),
                                Color::srgb(0.35, 0.55, 0.90),
                                Color::srgb(0.75, 0.90, 1.0),
                                toast,
                            );
                        } else {
                            // Secondary Fallback: Increase Max Mana
                            player.bonus_max_mana += 5;
                            spawn_toast(
                                &mut commands,
                                &assets,
                                localization.get("ability_pool_exhausted", lang),
                                Color::srgba(0.08, 0.10, 0.20, 0.93),
                                Color::srgb(0.35, 0.55, 0.90),
                                Color::srgb(0.75, 0.90, 1.0),
                                toast,
                            );
                        }
                    }
                } else {
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization.get("toast_study_nothing", lang),
                        Color::srgba(0.08, 0.10, 0.20, 0.93),
                        Color::srgb(0.35, 0.55, 0.90),
                        Color::srgb(0.75, 0.90, 1.0),
                        toast,
                    );
                }
            },
            1 => {
                // Mentorship (learn perk)
                let roll = rng.random_range(0..100);
                if roll < chance {
                    let candidates: Vec<_> = all_perks()
                        .iter()
                        .filter(|pk| {
                            pk.level == target_level && !player.perks.contains(&pk.name.to_string())
                        })
                        .collect();

                    if let Some(perk) = candidates.choose(&mut rng) {
                        let name = capitalize_words(&perk.name.to_string());
                        player.perks.push(perk.name.to_string());
                        spawn_toast(
                            &mut commands,
                            &assets,
                            localization.get("toast_study_perk", lang).replace("{perk}", &name),
                            Color::srgba(0.08, 0.10, 0.20, 0.93),
                            Color::srgb(0.35, 0.55, 0.90),
                            Color::srgb(0.75, 0.90, 1.0),
                            toast,
                        );
                    } else {
                        // Fallback: search range -2..=+2
                        let candidates_any: Vec<_> = all_perks()
                            .iter()
                            .filter(|pk| {
                                let diff = (pk.level as i32 - player.level as i32).abs();
                                diff <= 2 && !player.perks.contains(&pk.name.to_string())
                            })
                            .collect();

                        if let Some(perk) = candidates_any.choose(&mut rng) {
                            let name = capitalize_words(&perk.name.to_string());
                            player.perks.push(perk.name.to_string());
                            spawn_toast(
                                &mut commands,
                                &assets,
                                localization.get("toast_study_perk", lang).replace("{perk}", &name),
                                Color::srgba(0.08, 0.10, 0.20, 0.93),
                                Color::srgb(0.35, 0.55, 0.90),
                                Color::srgb(0.75, 0.90, 1.0),
                                toast,
                            );
                        } else {
                            // Secondary Fallback: Increase Max Health
                            player.bonus_max_health += 5;
                            spawn_toast(
                                &mut commands,
                                &assets,
                                localization.get("perk_pool_exhausted", lang),
                                Color::srgba(0.08, 0.10, 0.20, 0.93),
                                Color::srgb(0.35, 0.55, 0.90),
                                Color::srgb(0.75, 0.90, 1.0),
                                toast,
                            );
                        }
                    }
                } else {
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization.get("toast_study_nothing", lang),
                        Color::srgba(0.08, 0.10, 0.20, 0.93),
                        Color::srgb(0.35, 0.55, 0.90),
                        Color::srgb(0.75, 0.90, 1.0),
                        toast,
                    );
                }
            },
            2 => {
                // Conditioning (increase attribute)
                let roll = rng.random_range(0..100);
                if roll < chance {
                    let old_hp = player.max_health();
                    let old_mp = player.max_mana();

                    // Determine how many attributes to increase
                    let count = match slider_val {
                        0 => 1,
                        1 => {
                            if rng.random_bool(0.5) {
                                1
                            } else {
                                2
                            }
                        },
                        2 => {
                            let r = rng.random_range(0..100);
                            if r < 20 {
                                1
                            } else if r < 60 {
                                2
                            } else {
                                3
                            }
                        },
                        _ => 1,
                    };

                    let mut attrs = vec![
                        Attribute::Strength,
                        Attribute::Dexterity,
                        Attribute::Constitution,
                        Attribute::Intelligence,
                        Attribute::Wisdom,
                        Attribute::Charisma,
                    ];
                    attrs.shuffle(&mut rng);

                    let mut increased = Vec::new();
                    for i in 0..(count as usize).min(attrs.len()) {
                        let attr = attrs[i];
                        let attr_name =
                            localization.get(&format!("attribute.{}", attr.to_lowername()), lang);
                        increased.push(attr_name);
                        match attr {
                            Attribute::Strength => player.strength += 1,
                            Attribute::Dexterity => player.dexterity += 1,
                            Attribute::Constitution => player.constitution += 1,
                            Attribute::Intelligence => player.intelligence += 1,
                            Attribute::Wisdom => player.wisdom += 1,
                            Attribute::Charisma => player.charisma += 1,
                        }
                    }

                    player.update_health_mana(old_hp, old_mp);

                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization
                            .get("conditioning_succeeded", lang)
                            .replace("{attrs}", &increased.join(", ")),
                        Color::srgba(0.08, 0.10, 0.20, 0.93),
                        Color::srgb(0.35, 0.55, 0.90),
                        Color::srgb(0.75, 0.90, 1.0),
                        toast,
                    );
                } else {
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization.get("conditioning_failed", lang),
                        Color::srgba(0.08, 0.10, 0.20, 0.93),
                        Color::srgb(0.35, 0.55, 0.90),
                        Color::srgb(0.75, 0.90, 1.0),
                        toast,
                    );
                }
            },
            _ => {},
        }

        play_audio_msg.write(PlayAudioMsg::new("study"));

        if player.ap <= ap_cost {
            trigger_level_up(&mut player, &mut level_up, &mut play_audio_msg, &mut next_game_state);
        } else {
            player.ap -= ap_cost;
        }
    }
}
