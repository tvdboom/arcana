use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use bevy_renet::{RenetClient, RenetServer};
use bevy_renet::renet::DefaultChannel;

use crate::core::audio::SoundEffect;
use crate::core::compat::MessageWriterSendExt;
use crate::core::localization::Localizer;
use crate::core::network::{connect_client, get_local_ip, host_server, NetworkManager};
use crate::core::player::Character;
use crate::core::states::AppState;

use bevy::prelude::MessageWriter as EventWriter;

pub struct DuelsUiPlugin;

impl Plugin for DuelsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, draw_duels_lobby.run_if(in_state(AppState::PvPLobby)));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_duels_lobby(
    mut contexts: EguiContexts,
    mut net_manager: ResMut<NetworkManager>,
    mut character: ResMut<Character>,
    mut app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mut sfx_writer: EventWriter<SoundEffect>,
    localizer: Res<Localizer>,
    server: Option<Res<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
    mut host_ip_input: Local<String>,
    mut level_cap_input: Local<u32>,
    mut wager_gold_input: Local<u32>,
    mut has_introduced: Local<bool>,
) {
    let character = &mut *character;
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    crate::core::ui::theme::apply_custom_theme(ctx);

    // Initial default setting values
    if *level_cap_input == 0 {
        *level_cap_input = character.level;
    }
    if host_ip_input.is_empty() {
        *host_ip_input = "127.0.0.1".to_string();
    }

    egui::Window::new(localizer.t("pvp_chamber"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .fixed_size(egui::vec2(440.0, 500.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(localizer.t("online_arena")).size(18.0).color(egui::Color32::from_rgb(0, 240, 255)));
                ui.add_space(15.0);

                if server.is_none() && client.is_none() {
                    // Not connected or hosting
                    ui.label(localizer.t("host_or_join_duel"));
                    ui.add_space(15.0);

                    // Host Controls
                    ui.heading(localizer.t("host_match"));
                    let local_ip = get_local_ip();
                    ui.label(localizer.format("local_ip", &[("ip", local_ip.clone())]));
                    if ui.button(localizer.t("host_duel_server")).clicked() {
                        sfx_writer.send(SoundEffect::Click);
                        if let Err(e) = host_server(&mut commands) {
                            println!("Hosting failed: {}", e);
                        } else {
                            net_manager.is_host = true;
                            net_manager.ip_address = local_ip;
                        }
                    }
                    ui.add_space(20.0);

                    // Client Controls
                    ui.heading(localizer.t("join_match"));
                    ui.horizontal(|ui| {
                        ui.label(localizer.t("host_ip_address"));
                        ui.text_edit_singleline(&mut *host_ip_input);
                    });
                    if ui.button(localizer.t("connect_to_host")).clicked() {
                        sfx_writer.send(SoundEffect::Click);
                        if let Err(e) = connect_client(&mut commands, &host_ip_input) {
                            println!("Connection failed: {}", e);
                        } else {
                            net_manager.is_host = false;
                        }
                    }

                } else {
                    // Connected/hosting lobby view
                    if net_manager.is_host {
                        ui.label(egui::RichText::new(localizer.t("server_host")).color(egui::Color32::from_rgb(160, 32, 240)));
                        ui.label(localizer.t("waiting_for_opponent"));
                    } else {
                        ui.label(egui::RichText::new(localizer.t("client_context")).color(egui::Color32::from_rgb(0, 240, 255)));
                        ui.label(localizer.t("joined_lobby"));
                    }
                    ui.add_space(15.0);

                    // Show players Info
                    ui.heading(localizer.t("fighters_in_room"));
                    ui.label(localizer.format("me_level", &[("name", character.name.clone()), ("level", character.level.to_string())]));
                    if let Some(ref opp) = net_manager.opponent_character {
                        ui.label(egui::RichText::new(localizer.format("opponent_level", &[("name", opp.name.clone()), ("level", opp.level.to_string())]))
                            .color(egui::Color32::from_rgb(0, 240, 255)));
                    } else {
                        ui.label(localizer.t("opponent_waiting"));
                    }
                    ui.add_space(20.0);

                    // Configuration Controls (Host manages, Client syncs)
                    if net_manager.is_host {
                        ui.heading(localizer.t("configure_match"));
                        
                        // Level Cap slider
                        ui.horizontal(|ui| {
                            ui.label(localizer.t("level_cap_label"));
                            ui.add(egui::Slider::new(&mut *level_cap_input, 1..=20));
                        });

                        // Wager Gold Input
                        ui.horizontal(|ui| {
                            ui.label(localizer.t("gold_stakes_wager"));
                            ui.add(egui::DragValue::new(&mut *wager_gold_input));
                        });

                        if ui.button(localizer.t("sync_settings")).clicked() {
                            sfx_writer.send(SoundEffect::Click);
                            net_manager.level_cap = *level_cap_input;
                            net_manager.wager_gold = *wager_gold_input;

                            if let Some(ref mut c) = client {
                                let msg = crate::core::network::ClientMessage::UpdateWager {
                                    wager_gold: net_manager.wager_gold,
                                    level_cap: net_manager.level_cap,
                                };
                                if let Ok(bytes) = postcard::to_stdvec(&msg) {
                                    c.send_message(DefaultChannel::ReliableOrdered, bytes);
                                }
                            }
                        }
                    } else {
                        ui.heading(localizer.t("match_conditions"));
                        ui.label(localizer.format("agreed_level_cap", &[("level", net_manager.level_cap.to_string())]));
                        ui.label(localizer.format("gold_stakes_value", &[("gold", net_manager.wager_gold.to_string())]));
                    }
                    ui.add_space(20.0);

                    // Ready triggers
                    ui.horizontal(|ui| {
                        let ready_label = if net_manager.my_ready { localizer.t("cancel_ready") } else { localizer.t("mark_ready") };
                        if ui.button(ready_label).clicked() {
                            sfx_writer.send(SoundEffect::Click);
                            net_manager.my_ready = !net_manager.my_ready;

                            if net_manager.my_ready {
                                if let Some(ref mut c) = client {
                                    let msg = crate::core::network::ClientMessage::Ready;
                                    if let Ok(bytes) = postcard::to_stdvec(&msg) {
                                        c.send_message(DefaultChannel::ReliableOrdered, bytes);
                                    }
                                }
                            }
                        }
                        ui.add_space(20.0);
                        let opp_ready_txt = if net_manager.opponent_ready { localizer.t("opponent_ready") } else { localizer.t("opponent_not_ready") };
                        let opp_ready_col = if net_manager.opponent_ready { egui::Color32::from_rgb(0, 240, 255) } else { egui::Color32::GRAY };
                        ui.label(egui::RichText::new(opp_ready_txt).color(opp_ready_col).strong());
                    });
                }

                // Exit Lobby
                ui.add_space(30.0);
                if ui.button(localizer.t("leave_lobby")).clicked() {
                    sfx_writer.send(SoundEffect::Click);
                    // Despawn network resources
                    commands.remove_resource::<RenetClient>();
                    commands.remove_resource::<RenetServer>();
                    net_manager.opponent_character = None;
                    net_manager.opponent_ready = false;
                    net_manager.my_ready = false;
                    *has_introduced = false;
                    app_state.set(AppState::Planning);
                }
            });
        });

    if !*has_introduced {
        if let Some(ref mut c) = client {
            let introduce_msg = crate::core::network::ClientMessage::Introduce {
                character: Box::new(character.clone()),
            };
            if let Ok(bytes) = postcard::to_stdvec(&introduce_msg) {
                c.send_message(DefaultChannel::ReliableOrdered, bytes);
                *has_introduced = true;
            }
        }
    }
}
