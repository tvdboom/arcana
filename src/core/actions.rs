use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::{all_abilities, all_equipment, all_perks};
use crate::core::inventory::armor::EquipmentSlot;
use crate::core::inventory::equipment::Equipment;
use crate::core::localization::Localization;
use crate::core::menu::buttons::DisabledButton;
use crate::core::player::Player;
use crate::core::settings::Settings;
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::playing::{capitalize_words, reward_equipment};
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
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
}

impl Action {
    pub fn gold_cost(&self) -> u32 {
        match self {
            Action::Craft => 15,
            Action::Shop => 30,
            _ => 0,
        }
    }

    pub fn ap_cost(&self) -> u32 {
        match self {
            Action::Shop => 0,
            Action::Rest | Action::Study | Action::Work => 1,
            Action::Craft | Action::Train | Action::Hunt => 2,
            Action::Quest => 3,
        }
    }
}

#[derive(Component)]
pub struct ActionButton(pub Action);

pub fn handle_playing_action_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    action_btn_q: Query<&ActionButton, Without<DisabledButton>>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
) {
    if let Ok(action_btn) = action_btn_q.get(event.entity) {
        let action = action_btn.0;
        let lang = settings.language;

        let toast = toast_container_q.single().unwrap();

        // Play action sound (work/study/rest/train use their own sound, others use generic button)
        if matches!(action, Action::Work | Action::Study | Action::Rest | Action::Train) {
            play_audio_msg.write(PlayAudioMsg::new(action.to_lowername()));
        } else {
            play_audio_msg.write(PlayAudioMsg::new("button"));
        }

        player.gold -= action.gold_cost();

        let mut rng = rng();

        let max_hp = player.max_health();
        let max_mp = player.max_mana();

        match action {
            Action::Rest => {
                let msg = if player.health < max_hp || player.mana < max_mp {
                    // Recover health and mana if below maximum
                    // The recovered amount lies between 0 and 10% + 1% per wisdom modifier
                    let max_recover_health = (player.max_health() as f32 * 0.1
                        + 0.01 * player.wisdom_mod() as f32)
                        as u32;
                    let max_recover_mana =
                        (player.max_mana() as f32 * 0.1 + 0.01 * player.wisdom_mod() as f32) as u32;

                    let recover_health = rng.random_range(0..max_recover_health);
                    let recover_mana = rng.random_range(0..max_recover_mana);

                    let health_gained = recover_health.min(max_hp - player.health);
                    let mana_gained = recover_mana.min(max_mp - player.mana);

                    player.health = (player.health + health_gained).min(max_hp);
                    player.mana = (player.mana + mana_gained).min(max_mp);

                    localization
                        .get("toast_rest_recovered", lang)
                        .replace("{hp}", &recover_health.to_string())
                        .replace("{mp}", &recover_mana.to_string())
                } else {
                    // Gain increased max health or max mana
                    if rng.random_bool(0.25) {
                        let bonus_health = rng.random_range(1..=5) * 5 * (1 + player.wisdom_mod());
                        player.bonus_max_health += bonus_health;

                        localization
                            .get("toast_rest_max_hp", lang)
                            .replace("{hp}", &bonus_health.to_string())
                    } else if rng.random_bool(0.25) {
                        let bonus_mana = rng.random_range(1..=5) * 5 * (1 + player.wisdom_mod());
                        player.bonus_max_mana += bonus_mana;

                        localization
                            .get("toast_rest_max_mp", lang)
                            .replace("{mp}", &bonus_mana.to_string())
                    } else {
                        localization.get("toast_rest_no_recovery", lang)
                    }
                };

                spawn_toast(
                    &mut commands,
                    &assets,
                    msg,
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    toast,
                );
            },
            Action::Study => {
                let int_bonus = (player.intelligence() as f32 - 10.).max(0.) * 0.025;
                let perk_chance = (0.333 + int_bonus).min(0.65) as f64;
                let ability_chance = (0.200 + int_bonus).min(0.45) as f64;
                let wisdom_chance = 0.05_f64;

                // Weighted level selection: 50% current, 25% lower, 25% higher
                let level_offset: i8 = match rng.random_range(0u8..4) {
                    0 => 1,
                    1 | 2 => 0,
                    _ => -1,
                };
                let target_level = (player.level as i8 + level_offset).clamp(1, 20) as u8;

                let mut toast_msg = localization.get("toast_study_nothing", lang);

                // Roll for perk first (higher chance)
                if rng.random_bool(perk_chance) {
                    let candidates: Vec<_> = all_perks()
                        .iter()
                        .filter(|pk| {
                            (pk.level == target_level as u32 || pk.level == player.level as u32)
                                && !player.perks.contains(&pk.name.to_string())
                        })
                        .collect();
                    if let Some(perk) = candidates.choose(&mut rng) {
                        let name = capitalize_words(&perk.name.to_string());
                        player.perks.push(perk.name.to_string());
                        toast_msg =
                            localization.get("toast_study_perk", lang).replace("{perk}", &name);
                    }
                // Otherwise roll for ability (lower chance)
                } else if rng.random_bool(ability_chance) {
                    let candidates: Vec<_> = all_abilities()
                        .iter()
                        .filter(|ab| {
                            (ab.level == target_level as u32 || ab.level == player.level as u32)
                                && !player.abilities.contains(&ab.name.to_string())
                        })
                        .collect();
                    if let Some(ability) = candidates.choose(&mut rng) {
                        let name = capitalize_words(&ability.name.to_string());
                        player.abilities.push(ability.name.to_string());
                        toast_msg = localization
                            .get("toast_study_ability", lang)
                            .replace("{ability}", &name);
                    }
                }

                // Rare wisdom bonus (independent)
                if rng.random_bool(wisdom_chance) {
                    player.wisdom += 1;
                    if toast_msg == localization.get("toast_study_nothing", lang) {
                        toast_msg = localization.get("toast_study_wisdom", lang);
                    } else {
                        toast_msg = format!(
                            "{}  {}",
                            toast_msg,
                            localization.get("toast_study_wisdom", lang)
                        );
                    }
                }

                spawn_toast(
                    &mut commands,
                    &assets,
                    toast_msg,
                    Color::srgba(0.08, 0.10, 0.20, 0.93),
                    Color::srgb(0.35, 0.55, 0.90),
                    Color::srgb(0.75, 0.90, 1.0),
                    toast,
                );
            },
            Action::Hunt => {
                let gold_earned = rng.random_range(10..=20);
                player.gold += gold_earned;
            },
            Action::Work => {
                let charisma = player.charisma() as i32;
                let level = player.level as i32;
                let base = charisma * level;
                let min_gold = (base * 4 / 5).max(1) as u32;
                let max_gold = (base * 6 / 5).max(2) as u32;
                let gold_earned = rng.random_range(min_gold..=max_gold);
                player.gold += gold_earned;
                spawn_toast(
                    &mut commands,
                    &assets,
                    localization
                        .get("toast_gold_earned", lang)
                        .replace("{gold}", &gold_earned.to_string()),
                    Color::srgba(0.18, 0.13, 0.02, 0.93),
                    Color::srgb(0.85, 0.65, 0.15),
                    Color::srgb(1.0, 0.88, 0.30),
                    toast,
                );
            },
            Action::Shop => {
                let lvl = player.level;
                let items: Vec<_> = all_equipment()
                    .iter()
                    .filter(|eq| match eq {
                        Equipment::Armor(a) => {
                            a.slot == EquipmentSlot::Consumable && a.level <= lvl as u32
                        },
                        _ => false,
                    })
                    .collect();

                if let Some(item) = items.choose(&mut rng) {
                    let name = item.name().to_string();
                    reward_equipment(&mut player, name);
                }
            },
            Action::Quest => {
                let gold_earned = rng.random_range(20..=40);
                if rng.random_bool(0.5) {
                    let items: Vec<_> = all_equipment()
                        .iter()
                        .filter(|eq| eq.level() <= player.level as u32)
                        .collect();

                    if let Some(item) = items.choose(&mut rng) {
                        reward_equipment(&mut player, item.name().to_string());
                    }
                }
                player.gold += gold_earned;
            },
            Action::Train => {
                // Training costs health and mana (10-30% of max)
                let hp_cost = rng.random_range(max_hp as f32 * 0.1..=max_hp as f32 * 0.3);
                let mp_cost = rng.random_range(max_mp as f32 * 0.1..=max_mp as f32 * 0.3);

                player.health = (player.health - hp_cost as u32).max(1);
                player.mana = (player.mana - mp_cost as u32).max(0);

                let str_bonus = player.strength() as f64;
                let dex_bonus = player.dexterity() as f64;
                let combined = str_bonus + dex_bonus;

                // Base 30% chance of something happening, +1% per combined str+dex point
                let success_chance = (30.0 + combined) / 100.0;

                let toast_msg = if rng.random_bool(success_chance) {
                    // Something good happened
                    let roll = rng.random_range(0..100);

                    if roll < 25 {
                        // Learn a new ability (if available)
                        let ability_pool: Vec<_> = all_abilities()
                            .iter()
                            .filter(|ab| {
                                ab.level <= player.level as u32
                                    && !player.abilities.contains(&ab.name.to_string())
                            })
                            .collect();

                        if !ability_pool.is_empty() {
                            if let Some(ability) = ability_pool.choose(&mut rng) {
                                player.abilities.push(ability.name.to_string());
                                localization
                                    .get("toast_train_learned_ability", lang)
                                    .replace("{ability}", &ability.name)
                            } else {
                                localization.get("toast_train_nothing_new", lang)
                            }
                        } else {
                            // No abilities available, increase max health instead
                            player.bonus_max_health += 5;
                            localization.get("toast_train_plus_5_hp", lang)
                        }
                    } else if roll < 50 {
                        // Increase strength
                        player.strength += 1;
                        localization.get("toast_train_plus_str", lang)
                    } else if roll < 75 {
                        // Increase dexterity
                        player.dexterity += 1;
                        localization.get("toast_train_plus_dex", lang)
                    } else {
                        // Increase max health (represents attack training)
                        player.bonus_max_health += 10;
                        localization.get("toast_train_plus_10_hp", lang)
                    }
                } else {
                    localization.get("toast_train_no_improvement", lang)
                };

                spawn_toast(
                    &mut commands,
                    &assets,
                    toast_msg,
                    Color::srgba(0.08, 0.10, 0.20, 0.93),
                    Color::srgb(0.90, 0.55, 0.35),
                    Color::srgb(1.0, 0.90, 0.75),
                    toast,
                );
            },
            Action::Craft => {
                let items: Vec<_> =
                    all_equipment().iter().filter(|eq| eq.level() == player.level as u32).collect();

                if let Some(item) = items.choose(&mut rng) {
                    reward_equipment(&mut player, item.name().to_string());
                }
            },
        };

        // Deduct action points
        if player.ap <= action.ap_cost() {
            let old_max_health = player.max_health();
            let old_max_mana = player.max_mana();

            player.level += 1;
            player.ap = 10;
            // No automatic attribute increases - player chooses 2 points
            // Bonus health/mana increase
            player.bonus_max_health += 10;
            player.bonus_max_mana += 10;

            let health_diff = player.max_health() - old_max_health;
            let mana_diff = player.max_mana() - old_max_mana;

            player.health = (player.health + health_diff).min(player.max_health());
            player.mana = (player.mana + mana_diff).min(player.max_mana());

            // Generate ability and perk choices for the new level
            let new_level = player.level;

            let mut ability_choices = Vec::new();
            let mut ability_pool: Vec<_> = all_abilities()
                .iter()
                .filter(|ab| {
                    ab.level == new_level as u32 && !player.abilities.contains(&ab.name.to_string())
                })
                .collect();
            for _ in 0..3 {
                if ability_pool.is_empty() {
                    break;
                }
                let idx = rng.random_range(0..ability_pool.len());
                ability_choices.push(ability_pool[idx].name.to_string());
                ability_pool.remove(idx);
            }

            let mut perk_choices = Vec::new();
            let mut perk_pool: Vec<_> = all_perks()
                .iter()
                .filter(|pk| {
                    pk.level == new_level as u32 && !player.perks.contains(&pk.name.to_string())
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
                new_level,
                points_remaining: 2,
                attr_gains: [0; 6],
                ability_choices,
                perk_choices,
                ability_chosen,
                perk_chosen,
            };

            play_audio_msg.write(PlayAudioMsg::new("levelup").volume(-10.));
        } else {
            player.ap -= action.ap_cost();
        }
    }
}
