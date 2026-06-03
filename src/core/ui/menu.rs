use bevy::prelude::*;
use bevy::app::AppExit;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::core::audio::{AudioManager, SoundEffect};
use crate::core::compat::MessageWriterSendExt;
use crate::core::localization::{Localizer, Language};
use crate::core::persistence::{AudioMode, PersistenceManager};
use crate::core::states::AppState;

#[derive(Component)]
pub struct MainMenuBgSprite;

pub struct MenuUiPlugin;

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_menu_bg)
            .add_systems(OnExit(AppState::MainMenu), despawn_menu_bg)
            .add_systems(EguiPrimaryContextPass, draw_menu_ui.run_if(in_state(AppState::MainMenu)));
    }
}

fn spawn_menu_bg(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Sprite {
            image: asset_server.load("images/main_menu_bg.png"),
            custom_size: Some(Vec2::new(1600.0, 900.0)),
            ..default()
        },
        MainMenuBgSprite,
    ));
}

fn despawn_menu_bg(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuBgSprite>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_menu_ui(
    mut contexts: EguiContexts,
    mut app_state: ResMut<NextState<AppState>>,
    mut audio_manager: ResMut<AudioManager>,
    mut localizer: ResMut<Localizer>,
    mut commands: Commands,
    mut sfx_writer: MessageWriter<SoundEffect>,
    mut exit: MessageWriter<AppExit>,
    mut show_settings: Local<bool>,
    mut show_load_dialog: Local<bool>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    crate::core::ui::theme::apply_custom_theme(ctx);

    let panel_width = 360.0;
    let button_size = egui::vec2(230.0, 42.0);
    
    egui::Window::new("main_menu_panel")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .fixed_size(egui::vec2(panel_width, 420.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_width(panel_width - 18.0);
                ui.add_space(18.0);
                
                if *show_settings {
                    // Settings View
                    ui.label(egui::RichText::new(localizer.t("settings")).size(22.0));
                    ui.add_space(14.0);

                    // Language toggles
                    ui.label(egui::RichText::new(localizer.t("language")).size(16.0));
                    ui.horizontal_centered(|ui| {
                        if ui.selectable_label(localizer.language == Language::English, "EN").clicked() {
                            localizer.language = Language::English;
                            audio_manager.current_settings.language = Language::English;
                            let _ = PersistenceManager::save_settings(&audio_manager.current_settings);
                            sfx_writer.write(SoundEffect::Click);
                        }
                        if ui.selectable_label(localizer.language == Language::Spanish, "ES").clicked() {
                            localizer.language = Language::Spanish;
                            audio_manager.current_settings.language = Language::Spanish;
                            let _ = PersistenceManager::save_settings(&audio_manager.current_settings);
                            sfx_writer.write(SoundEffect::Click);
                        }
                    });
                    ui.add_space(12.0);

                    // Audio configuration
                    ui.label(egui::RichText::new(localizer.t("audio_mode")).size(16.0));
                    let mut current_mode = audio_manager.current_settings.audio_mode;
                    ui.horizontal_centered(|ui| {
                        if ui.selectable_label(current_mode == AudioMode::Mute, localizer.t("audio_mute")).clicked() {
                            current_mode = AudioMode::Mute;
                            sfx_writer.write(SoundEffect::Click);
                        }
                        if ui.selectable_label(current_mode == AudioMode::SfxOnly, localizer.t("audio_sfx_only")).clicked() {
                            current_mode = AudioMode::SfxOnly;
                            sfx_writer.write(SoundEffect::Click);
                        }
                        if ui.selectable_label(current_mode == AudioMode::SfxAndMusic, localizer.t("audio_music_sfx")).clicked() {
                            current_mode = AudioMode::SfxAndMusic;
                            sfx_writer.write(SoundEffect::Click);
                        }
                    });
                    if current_mode != audio_manager.current_settings.audio_mode {
                        audio_manager.current_settings.audio_mode = current_mode;
                        let _ = PersistenceManager::save_settings(&audio_manager.current_settings);
                    }
                    ui.add_space(12.0);

                    // Auto Save
                    let mut auto_save = audio_manager.current_settings.auto_save;
                    ui.horizontal_centered(|ui| {
                        if ui.checkbox(&mut auto_save, egui::RichText::new(localizer.t("auto_save")).size(16.0)).changed() {
                            audio_manager.current_settings.auto_save = auto_save;
                            let _ = PersistenceManager::save_settings(&audio_manager.current_settings);
                            sfx_writer.write(SoundEffect::Click);
                        }
                    });
                    ui.add_space(20.0);

                    // Back button
                    ui.horizontal_centered(|ui| {
                        if ui.add_sized(button_size, egui::Button::new(egui::RichText::new(localizer.t("back")).size(19.0))).clicked() {
                            *show_settings = false;
                            sfx_writer.write(SoundEffect::Click);
                        }
                    });

                } else if *show_load_dialog {
                    // Load Dialog
                    ui.label(egui::RichText::new(localizer.t("saves_list")).size(22.0));
                    ui.add_space(12.0);

                    let saves = PersistenceManager::list_saves();
                    if saves.is_empty() {
                        ui.label(egui::RichText::new(localizer.t("no_saves_found")).size(18.0));
                        ui.add_space(15.0);
                    } else {
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for name in saves {
                                    if ui.add_sized(button_size, egui::Button::new(egui::RichText::new(localizer.format("load_save", &[("name", name.clone())])).size(18.0))).clicked() {
                                    sfx_writer.send(SoundEffect::Click);
                                    match PersistenceManager::load_character(&name) {
                                        Ok(character) => {
                                            commands.insert_resource(character);
                                            *show_load_dialog = false;
                                            app_state.set(AppState::Planning);
                                        }
                                        Err(err) => {
                                            println!("Failed to load: {}", err);
                                        }
                                    }
                                }
                                ui.add_space(5.0);
                            }
                        });
                    }

                    ui.add_space(12.0);
                    ui.horizontal_centered(|ui| {
                        if ui.add_sized(button_size, egui::Button::new(egui::RichText::new(localizer.t("back")).size(19.0))).clicked() {
                            *show_load_dialog = false;
                            sfx_writer.write(SoundEffect::Click);
                        }
                    });

                } else {
                    // Default Title Menu
                    ui.add_space(18.0);

                    ui.horizontal_centered(|ui| {
                        if ui.add_sized(button_size, egui::Button::new(egui::RichText::new(localizer.t("new_game")).size(20.0))).clicked() {
                            sfx_writer.write(SoundEffect::Click);
                            app_state.set(AppState::CharacterCreation);
                        }
                    });
                    ui.add_space(12.0);

                    ui.horizontal_centered(|ui| {
                        if ui.add_sized(button_size, egui::Button::new(egui::RichText::new(localizer.t("load_game")).size(20.0))).clicked() {
                            sfx_writer.write(SoundEffect::Click);
                            *show_load_dialog = true;
                        }
                    });
                    ui.add_space(12.0);

                    ui.horizontal_centered(|ui| {
                        if ui.add_sized(button_size, egui::Button::new(egui::RichText::new(localizer.t("settings")).size(20.0))).clicked() {
                            sfx_writer.send(SoundEffect::Click);
                            *show_settings = true;
                        }
                    });
                    ui.add_space(12.0);

                    ui.horizontal_centered(|ui| {
                        if ui.add_sized(button_size, egui::Button::new(egui::RichText::new(localizer.t("quit")).size(20.0))).clicked() {
                            sfx_writer.write(SoundEffect::Click);
                            exit.write(AppExit::Success);
                        }
                    });
                }

                ui.add_space(18.0);
            });
        });

    Ok(())
}
