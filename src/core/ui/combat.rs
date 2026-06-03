use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::core::audio::SoundEffect;
use crate::core::compat::MessageWriterSendExt;
use crate::core::localization::Localizer;
use crate::core::player::Character;
use crate::core::states::AppState;
use crate::core::systems::combat_engine::{CombatSession, VictoryState};

use bevy::prelude::MessageWriter as EventWriter;

pub struct CombatUiPlugin;

impl Plugin for CombatUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tick_combat_system.run_if(in_state(AppState::Combat)))
            .add_systems(EguiPrimaryContextPass, draw_combat_ui.run_if(in_state(AppState::Combat)));
    }
}

fn tick_combat_system(
    time: Res<Time>,
    mut combat_session: ResMut<CombatSession>,
    mut sfx_writer: EventWriter<SoundEffect>,
) {
    let delta = time.delta_secs();
    combat_session.tick(delta, &mut sfx_writer);
}

#[allow(clippy::too_many_arguments)]
fn draw_combat_ui(
    mut contexts: EguiContexts,
    mut combat_session: ResMut<CombatSession>,
    mut character: ResMut<Character>,
    mut app_state: ResMut<NextState<AppState>>,
    localizer: Res<Localizer>,
    mut sfx_writer: EventWriter<SoundEffect>,
) {
    let character = &mut *character;
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    crate::core::ui::theme::apply_custom_theme(ctx);

    let Some(player_view) = combat_session.player.clone() else { return };
    let Some(opponent_view) = combat_session.opponent.clone() else { return };
    let player_pet_view = combat_session.player_pet.clone();
    let opponent_pet_view = combat_session.opponent_pet.clone();
    let logs_view = combat_session.logs.clone();
    let victory_state = combat_session.victory_state;
    let is_pvp = combat_session.is_pvp;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(localizer.t("battlefield")).size(24.0).color(egui::Color32::from_rgb(0, 240, 255)));
            ui.add_space(20.0);

            // Row for main combatant blocks
            ui.columns(2, |columns| {
                // Column 0: Player
                columns[0].vertical(|ui| {
                    ui.heading(&player_view.name);
                    ui.label(localizer.format("level_value", &[("level", player_view.level.to_string())]));
                    
                    // Health bar
                    let hp_pct = player_view.hp as f32 / player_view.max_hp as f32;
                    ui.add(egui::ProgressBar::new(hp_pct.max(0.0))
                        .text(localizer.format("hp_bar", &[("current", player_view.hp.to_string()), ("max", player_view.max_hp.to_string())]))
                        .fill(egui::Color32::from_rgb(220, 50, 50)));

                    // Mana bar
                    if player_view.max_mana > 0 {
                        let mp_pct = player_view.mp as f32 / player_view.max_mana as f32;
                        ui.add(egui::ProgressBar::new(mp_pct.max(0.0))
                            .text(localizer.format("mana_bar", &[("current", player_view.mp.to_string()), ("max", player_view.max_mana.to_string())]))
                            .fill(egui::Color32::from_rgb(50, 50, 220)));
                    }

                    // Shields
                    if player_view.shield_value > 0 {
                        ui.label(egui::RichText::new(localizer.format("shield_active", &[("shield", player_view.shield_value.to_string()), ("seconds", format!("{:.1}", player_view.shield_timer))]))
                            .color(egui::Color32::from_rgb(0, 240, 255)));
                    }

                    // Stat highlights
                    ui.label(localizer.format("combat_stats", &[("strength", (player_view.strength as i32 + player_view.strength_bonus).to_string()), ("dexterity", player_view.dexterity.to_string()), ("intelligence", player_view.intelligence.to_string())]));

                    // Player Pet
                    if let Some(ref p_pet) = player_pet_view {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(&p_pet.name).strong());
                        let pet_hp_pct = p_pet.hp as f32 / p_pet.max_hp as f32;
                        ui.add(egui::ProgressBar::new(pet_hp_pct.max(0.0))
                            .text(localizer.format("pet_hp_bar", &[("current", p_pet.hp.to_string()), ("max", p_pet.max_hp.to_string())]))
                            .fill(egui::Color32::from_rgb(180, 50, 150)));
                    }
                });

                // Column 1: Opponent
                columns[1].vertical(|ui| {
                    ui.heading(&opponent_view.name);
                    ui.label(localizer.format("level_value", &[("level", opponent_view.level.to_string())]));

                    // Health bar
                    let opp_hp_pct = opponent_view.hp as f32 / opponent_view.max_hp as f32;
                    ui.add(egui::ProgressBar::new(opp_hp_pct.max(0.0))
                        .text(localizer.format("hp_bar", &[("current", opponent_view.hp.to_string()), ("max", opponent_view.max_hp.to_string())]))
                        .fill(egui::Color32::from_rgb(220, 50, 50)));

                    // Mana bar
                    if opponent_view.max_mana > 0 {
                        let opp_mp_pct = opponent_view.mp as f32 / opponent_view.max_mana as f32;
                        ui.add(egui::ProgressBar::new(opp_mp_pct.max(0.0))
                            .text(localizer.format("mana_bar", &[("current", opponent_view.mp.to_string()), ("max", opponent_view.max_mana.to_string())]))
                            .fill(egui::Color32::from_rgb(50, 50, 220)));
                    }

                    // Shields
                    if opponent_view.shield_value > 0 {
                        ui.label(egui::RichText::new(localizer.format("shield_active", &[("shield", opponent_view.shield_value.to_string()), ("seconds", format!("{:.1}", opponent_view.shield_timer))]))
                            .color(egui::Color32::from_rgb(0, 240, 255)));
                    }

                    // Stat highlights
                    ui.label(localizer.format("combat_stats", &[("strength", (opponent_view.strength as i32 + opponent_view.strength_bonus).to_string()), ("dexterity", opponent_view.dexterity.to_string()), ("intelligence", opponent_view.intelligence.to_string())]));

                    // Opponent Pet
                    if let Some(ref o_pet) = opponent_pet_view {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(&o_pet.name).strong());
                        let opp_pet_hp = o_pet.hp as f32 / o_pet.max_hp as f32;
                        ui.add(egui::ProgressBar::new(opp_pet_hp.max(0.0))
                            .text(localizer.format("pet_hp_bar", &[("current", o_pet.hp.to_string()), ("max", o_pet.max_hp.to_string())]))
                            .fill(egui::Color32::from_rgb(180, 50, 150)));
                    }
                });
            });

            ui.add_space(20.0);

            // Log Console Box
            ui.heading(localizer.t("combat_logs"));
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(10, 14, 20))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 55, 72)))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        ui.set_width(700.0);
                        for log in logs_view.iter().rev().take(15) {
                            ui.label(log);
                        }
                    });
                });

            ui.add_space(20.0);

            // Combat Controls (Abilities / Consumables)
            if victory_state.is_none() {
                ui.heading(localizer.t("combat_actions"));
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    // Spells
                    for (idx, state) in player_view.abilities.iter().enumerate() {
                        let btn_txt = if state.cooldown_timer > 0.0 {
                            format!("{} ({:.1}s)", state.ability.name, state.cooldown_timer)
                        } else {
                            format!("{} ({} MP)", state.ability.name, state.ability.mana_cost)
                        };

                        let enabled = state.cooldown_timer <= 0.0 && player_view.mp >= state.ability.mana_cost as i32;
                        if ui.add_enabled(enabled, egui::Button::new(btn_txt)).clicked() {
                            let _ = combat_session.cast_ability(true, idx, &mut sfx_writer);
                        }
                    }

                    ui.add_space(30.0);

                    // Potions
                    for (idx, state) in player_view.consumables.iter().enumerate() {
                        let btn_txt = if state.used {
                            localizer.format("consumed", &[("name", state.item.name.clone())])
                        } else {
                            state.item.name.clone()
                        };

                        if ui.add_enabled(!state.used, egui::Button::new(btn_txt)).clicked() {
                            let _ = combat_session.use_consumable(true, idx, &mut sfx_writer);
                        }
                    }
                });
            }
        });
    });

    // Victory/Defeat screen overlay
    if let Some(state) = victory_state {
        egui::Window::new(localizer.t("combat_resolved"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let title = match state {
                        VictoryState::PlayerWins => {
                            egui::RichText::new(localizer.t("victory")).size(30.0).color(egui::Color32::from_rgb(0, 240, 255)).strong()
                        }
                        VictoryState::OpponentWins => {
                            egui::RichText::new(localizer.t("defeat")).size(30.0).color(egui::Color32::from_rgb(220, 50, 50)).strong()
                        }
                    };
                    ui.label(title);
                    ui.add_space(20.0);

                    if ui.button(localizer.t("return_to_town")).clicked() {
                        sfx_writer.send(SoundEffect::Click);
                        
                        // Apply results to single player character
                        character.current_health = player_view.hp.max(10); // leave with 10 HP if died
                        character.current_mana = player_view.mp.max(0);

                        // If player won hunt, reward gold and XP
                        if !is_pvp && state == VictoryState::PlayerWins {
                            let gold_loot = opponent_view.level * 25;
                            let xp_loot = opponent_view.level * 30;
                            character.gold += gold_loot;
                            character.xp += xp_loot;

                            // Check level up (handled during Resolve Phase, but let's give notice)
                            println!("Looted {} gold and gained {} XP.", gold_loot, xp_loot);
                        }

                        // Close session and redirect
                        combat_session.active = false;
                        app_state.set(AppState::Planning);
                    }
                });
            });
    }
}
