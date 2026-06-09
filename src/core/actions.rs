use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
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
use rand::RngExt;
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
            Action::Rest | Action::Study | Action::Work => 1,
            Action::Craft | Action::Train | Action::Hunt | Action::Shop => 2,
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
        let cost_gold = action.gold_cost();
        let lang = settings.language;

        let container_entity = toast_container_q.single().ok();

        if player.gold < cost_gold {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            return;
        }

        // Play action sound (work/study/rest/train use their own sound, others use generic button)
        if matches!(action, Action::Work | Action::Study | Action::Rest | Action::Train) {
            play_audio_msg.write(PlayAudioMsg::new(action.to_lowername()));
        } else {
            play_audio_msg.write(PlayAudioMsg::new("button"));
        }
        player.gold -= cost_gold;

        // Handle the specific action
        let ap_cost = match action {
            Action::Hunt => {
                let gold_earned = rand::rng().random_range(10..=20);
                player.gold += gold_earned;
                2
            },
            Action::Work => {
                let charisma = player.charisma() as i32;
                let level = player.level as i32;
                let base = charisma * level;
                let min_gold = (base * 4 / 5).max(1) as u32;
                let max_gold = (base * 6 / 5).max(2) as u32;
                let gold_earned = rand::rng().random_range(min_gold..=max_gold);
                player.gold += gold_earned;
                if let Some(container) = container_entity {
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization
                            .get("toast_gold_earned", lang)
                            .replace("{gold}", &gold_earned.to_string()),
                        Color::srgba(0.18, 0.13, 0.02, 0.93),
                        Color::srgb(0.85, 0.65, 0.15),
                        Color::srgb(1.0, 0.88, 0.30),
                        container,
                    );
                }
                1
            },
            Action::Shop => {
                let lvl = player.level;
                let items: Vec<&crate::core::catalog::GeneratedEquipment> =
                    crate::core::catalog::GENERATED_EQUIPMENT
                        .iter()
                        .filter(|eq| eq.kind == "consumable" && eq.level <= lvl)
                        .collect();
                use rand::seq::IndexedRandom;
                if let Some(item) = items.choose(&mut rand::rng()) {
                    let name = item.name.to_string();
                    reward_equipment(&mut player, name);
                }
                0
            },
            Action::Quest => {
                let gold_earned = rand::rng().random_range(20..=40);
                if rand::rng().random_bool(0.5) {
                    let items: Vec<&crate::core::catalog::GeneratedEquipment> =
                        crate::core::catalog::GENERATED_EQUIPMENT
                            .iter()
                            .filter(|eq| eq.level <= player.level)
                            .collect();
                    use rand::seq::IndexedRandom;
                    if let Some(item) = items.choose(&mut rand::rng()) {
                        reward_equipment(&mut player, item.name.to_string());
                    }
                }
                player.gold += gold_earned;
                3
            },
            Action::Train => {
                // Training costs health and mana (10-30% of max)
                let max_hp = player.max_health();
                let max_mp = player.max_mana();
                let hp_cost = rand::rng().random_range(max_hp * 0.1..=max_hp * 0.3);
                let mp_cost = rand::rng().random_range(max_mp * 0.1..=max_mp * 0.3);

                player.health = (player.health - hp_cost).max(1.0);
                player.mana = (player.mana - mp_cost).max(0.0);

                let str_bonus = player.strength() as f64;
                let dex_bonus = player.dexterity() as f64;
                let combined = str_bonus + dex_bonus;

                // Base 30% chance of something happening, +1% per combined str+dex point
                let success_chance = (30.0 + combined) / 100.0;

                let toast_msg = if rand::rng().random_bool(success_chance) {
                    // Something good happened
                    let roll = rand::rng().random_range(0..100);

                    if roll < 25 {
                        // Learn a new ability (if available)
                        let class_hint = player.class.to_lowername();
                        let ability_pool: Vec<_> = crate::core::catalog::GENERATED_ABILITIES
                            .iter()
                            .filter(|ab| {
                                ab.level <= player.level
                                    && ab.class_hint == class_hint
                                    && !player.abilities.contains(&ab.name.to_string())
                            })
                            .collect();

                        if !ability_pool.is_empty() {
                            if let Some(ability) = ability_pool.choose(&mut rand::rng()) {
                                player.abilities.push(ability.name.to_string());
                                localization
                                    .get("toast_train_learned_ability", lang)
                                    .replace("{ability}", &ability.name)
                            } else {
                                localization.get("toast_train_nothing_new", lang)
                            }
                        } else {
                            // No abilities available, increase max health instead
                            player.bonus_max_health += 5.0;
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
                        player.bonus_max_health += 10.0;
                        localization.get("toast_train_plus_10_hp", lang)
                    }
                } else {
                    localization.get("toast_train_no_improvement", lang)
                };

                if let Some(container) = container_entity {
                    spawn_toast(
                        &mut commands,
                        &assets,
                        toast_msg,
                        Color::srgba(0.08, 0.10, 0.20, 0.93),
                        Color::srgb(0.90, 0.55, 0.35),
                        Color::srgb(1.0, 0.90, 0.75),
                        container,
                    );
                }
                2
            },
            Action::Craft => {
                let items: Vec<&crate::core::catalog::GeneratedEquipment> =
                    crate::core::catalog::GENERATED_EQUIPMENT
                        .iter()
                        .filter(|eq| eq.level == player.level)
                        .collect();
                use rand::seq::IndexedRandom;
                if let Some(item) = items.choose(&mut rand::rng()) {
                    reward_equipment(&mut player, item.name.to_string());
                }
                2
            },
            Action::Rest => {
                let wisdom = player.wisdom() as i32;
                let level = player.level as i32;
                let base = wisdom * level;
                let min_recover = (base * 4 / 5).max(1) as f32;
                let max_recover = (base * 6 / 5).max(2) as f32;
                let recover_amount = rand::rng().random_range(min_recover..=max_recover);

                let max_hp = player.max_health().floor();
                let max_mp = player.max_mana().floor();
                let health_before = player.health;
                let mana_before = player.mana;
                player.health = (player.health + recover_amount).min(max_hp);
                player.mana = (player.mana + recover_amount).min(max_mp);
                let health_gained = (player.health - health_before).round() as i32;
                let mana_gained = (player.mana - mana_before).round() as i32;

                let mut pet_gained = 0;
                let mut pet_name = String::new();
                if let Some(ref mut pet) = player.pet {
                    pet_name = pet.name.clone();
                    let pet_max_hp = pet.max_health as f32;
                    let pet_health_before = pet.health as f32;
                    let new_pet_health = (pet_health_before + recover_amount).min(pet_max_hp);
                    pet.health = new_pet_health.round() as i32;
                    pet_gained = (new_pet_health - pet_health_before).round() as i32;
                }

                // Small chance of permanently increasing max health / max mana
                let wisdom_bonus = (player.wisdom() as f32 - 10.).max(0.) * 0.005;
                let max_chance = (0.05 + wisdom_bonus).min(0.20) as f64;
                let mut bonus_lines = Vec::new();
                if pet_gained > 0 {
                    bonus_lines.push(
                        localization
                            .get("toast_rest_pet", lang)
                            .replace("{pet}", &pet_name)
                            .replace("{gain}", &pet_gained.to_string()),
                    );
                }
                if rand::rng().random_bool(max_chance) {
                    let gain = rand::rng().random_range(2.0_f32..=5.0_f32).round();
                    player.bonus_max_health += gain;
                    player.health = (player.health + gain).min(player.max_health().floor());
                    bonus_lines.push(
                        localization
                            .get("toast_rest_max_hp", lang)
                            .replace("{gain}", &(gain as i32).to_string()),
                    );
                }
                if rand::rng().random_bool(max_chance) {
                    let gain = rand::rng().random_range(2.0_f32..=5.0_f32).round();
                    player.bonus_max_mana += gain;
                    player.mana = (player.mana + gain).min(player.max_mana().floor());
                    bonus_lines.push(
                        localization
                            .get("toast_rest_max_mp", lang)
                            .replace("{gain}", &(gain as i32).to_string()),
                    );
                }

                let mut toast_parts = vec![localization
                    .get("toast_rest_recovered", lang)
                    .replace("{hp}", &health_gained.to_string())
                    .replace("{mp}", &mana_gained.to_string())];
                toast_parts.extend(bonus_lines);
                let toast_msg = toast_parts.join("  ");
                if let Some(container) = container_entity {
                    spawn_toast(
                        &mut commands,
                        &assets,
                        toast_msg,
                        Color::srgba(0.08, 0.16, 0.12, 0.93),
                        Color::srgb(0.25, 0.75, 0.50),
                        Color::srgb(0.60, 1.0, 0.75),
                        container,
                    );
                }
                1
            },
            Action::Study => {
                use rand::seq::IndexedRandom;
                let int_bonus = (player.intelligence() as f32 - 10.).max(0.) * 0.025;
                let perk_chance = (0.333 + int_bonus).min(0.65) as f64;
                let ability_chance = (0.200 + int_bonus).min(0.45) as f64;
                let wisdom_chance = 0.05_f64;
                let class_hint = player.class.to_lowername();

                // Weighted level selection: 50% current, 25% lower, 25% higher
                let level_offset: i8 = match rand::rng().random_range(0u8..4) {
                    0 => 1,
                    1 | 2 => 0,
                    _ => -1,
                };
                let target_level = (player.level as i8 + level_offset).clamp(1, 20) as u8;

                let mut toast_msg = localization.get("toast_study_nothing", lang);

                // Roll for perk first (higher chance)
                if rand::rng().random_bool(perk_chance) {
                    let candidates: Vec<&crate::core::catalog::GeneratedPerk> =
                        crate::core::catalog::GENERATED_PERKS
                            .iter()
                            .filter(|pk| {
                                (pk.level == target_level || pk.level == player.level)
                                    && pk.class_hint == class_hint
                                    && !player.perks.contains(&pk.name.to_string())
                            })
                            .collect();
                    if let Some(perk) = candidates.choose(&mut rand::rng()) {
                        let name = capitalize_words(&perk.name.to_string());
                        player.perks.push(perk.name.to_string());
                        toast_msg =
                            localization.get("toast_study_perk", lang).replace("{perk}", &name);
                    }
                // Otherwise roll for ability (lower chance)
                } else if rand::rng().random_bool(ability_chance) {
                    let candidates: Vec<&crate::core::catalog::GeneratedAbility> =
                        crate::core::catalog::GENERATED_ABILITIES
                            .iter()
                            .filter(|ab| {
                                (ab.level == target_level || ab.level == player.level)
                                    && ab.class_hint == class_hint
                                    && !player.abilities.contains(&ab.name.to_string())
                            })
                            .collect();
                    if let Some(ability) = candidates.choose(&mut rand::rng()) {
                        let name = capitalize_words(&ability.name.to_string());
                        player.abilities.push(ability.name.to_string());
                        toast_msg = localization
                            .get("toast_study_ability", lang)
                            .replace("{ability}", &name);
                    }
                }

                // Rare wisdom bonus (independent)
                if rand::rng().random_bool(wisdom_chance) {
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

                if let Some(container) = container_entity {
                    spawn_toast(
                        &mut commands,
                        &assets,
                        toast_msg,
                        Color::srgba(0.08, 0.10, 0.20, 0.93),
                        Color::srgb(0.35, 0.55, 0.90),
                        Color::srgb(0.75, 0.90, 1.0),
                        container,
                    );
                }
                1
            },
        };

        // Deduct action points
        if player.ap <= ap_cost {
            let old_max_health = player.max_health();
            let old_max_mana = player.max_mana();

            player.level += 1;
            player.ap = 10 + (player.level as u32) * 2;
            // No automatic attribute increases - player chooses 2 points
            // Bonus health/mana increase
            player.bonus_max_health += 10.;
            player.bonus_max_mana += 10.;

            let health_diff = player.max_health() - old_max_health;
            let mana_diff = player.max_mana() - old_max_mana;

            player.health = (player.health + health_diff).min(player.max_health().floor());
            player.mana = (player.mana + mana_diff).min(player.max_mana().floor());

            // Generate ability and perk choices for the new level
            let class_hint = player.class.to_lowername();
            let new_level = player.level;

            let mut ability_choices = Vec::new();
            let mut ability_pool: Vec<_> = crate::core::catalog::GENERATED_ABILITIES
                .iter()
                .filter(|ab| {
                    ab.level == new_level
                        && ab.class_hint == class_hint
                        && !player.abilities.contains(&ab.name.to_string())
                })
                .collect();
            for _ in 0..3 {
                if ability_pool.is_empty() {
                    break;
                }
                let idx = rand::rng().random_range(0..ability_pool.len());
                ability_choices.push(ability_pool[idx].name.to_string());
                ability_pool.remove(idx);
            }

            let mut perk_choices = Vec::new();
            let mut perk_pool: Vec<_> = crate::core::catalog::GENERATED_PERKS
                .iter()
                .filter(|pk| {
                    pk.level == new_level
                        && pk.class_hint == class_hint
                        && !player.perks.contains(&pk.name.to_string())
                })
                .collect();
            for _ in 0..3 {
                if perk_pool.is_empty() {
                    break;
                }
                let idx = rand::rng().random_range(0..perk_pool.len());
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
            player.ap -= ap_cost;
        }
    }
}
