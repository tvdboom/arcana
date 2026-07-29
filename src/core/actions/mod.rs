//! Player action definitions, shared progression logic, and action dispatch systems.

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
use crate::core::catalog::catalog::{all_abilities, all_perks};
use crate::core::catalog::equipment::Kind;
use crate::core::classes::Class;
use crate::core::localization::Localization;
use crate::core::menu::buttons::DisabledButton;
use crate::core::player::{Attribute, Player};
use crate::core::settings::Settings;
use crate::core::states::{is_panel_state, GameState};
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::{rng, RngExt};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

const MAX_CATALOG_LEVEL: u32 = 20;
const ENDGAME_MONSTER_MIN_LEVEL: u32 = MAX_CATALOG_LEVEL - 1;

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
    /// Performs the ap cost operation.
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

/// Returns the highest catalog level that can supply an unowned level-up choice.
///
/// At level 20 and beyond, choices start at the final catalog tier and fall back one tier at a
/// time so continued progression never searches for a catalog level that does not exist.
fn level_up_choice_level(
    player_level: u32,
    mut has_unowned_choice_at_level: impl FnMut(u32) -> bool,
) -> Option<u32> {
    if player_level < MAX_CATALOG_LEVEL {
        return Some(player_level);
    }

    (1..=MAX_CATALOG_LEVEL).rev().find(|&level| has_unowned_choice_at_level(level))
}

/// Returns the monster-level range for a hunt or quest combat encounter.
///
/// Endgame characters fight only level-19 or level-20 monsters because the catalog's level cap
/// is 20; lower-level characters retain the existing tier-based encounter ranges.
pub(crate) fn encounter_level_range(player_level: u32, tier: u32) -> (u32, u32) {
    if player_level >= MAX_CATALOG_LEVEL {
        return (ENDGAME_MONSTER_MIN_LEVEL, MAX_CATALOG_LEVEL);
    }

    match tier {
        0 => (player_level.saturating_sub(2).max(1), player_level),
        1 => (player_level.saturating_sub(1).max(1), player_level.saturating_add(1)),
        _ => (player_level, player_level.saturating_add(2)),
    }
}

// Reusable level up helper
/// Performs the trigger level up operation.
pub fn trigger_level_up(
    player: &mut Player,
    level_up: &mut LevelUpPending,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    next_game_state: &mut NextState<GameState>,
) {
    next_game_state.set(GameState::Playing);
    let mut rng = rng();
    player.bonus_max_health += 10;
    player.bonus_max_mana += 10;

    if let Some(pet) = &mut player.pet {
        pet.health += 10;
        pet.max_health += 10;
    }

    let mut ability_choices = Vec::new();
    let ability_choice_level = level_up_choice_level(player.level(), |level| {
        all_abilities().iter().any(|ability| {
            ability.level == level && !player.abilities.contains(&ability.name.to_string())
        })
    });
    let ability_pool: Vec<_> = all_abilities()
        .iter()
        .filter(|ability| {
            Some(ability.level) == ability_choice_level
                && !player.abilities.contains(&ability.name.to_string())
        })
        .collect();

    let mut weighted_pool: Vec<(&Ability, f64)> = ability_pool
        .iter()
        .map(|ab| {
            let mut weight = 1.0;
            let is_magical = ab.kind.is_magic();
            if player.class.is_magical() && is_magical {
                weight *= 2.0;
            }
            if let Class::Mage(ajah) = player.class {
                if ab.kind == ajah.kind() {
                    weight *= 3.0;
                }
            }
            if !player.class.is_magical() && ab.kind == Kind::Physical {
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
    let perk_choice_level = level_up_choice_level(player.level(), |level| {
        all_perks()
            .iter()
            .any(|perk| perk.level == level && !player.perks.contains(&perk.name.to_string()))
    });
    let mut perk_pool: Vec<_> = all_perks()
        .iter()
        .filter(|perk| {
            Some(perk.level) == perk_choice_level && !player.perks.contains(&perk.name.to_string())
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
        new_level: player.level(),
        points_remaining: 2,
        attr_gains: [0; 6],
        ability_choices,
        perk_choices,
        ability_chosen,
        perk_chosen,
    };

    play_audio_msg.write(PlayAudioMsg::new("levelup").volume(-10.));
}

// Reusable XP gain helper that triggers level up.
/// Performs the gain xp operation.
pub fn gain_xp(
    player: &mut Player,
    amount: u32,
    level_up: &mut LevelUpPending,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    next_game_state: &mut NextState<GameState>,
) {
    let old_level = player.level();
    player.xp += amount;
    let new_level = player.level();
    if new_level > old_level {
        trigger_level_up(player, level_up, play_audio_msg, next_game_state);
    }
}

/// Handles playing action clicks.
pub fn handle_playing_action_clicks(
    event: On<Pointer<Click>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    action_btn_q: Query<&ActionButton, Without<DisabledButton>>,
    _game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if let Ok(action_btn) = action_btn_q.get(event.entity) {
        let action = action_btn.0;

        if action != Action::Duel {
            crate::core::network::teardown_duel(&mut commands);
        }

        let current_state = _game_state.get();

        // Toggle behavior: if clicking the button of the action that is currently open, close it.
        let is_currently_open = matches!(
            (action, current_state),
            (Action::Shop, GameState::Shop)
                | (Action::Work, GameState::Work)
                | (Action::Study, GameState::Study)
                | (Action::Train, GameState::Train)
                | (Action::Rest, GameState::Rest)
                | (Action::Craft, GameState::Craft)
                | (Action::Hunt, GameState::Hunt)
                | (Action::Quest, GameState::Quest)
                | (Action::Duel, GameState::Duel)
        );

        if is_currently_open {
            next_game_state.set(GameState::Playing);
            play_audio_msg.write(PlayAudioMsg::new("button"));
            return;
        }

        // Close any open panel before switching to another one.
        if *current_state != GameState::Playing && is_panel_state(*current_state) {
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
                next_game_state.set(GameState::Hunt);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Craft => {
                next_game_state.set(GameState::Craft);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Quest => {
                next_game_state.set(GameState::Quest);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
            Action::Duel => {
                next_game_state.set(GameState::Duel);
                play_audio_msg.write(PlayAudioMsg::new("button"));
            },
        }
    }
}

// System to handle click on work cards
/// Handles work card clicks.
pub fn handle_work_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_q: Query<&work::WorkCardMarker>,
    slider_state: Res<work::WorkSliderState>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
) {
    if let Ok(marker) = card_q.get(event.entity) {
        let slider_val = slider_state.0;
        let ap_cost = slider_val + 1;

        let lang = settings.language;
        let toast = toast_container_q.single().unwrap();

        let values = work::calculate_work_values(&player, slider_val);
        let craft_cost = values.craft_cost;
        let manual_cost = values.manual_cost;

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
            2 if player.health() <= manual_cost => {
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
            },
            _ => {},
        }

        let mut rng = rng();

        match marker.0 {
            0 => {
                // Clerical Labor
                let gold_earned = rng.random_range(values.min_clerical..=values.max_clerical);

                let award_artifact = rng.random_bool(0.5);
                if award_artifact {
                    let matching_artifacts: Vec<_> = crate::core::catalog::catalog::all_artifacts()
                        .iter()
                        .filter(|art| {
                            let name_lower = art.name.to_lowercase();
                            name_lower.contains("scroll")
                                || name_lower.contains("writing")
                                || name_lower.contains("paper")
                                || name_lower.contains("book")
                                || name_lower.contains("bible")
                                || name_lower.contains("page")
                        })
                        .collect();

                    if !matching_artifacts.is_empty() {
                        let mut closest_artifacts = Vec::new();
                        let mut min_diff = i32::MAX;
                        for art in &matching_artifacts {
                            let diff = (art.price as i32 - gold_earned as i32).abs();
                            if diff < min_diff {
                                min_diff = diff;
                                closest_artifacts.clear();
                                closest_artifacts.push(*art);
                            } else if diff == min_diff {
                                closest_artifacts.push(*art);
                            }
                        }
                        let chosen =
                            closest_artifacts[rng.random_range(0..closest_artifacts.len())];
                        player.add_inventory_item(chosen.name.clone());

                        spawn_toast(
                            &mut commands,
                            &assets,
                            format!("Clerical labor done! Earned artifact: {}", chosen.name),
                            Color::srgba(0.08, 0.16, 0.12, 0.93),
                            Color::srgb(0.25, 0.75, 0.50),
                            Color::srgb(0.60, 1.0, 0.75),
                            toast,
                        );
                    } else {
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
                    }
                } else {
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
                }
            },
            1 => {
                // Craft Labor
                let gold_earned = rng.random_range(values.min_craft..=values.max_craft);

                let next_mana = player.mana().saturating_sub(craft_cost);
                player.set_mana(next_mana);

                let award_artifact = rng.random_bool(0.5);
                if award_artifact {
                    let matching_artifacts: Vec<_> = crate::core::catalog::catalog::all_artifacts()
                        .iter()
                        .filter(|art| {
                            let name_lower = art.name.to_lowercase();
                            name_lower.contains("blacksmith")
                                || name_lower.contains("patch")
                                || name_lower.contains("horseshoe")
                                || name_lower.contains("knife")
                                || name_lower.contains("rod")
                                || name_lower.contains("hook")
                                || name_lower.contains("coat")
                                || name_lower.contains("leather")
                                || name_lower.contains("skin")
                                || name_lower.contains("shell")
                                || name_lower.contains("key")
                                || name_lower.contains("candlestick")
                                || name_lower.contains("torch")
                                || name_lower.contains("ingot")
                                || name_lower.contains("bar")
                                || name_lower.contains("needle")
                                || name_lower.contains("thread")
                                || name_lower.contains("cloth")
                        })
                        .collect();

                    if !matching_artifacts.is_empty() {
                        let mut closest_artifacts = Vec::new();
                        let mut min_diff = i32::MAX;
                        for art in &matching_artifacts {
                            let diff = (art.price as i32 - gold_earned as i32).abs();
                            if diff < min_diff {
                                min_diff = diff;
                                closest_artifacts.clear();
                                closest_artifacts.push(*art);
                            } else if diff == min_diff {
                                closest_artifacts.push(*art);
                            }
                        }
                        let chosen =
                            closest_artifacts[rng.random_range(0..closest_artifacts.len())];
                        player.add_inventory_item(chosen.name.clone());

                        spawn_toast(
                            &mut commands,
                            &assets,
                            format!(
                                "Craft labor done! Earned artifact: {} (-{} Mana)",
                                chosen.name, craft_cost
                            ),
                            Color::srgba(0.08, 0.16, 0.12, 0.93),
                            Color::srgb(0.25, 0.75, 0.50),
                            Color::srgb(0.60, 1.0, 0.75),
                            toast,
                        );
                    } else {
                        player.gold += gold_earned;
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
                    }
                } else {
                    player.gold += gold_earned;
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
                }
            },
            2 => {
                // Manual Labor
                let gold_earned = rng.random_range(values.min_manual..=values.max_manual);

                let next_health = player.health().saturating_sub(manual_cost).max(1);
                player.set_health(next_health);

                let award_artifact = rng.random_bool(0.5);
                if award_artifact {
                    let matching_artifacts: Vec<_> = crate::core::catalog::catalog::all_artifacts()
                        .iter()
                        .filter(|art| {
                            let name_lower = art.name.to_lowercase();
                            name_lower.contains("ore")
                                || name_lower.contains("stone")
                                || name_lower.contains("stoune")
                                || name_lower.contains("crystal")
                                || name_lower.contains("diamond")
                                || name_lower.contains("brilliant")
                                || name_lower.contains("pearl")
                                || name_lower.contains("pyrite")
                                || name_lower.contains("coal")
                                || name_lower.contains("clay")
                        })
                        .collect();

                    if !matching_artifacts.is_empty() {
                        let mut closest_artifacts = Vec::new();
                        let mut min_diff = i32::MAX;
                        for art in &matching_artifacts {
                            let diff = (art.price as i32 - gold_earned as i32).abs();
                            if diff < min_diff {
                                min_diff = diff;
                                closest_artifacts.clear();
                                closest_artifacts.push(*art);
                            } else if diff == min_diff {
                                closest_artifacts.push(*art);
                            }
                        }
                        let chosen =
                            closest_artifacts[rng.random_range(0..closest_artifacts.len())];
                        player.add_inventory_item(chosen.name.clone());

                        spawn_toast(
                            &mut commands,
                            &assets,
                            format!(
                                "Manual labor done! Earned artifact: {} (-{} HP)",
                                chosen.name, manual_cost
                            ),
                            Color::srgba(0.08, 0.16, 0.12, 0.93),
                            Color::srgb(0.25, 0.75, 0.50),
                            Color::srgb(0.60, 1.0, 0.75),
                            toast,
                        );
                    } else {
                        player.gold += gold_earned;
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
                    }
                } else {
                    player.gold += gold_earned;
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
                }
            },
            _ => {},
        }

        play_audio_msg.write(PlayAudioMsg::new("work"));

        player.ap += ap_cost;
    }
}

// System to handle click on study cards
/// Handles study card clicks.
pub fn handle_study_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_q: Query<&study::StudyCardMarker>,
    slider_state: Res<study::StudySliderState>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
) {
    if let Ok(marker) = card_q.get(event.entity) {
        let slider_val = slider_state.0;
        let ap_cost = slider_val + 1;

        let lang = settings.language;
        let toast = toast_container_q.single().unwrap();

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

        let target_level = (player.level() as i32 + offset).clamp(1, 20) as u32;

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
                        let name = localization.catalog_name("ability", &ability.name, lang);
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
                                let diff = (ab.level as i32 - player.level() as i32).abs();
                                diff <= 2 && !player.abilities.contains(&ab.name.to_string())
                            })
                            .collect();

                        if let Some(ability) = candidates_any.choose(&mut rng) {
                            let name = localization.catalog_name("ability", &ability.name, lang);
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
                        let name = localization.catalog_name("perk", &perk.name, lang);
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
                                let diff = (pk.level as i32 - player.level() as i32).abs();
                                diff <= 2 && !player.perks.contains(&pk.name.to_string())
                            })
                            .collect();

                        if let Some(perk) = candidates_any.choose(&mut rng) {
                            let name = localization.catalog_name("perk", &perk.name, lang);
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

                    let mut attrs = [
                        Attribute::Strength,
                        Attribute::Dexterity,
                        Attribute::Constitution,
                        Attribute::Intelligence,
                        Attribute::Wisdom,
                        Attribute::Charisma,
                    ];
                    attrs.shuffle(&mut rng);

                    let mut increased = Vec::new();
                    for attr in attrs.iter().take((count as usize).min(attrs.len())) {
                        let attr_name =
                            localization.get(format!("attribute.{}", attr.to_lowername()), lang);
                        increased.push(attr_name);
                        match *attr {
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

        player.ap += ap_cost;
    }
}

#[cfg(test)]
mod tests {
    use super::{encounter_level_range, level_up_choice_level};

    /// Verifies endgame level-ups begin with the final catalog tier and fall back by tier.
    #[test]
    fn endgame_level_up_choices_fall_back_to_the_highest_available_tier() {
        assert_eq!(level_up_choice_level(20, |level| level == 20), Some(20));
        assert_eq!(level_up_choice_level(35, |level| level == 19), Some(19));
        assert_eq!(level_up_choice_level(35, |level| level == 7), Some(7));
    }

    /// Verifies endgame encounters stay within the two highest monster catalog tiers.
    #[test]
    fn endgame_encounters_are_level_nineteen_or_twenty() {
        for tier in 0..=2 {
            assert_eq!(encounter_level_range(20, tier), (19, 20));
            assert_eq!(encounter_level_range(99, tier), (19, 20));
        }
    }
}
