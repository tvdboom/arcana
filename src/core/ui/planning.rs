use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};

use crate::core::audio::{AudioManager, SoundEffect};
use crate::core::compat::MessageWriterSendExt;
use crate::core::localization::Localizer;
use crate::core::persistence::PersistenceManager;
use crate::core::player::{AjahColor, Character, Class, PetType, Race, Slot};
use crate::core::rules::abilities::AbilityDatabase;
use crate::core::rules::items::ItemDatabase;
use crate::core::states::AppState;
use crate::core::systems::actions::ActionManager;
use crate::core::systems::combat_engine::CombatSession;

use bevy::prelude::MessageWriter as EventWriter;

pub struct PlanningUiPlugin;

impl Plugin for PlanningUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, draw_char_creation.run_if(in_state(AppState::CharacterCreation)))
            .add_systems(EguiPrimaryContextPass, draw_planning_ui.run_if(in_state(AppState::Planning)));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum CreationStage {
    #[default]
    Race,
    Class,
    Ajah,
    Pet,
    Name,
}

impl CreationStage {
    fn next(self, class: Class) -> Self {
        match self {
            Self::Race => Self::Class,
            Self::Class => match class {
                Class::Mage => Self::Ajah,
                Class::Druid => Self::Pet,
                _ => Self::Name,
            },
            Self::Ajah | Self::Pet | Self::Name => Self::Name,
        }
    }

    fn previous(self, class: Class) -> Self {
        match self {
            Self::Race => Self::Race,
            Self::Class => Self::Race,
            Self::Ajah | Self::Pet => Self::Class,
            Self::Name => match class {
                Class::Mage => Self::Ajah,
                Class::Druid => Self::Pet,
                _ => Self::Class,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum PetChoice {
    #[default]
    Wolf,
    Bear,
}

// Struct to store character creator choices
#[derive(Resource)]
struct CreatorState {
    name: String,
    race: Race,
    class: Class,
    ajah: Option<AjahColor>,
    pet_choice: PetChoice,
    stage: CreationStage,
}

impl Default for CreatorState {
    fn default() -> Self {
        Self {
            name: String::new(),
            race: Race::Human,
            class: Class::Warrior,
            ajah: None,
            pet_choice: PetChoice::Wolf,
            stage: CreationStage::Race,
        }
    }
}

fn draw_selection_card(
    ui: &mut egui::Ui,
    texture_id: egui::TextureId,
    label: String,
    selected: bool,
) -> bool {
    let mut clicked = false;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add(egui::Image::new((texture_id, egui::vec2(170.0, 132.0))));
            ui.add_space(6.0);
            let text = if selected {
                egui::RichText::new(label.clone()).size(22.0).strong()
            } else {
                egui::RichText::new(label).size(22.0)
            };
            if ui
                .add_sized(egui::vec2(170.0, 40.0), egui::Button::new(text))
                .clicked()
            {
                clicked = true;
            }
        });
    });
    clicked
}

fn draw_char_creation(
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<NextState<AppState>>,
    localizer: Res<Localizer>,
    mut commands: Commands,
    mut sfx_writer: EventWriter<SoundEffect>,
    mut creator: Local<CreatorState>,
) {
    let race_human_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/race_human.png")));
    let race_elf_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/race_elf.png")));
    let race_orc_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/race_orc.png")));
    let race_dwarf_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/race_dwarf.png")));
    let class_warrior_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/class_warrior.png")));
    let class_rogue_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/class_rogue.png")));
    let class_mage_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/class_mage.png")));
    let class_druid_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/class_druid.png")));
    let pet_wolf_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/pet_wolf.png")));
    let pet_bear_tex = contexts.add_image(EguiTextureHandle::Strong(asset_server.load("images/pet_bear.png")));

    let Ok(ctx) = contexts.ctx_mut() else { return; };
    crate::core::ui::theme::apply_custom_theme(ctx);

    egui::Window::new(localizer.t("create_char"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .fixed_size(egui::vec2(840.0, 620.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                let step_key = match creator.stage {
                    CreationStage::Race => "creation_step_race",
                    CreationStage::Class => "creation_step_class",
                    CreationStage::Ajah => "creation_step_ajah",
                    CreationStage::Pet => "creation_step_pet",
                    CreationStage::Name => "creation_step_name",
                };
                let hint_key = match creator.stage {
                    CreationStage::Race => "creation_hint_race",
                    CreationStage::Class => "creation_hint_class",
                    CreationStage::Ajah => "creation_hint_ajah",
                    CreationStage::Pet => "creation_hint_pet",
                    CreationStage::Name => "creation_hint_name",
                };
                ui.label(egui::RichText::new(localizer.t(step_key)).size(30.0).strong());
                ui.label(egui::RichText::new(localizer.t(hint_key)).size(18.0));
                ui.add_space(12.0);

                match creator.stage {
                    CreationStage::Race => {
                        ui.horizontal_centered(|ui| {
                            if draw_selection_card(
                                ui,
                                race_human_tex,
                                localizer.t("race_human"),
                                creator.race == Race::Human,
                            ) {
                                creator.race = Race::Human;
                                sfx_writer.send(SoundEffect::Click);
                            }
                            ui.add_space(10.0);
                            if draw_selection_card(
                                ui,
                                race_elf_tex,
                                localizer.t("race_elf"),
                                creator.race == Race::Elf,
                            ) {
                                creator.race = Race::Elf;
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                        ui.add_space(10.0);
                        ui.horizontal_centered(|ui| {
                            if draw_selection_card(
                                ui,
                                race_orc_tex,
                                localizer.t("race_orc"),
                                creator.race == Race::Orc,
                            ) {
                                creator.race = Race::Orc;
                                sfx_writer.send(SoundEffect::Click);
                            }
                            ui.add_space(10.0);
                            if draw_selection_card(
                                ui,
                                race_dwarf_tex,
                                localizer.t("race_dwarf"),
                                creator.race == Race::Dwarf,
                            ) {
                                creator.race = Race::Dwarf;
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                    }
                    CreationStage::Class => {
                        ui.horizontal_centered(|ui| {
                            if draw_selection_card(
                                ui,
                                class_warrior_tex,
                                localizer.t("class_warrior"),
                                creator.class == Class::Warrior,
                            ) {
                                creator.class = Class::Warrior;
                                creator.ajah = None;
                                sfx_writer.send(SoundEffect::Click);
                            }
                            ui.add_space(10.0);
                            if draw_selection_card(
                                ui,
                                class_rogue_tex,
                                localizer.t("class_rogue"),
                                creator.class == Class::Rogue,
                            ) {
                                creator.class = Class::Rogue;
                                creator.ajah = None;
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                        ui.add_space(10.0);
                        ui.horizontal_centered(|ui| {
                            if draw_selection_card(
                                ui,
                                class_mage_tex,
                                localizer.t("class_mage"),
                                creator.class == Class::Mage,
                            ) {
                                creator.class = Class::Mage;
                                creator.ajah = Some(AjahColor::Yellow);
                                sfx_writer.send(SoundEffect::Click);
                            }
                            ui.add_space(10.0);
                            if draw_selection_card(
                                ui,
                                class_druid_tex,
                                localizer.t("class_druid"),
                                creator.class == Class::Druid,
                            ) {
                                creator.class = Class::Druid;
                                creator.ajah = None;
                                creator.pet_choice = PetChoice::Wolf;
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                    }
                    CreationStage::Ajah => {
                        ui.horizontal_centered(|ui| {
                            if ui
                                .add_sized(
                                    egui::vec2(340.0, 56.0),
                                    egui::Button::new(
                                        egui::RichText::new(localizer.t("ajah_yellow")).size(24.0),
                                    ),
                                )
                                .clicked()
                            {
                                creator.ajah = Some(AjahColor::Yellow);
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                        ui.add_space(10.0);
                        ui.horizontal_centered(|ui| {
                            if ui
                                .add_sized(
                                    egui::vec2(340.0, 56.0),
                                    egui::Button::new(
                                        egui::RichText::new(localizer.t("ajah_green")).size(24.0),
                                    ),
                                )
                                .clicked()
                            {
                                creator.ajah = Some(AjahColor::Green);
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                        ui.add_space(10.0);
                        ui.horizontal_centered(|ui| {
                            if ui
                                .add_sized(
                                    egui::vec2(340.0, 56.0),
                                    egui::Button::new(
                                        egui::RichText::new(localizer.t("ajah_red")).size(24.0),
                                    ),
                                )
                                .clicked()
                            {
                                creator.ajah = Some(AjahColor::Red);
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                        ui.add_space(10.0);
                        ui.horizontal_centered(|ui| {
                            if ui
                                .add_sized(
                                    egui::vec2(340.0, 56.0),
                                    egui::Button::new(
                                        egui::RichText::new(localizer.t("ajah_blue")).size(24.0),
                                    ),
                                )
                                .clicked()
                            {
                                creator.ajah = Some(AjahColor::Blue);
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                    }
                    CreationStage::Pet => {
                        ui.horizontal_centered(|ui| {
                            if draw_selection_card(
                                ui,
                                pet_wolf_tex,
                                localizer.t("pet_wolf"),
                                creator.pet_choice == PetChoice::Wolf,
                            ) {
                                creator.pet_choice = PetChoice::Wolf;
                                sfx_writer.send(SoundEffect::Click);
                            }
                            ui.add_space(16.0);
                            if draw_selection_card(
                                ui,
                                pet_bear_tex,
                                localizer.t("pet_bear"),
                                creator.pet_choice == PetChoice::Bear,
                            ) {
                                creator.pet_choice = PetChoice::Bear;
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                    }
                    CreationStage::Name => {
                        ui.label(
                            egui::RichText::new(localizer.format(
                                "race_value",
                                &[(
                                    "race",
                                    localizer.t(&format!("race_{:?}", creator.race).to_lowercase()),
                                )],
                            ))
                            .size(20.0),
                        );
                        ui.label(
                            egui::RichText::new(localizer.format(
                                "class_value",
                                &[(
                                    "class",
                                    localizer
                                        .t(&format!("class_{:?}", creator.class).to_lowercase()),
                                )],
                            ))
                            .size(20.0),
                        );
                        if let Some(ajah) = creator.ajah {
                            ui.label(
                                egui::RichText::new(localizer.format(
                                    "ajah_value",
                                    &[(
                                        "ajah",
                                        localizer.t(
                                            &format!("ajah_{:?}_short", ajah).to_lowercase(),
                                        ),
                                    )],
                                ))
                                .size(20.0),
                            );
                        }
                        if creator.class == Class::Druid {
                            let pet_name_key = match creator.pet_choice {
                                PetChoice::Wolf => "pet_wolf",
                                PetChoice::Bear => "pet_bear",
                            };
                            ui.label(
                                egui::RichText::new(localizer.format(
                                    "pet_value",
                                    &[("pet", localizer.t(pet_name_key))],
                                ))
                                .size(20.0),
                            );
                        }
                        ui.add_space(18.0);
                        ui.horizontal_centered(|ui| {
                            ui.label(egui::RichText::new(localizer.t("enter_name")).size(22.0));
                            ui.add(
                                egui::TextEdit::singleline(&mut creator.name)
                                    .desired_width(280.0)
                                    .font(egui::TextStyle::Heading),
                            );
                        });
                    }
                }

                ui.add_space(20.0);
                let can_continue = match creator.stage {
                    CreationStage::Ajah => creator.ajah.is_some(),
                    CreationStage::Name => !creator.name.trim().is_empty(),
                    _ => true,
                };
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            egui::vec2(180.0, 48.0),
                            egui::Button::new(egui::RichText::new(localizer.t("back")).size(22.0)),
                        )
                        .clicked()
                    {
                        sfx_writer.send(SoundEffect::Click);
                        if creator.stage == CreationStage::Race {
                            app_state.set(AppState::MainMenu);
                        } else {
                            creator.stage = creator.stage.previous(creator.class);
                        }
                    }
                    ui.add_space(20.0);
                    let next_text = if creator.stage == CreationStage::Name {
                        localizer.t("finish_creation")
                    } else {
                        localizer.t("continue")
                    };
                    if ui
                        .add_enabled(
                            can_continue,
                            egui::Button::new(egui::RichText::new(next_text).size(22.0)),
                        )
                        .clicked()
                    {
                        sfx_writer.send(SoundEffect::Click);
                        if creator.stage == CreationStage::Name {
                            let mut character = Character::new(
                                creator.name.clone(),
                                creator.race,
                                creator.class,
                                creator.ajah,
                            );
                            if creator.class == Class::Druid {
                                if let Some(pet) = character.pet.as_mut() {
                                    match creator.pet_choice {
                                        PetChoice::Wolf => {
                                            pet.name = "Greyfang".to_string();
                                            pet.pet_type = PetType::Wolf;
                                            pet.max_health = 60;
                                            pet.current_health = 60;
                                            pet.strength = 8;
                                            pet.dexterity = 8;
                                            pet.vitality = 8;
                                        }
                                        PetChoice::Bear => {
                                            pet.name = "Stonepaw".to_string();
                                            pet.pet_type = PetType::Bear;
                                            pet.max_health = 72;
                                            pet.current_health = 72;
                                            pet.strength = 10;
                                            pet.dexterity = 6;
                                            pet.vitality = 10;
                                        }
                                    }
                                }
                            }
                            let _ = PersistenceManager::save_character(&character);
                            commands.insert_resource(character);
                            *creator = CreatorState::default();
                            app_state.set(AppState::Planning);
                        } else {
                            creator.stage = creator.stage.next(creator.class);
                        }
                    }
                });
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn draw_planning_ui(
    mut contexts: EguiContexts,
    mut character: ResMut<Character>,
    mut app_state: ResMut<NextState<AppState>>,
    localizer: Res<Localizer>,
    mut combat_session: ResMut<CombatSession>,
    audio_manager: Res<AudioManager>,
    mut sfx_writer: EventWriter<SoundEffect>,
    mut quest_log: Local<Option<(String, String)>>,
    mut show_resolve_modal: Local<bool>,
    mut resolve_choices: Local<Option<(String, usize)>>, // allocated stats & chosen ability
) {
    let character = &mut *character;
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    crate::core::ui::theme::apply_custom_theme(ctx);

    // Sidebar: Stat Sheet
    egui::SidePanel::left("character_sheet")
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.heading(localizer.format("character_level", &[("name", character.name.clone()), ("level", character.level.to_string())]));
            ui.label(localizer.format("xp_value", &[("xp", character.xp.to_string()), ("target", (character.level * 100).to_string())]));
            ui.add_space(10.0);

            // Race / Class
            ui.horizontal(|ui| {
                ui.label(localizer.format("race_value", &[("race", localizer.t(&format!("race_{:?}", character.race).to_lowercase()))]));
                ui.add_space(10.0);
                ui.label(localizer.format("class_value", &[("class", localizer.t(&format!("class_{:?}", character.class).to_lowercase()))]));
            });
            if let Some(ajah) = &character.ajah {
                ui.label(localizer.format("ajah_value", &[("ajah", localizer.t(&format!("ajah_{:?}_short", ajah).to_lowercase()))]));
            }
            if let Some(trans) = &character.transformation {
                ui.label(egui::RichText::new(localizer.format("transformation_value", &[("transformation", localizer.t(&format!("transformation_{:?}", trans).to_lowercase()))]))
                    .color(egui::Color32::from_rgb(160, 32, 240)).strong());
            }
            ui.add_space(15.0);

            // AP Remaining
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}: {} / {}", localizer.t("ap"), character.ap, character.max_ap))
                    .size(18.0).color(egui::Color32::from_rgb(0, 240, 255)));
            });
            ui.add_space(15.0);

            // Core Vitals
            ui.label(localizer.format("gold_value", &[("gold", character.gold.to_string())]));
            ui.label(localizer.format("health_value", &[("current", character.current_health.to_string()), ("max", character.max_health().to_string())]));
            ui.label(localizer.format("mana_value", &[("current", character.current_mana.to_string()), ("max", character.max_mana().to_string())]));
            ui.label(localizer.format("armor_value", &[("armor", character.armor().to_string())]));
            ui.label(localizer.format("evasion_value", &[("value", format!("{:.1}", character.evasion_rate() * 100.0))]));
            ui.label(localizer.format("accuracy_value", &[("value", format!("{:.1}", character.accuracy_rate() * 100.0))]));
            ui.label(localizer.format("critical_value", &[("value", format!("{:.1}", character.critical_rate() * 100.0))]));
            ui.add_space(20.0);

            // Stat Point Training
            ui.heading(localizer.t("stats"));
            ui.add_space(5.0);
            
            let train_cost = character.level * 20;
            let display_stat = |ui: &mut egui::Ui, name_key: &str, current: u32, spent: u32, key: &str, character: &mut Character, sfx_writer: &mut EventWriter<SoundEffect>| {
                ui.horizontal(|ui| {
                    ui.label(localizer.format("stat_value", &[("stat", localizer.t(name_key)), ("current", current.to_string()), ("spent", spent.to_string())]));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(localizer.format("increase_gold", &[("gold", train_cost.to_string())])).clicked() {
                            if let Err(e) = ActionManager::train_stat(character, key) {
                                println!("Training failed: {}", e);
                            } else {
                                sfx_writer.send(SoundEffect::Click);
                            }
                        }
                    });
                });
            };

            display_stat(ui, "stat_strength", character.strength(), character.spent_stats.strength, "strength", character, &mut sfx_writer);
            display_stat(ui, "stat_dexterity", character.dexterity(), character.spent_stats.dexterity, "dexterity", character, &mut sfx_writer);
            display_stat(ui, "stat_intelligence", character.intelligence(), character.spent_stats.intelligence, "intelligence", character, &mut sfx_writer);
            display_stat(ui, "stat_charisma", character.charisma(), character.spent_stats.charisma, "charisma", character, &mut sfx_writer);

            ui.add_space(25.0);

            // Druid/Acquired Pet Panel
            if let Some(ref mut pet) = &mut character.pet {
                ui.heading(localizer.format("pet_level", &[("name", pet.name.clone()), ("level", pet.level.to_string())]));
                ui.label(localizer.format("health_value", &[("current", pet.current_health.to_string()), ("max", pet.max_health().to_string())]));
                ui.label(localizer.format("base_damage_value", &[("damage", pet.damage().to_string())]));
                ui.label(localizer.format("evasion_value", &[("value", format!("{:.1}", pet.evasion_rate() * 100.0))]));
                
                let pet_cost = character.level * 15;
                ui.horizontal(|ui| {
                    if ui.button(localizer.format("train_pet_strength", &[("gold", pet_cost.to_string())])).clicked() {
                        let mut gold = character.gold;
                        if pet.train_strength(pet_cost, &mut gold).is_ok() {
                            character.gold = gold;
                            sfx_writer.send(SoundEffect::Click);
                        }
                    }
                    if ui.button(localizer.format("train_pet_dexterity", &[("gold", pet_cost.to_string())])).clicked() {
                        let mut gold = character.gold;
                        if pet.train_dexterity(pet_cost, &mut gold).is_ok() {
                            character.gold = gold;
                            sfx_writer.send(SoundEffect::Click);
                        }
                    }
                    if ui.button(localizer.format("train_pet_vitality", &[("gold", pet_cost.to_string())])).clicked() {
                        let mut gold = character.gold;
                        if pet.train_vitality(pet_cost, &mut gold).is_ok() {
                            character.gold = gold;
                            sfx_writer.send(SoundEffect::Click);
                        }
                    }
                });
            }
        });

    // Central Area: Activities and Gear Shop
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button(localizer.t("back")).clicked() {
                sfx_writer.send(SoundEffect::Click);
                let _ = PersistenceManager::save_character(character);
                app_state.set(AppState::MainMenu);
            }
            ui.add_space(20.0);
            if ui.button(localizer.t("pvp_lobby")).clicked() {
                sfx_writer.send(SoundEffect::Click);
                app_state.set(AppState::PvPLobby);
            }
            ui.add_space(20.0);
            if ui.button(localizer.t("manual_save")).clicked() {
                sfx_writer.send(SoundEffect::Click);
                let _ = PersistenceManager::save_character(character);
            }
        });
        ui.add_space(20.0);

        // Quest outcome logger
        if let Some((title, text)) = quest_log.clone() {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(34, 43, 58))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(&title).size(16.0).color(egui::Color32::from_rgb(0, 240, 255)));
                    ui.label(&text);
                    if ui.button(localizer.t("dismiss")).clicked() {
                        *quest_log = None;
                        sfx_writer.send(SoundEffect::Click);
                    }
                });
            ui.add_space(15.0);
        }

        ui.columns(2, |columns| {
            // Column 0: Standard AP Actions
            columns[0].vertical(|ui| {
                ui.heading(localizer.t("ap_actions"));
                ui.add_space(10.0);

                if ui.button(localizer.format("work_button", &[("gold", format!("{:.0}", 40.0 * character.gold_work_mult()))])).clicked() {
                    if let Ok(g) = ActionManager::work(character) {
                        sfx_writer.send(SoundEffect::Click);
                        println!("Worked and earned {} gold", g);
                    }
                }
                ui.add_space(10.0);

                if ui.button(localizer.t("rest_button")).clicked()
                    && ActionManager::rest(character).is_ok()
                {
                    sfx_writer.send(SoundEffect::Click);
                }
                ui.add_space(10.0);

                if ui.button(localizer.t("quest_button")).clicked() {
                    match ActionManager::trigger_quest(character) {
                        Ok((t, log)) => {
                            *quest_log = Some((t, log));
                            sfx_writer.send(SoundEffect::LevelUp); // Quest fan-fare
                        }
                        Err(e) => { println!("Quest failed: {}", e); }
                    }
                }
                ui.add_space(10.0);

                // Study panel
                ui.label(localizer.t("study_new_spells"));
                let learned_names: Vec<String> = character.abilities.iter().map(|a| a.name.clone()).collect();
                let available_spells: Vec<_> = AbilityDatabase::get_all_abilities()
                    .into_iter()
                    .filter(|a| !learned_names.contains(&a.name))
                    .collect();
                
                let study_cost = character.level * 40;
                egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                    for abi in available_spells {
                        ui.horizontal(|ui| {
                            ui.label(localizer.format("named_gold", &[("name", abi.name.clone()), ("gold", study_cost.to_string())]));
                            if ui.button(localizer.t("study_button")).clicked()
                                && ActionManager::study_ability(character, &abi.name).is_ok()
                            {
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                    }
                });

                ui.add_space(20.0);

                // Hunting (PVE Combat Trigger)
                ui.heading(localizer.t("hunting_expeditions"));
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    for npc_name in &["Goblin", "Bandit", "Knight", "Mage", "Beast"] {
                        if ui.button(localizer.format("hunt_named", &[("name", localizer.t(&format!("npc_{}", npc_name.to_lowercase())))])).clicked() && character.ap >= 2 {
                            character.ap -= 2;
                            sfx_writer.send(SoundEffect::Click);
                            *combat_session = CombatSession::init_hunt(character, npc_name);
                            app_state.set(AppState::Combat);
                        }
                    }
                });
            });

            // Column 1: Inventory, Equipment, & Upgrading
            columns[1].vertical(|ui| {
                ui.heading(localizer.t("equipment_inventory"));
                ui.add_space(10.0);

                // Show Equipped Items
                ui.label(localizer.t("equipped"));
                let draw_equipped = |ui: &mut egui::Ui, slot_name: &str, slot: Slot, character: &mut Character, sfx_writer: &mut EventWriter<SoundEffect>| {
                    let item_name = match slot {
                        Slot::Helmet => character.gear.helmet.as_ref().map(|i| i.name.clone()).unwrap_or_else(|| localizer.t("none")),
                        Slot::Armor => character.gear.armor.as_ref().map(|i| i.name.clone()).unwrap_or_else(|| localizer.t("none")),
                        Slot::Boots => character.gear.boots.as_ref().map(|i| i.name.clone()).unwrap_or_else(|| localizer.t("none")),
                        Slot::Weapon => character.gear.main_hand.as_ref().map(|i| i.name.clone()).unwrap_or_else(|| localizer.t("none")),
                        Slot::Consumable => localizer.t("not_applicable"),
                    };
                    let has_item = item_name != localizer.t("none");
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {}", slot_name, item_name));
                        if has_item {
                            let upgrade_cost = character.level * 30;
                            if ui.button(localizer.format("upgrade_button", &[("gold", upgrade_cost.to_string())])).clicked()
                                && ActionManager::craft_upgrade(character, slot).is_ok()
                            {
                                sfx_writer.send(SoundEffect::Click);
                            }
                        }
                    });
                };

                draw_equipped(ui, &localizer.t("slot_helmet"), Slot::Helmet, character, &mut sfx_writer);
                draw_equipped(ui, &localizer.t("slot_armor"), Slot::Armor, character, &mut sfx_writer);
                draw_equipped(ui, &localizer.t("slot_boots"), Slot::Boots, character, &mut sfx_writer);
                draw_equipped(ui, &localizer.t("slot_main_weapon"), Slot::Weapon, character, &mut sfx_writer);

                ui.add_space(15.0);

                // Shop items
                ui.label(localizer.t("merchant_shop"));
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    for item in ItemDatabase::get_shop_items() {
                        let final_cost = (item.cost as f32 * (1.0 - character.shop_discount_rate())) as u32;
                        ui.horizontal(|ui| {
                            ui.label(localizer.format("named_gold", &[("name", item.name.clone()), ("gold", final_cost.to_string())]));
                            if ui.button(localizer.t("buy")).clicked() && character.gold >= final_cost {
                                character.gold -= final_cost;
                                character.inventory.push(item.clone());
                                sfx_writer.send(SoundEffect::Click);
                            }
                        });
                    }
                });

                ui.add_space(15.0);

                // Bag/Inventory
                ui.label(localizer.t("inventory_bag"));
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    let mut to_equip = None;
                    let mut to_sell = None;
                    for (idx, item) in character.inventory.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(&item.name);
                            if ui.button(localizer.t("equip")).clicked() {
                                to_equip = Some(idx);
                            }
                            if ui.button(localizer.t("sell")).clicked() {
                                to_sell = Some(idx);
                            }
                        });
                    }

                    if let Some(idx) = to_equip {
                        sfx_writer.send(SoundEffect::Click);
                        let item = character.inventory.remove(idx);
                        match item.slot {
                            Slot::Helmet => {
                                if let Some(old) = character.gear.helmet.replace(item) {
                                    character.inventory.push(old);
                                }
                            }
                            Slot::Armor => {
                                if let Some(old) = character.gear.armor.replace(item) {
                                    character.inventory.push(old);
                                }
                            }
                            Slot::Boots => {
                                if let Some(old) = character.gear.boots.replace(item) {
                                    character.inventory.push(old);
                                }
                            }
                            Slot::Weapon => {
                                if let Some(old) = character.gear.main_hand.replace(item) {
                                    character.inventory.push(old);
                                }
                            }
                            Slot::Consumable => {
                                if character.equipped_consumables.len() < 2 {
                                    character.equipped_consumables.push(item);
                                } else {
                                    character.inventory.push(item);
                                }
                            }
                        }
                    }

                    if let Some(idx) = to_sell {
                        sfx_writer.send(SoundEffect::Click);
                        let item = character.inventory.remove(idx);
                        character.gold += item.cost / 2;
                    }
                });
            });
        });

        // Resolve planning phase
        ui.add_space(20.0);
        let planning_done = character.ap == 0;
        if ui.add_enabled(planning_done, egui::Button::new(egui::RichText::new(localizer.t("next_level")).size(16.0).color(egui::Color32::from_rgb(0, 240, 255)))).clicked() {
            sfx_writer.send(SoundEffect::Click);
            *show_resolve_modal = true;
            *resolve_choices = Some(("strength".to_string(), 0)); // default allocation
        }
    });

    // Level Resolution Modal Overlay
    if *show_resolve_modal {
        egui::Window::new(localizer.t("level_up_choices"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(localizer.t("allocate_stats"));
                    let choices = resolve_choices.as_mut().unwrap();

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut choices.0, "strength".to_string(), localizer.t("strength_plus_two"));
                        ui.radio_value(&mut choices.0, "dexterity".to_string(), localizer.t("dexterity_plus_two"));
                        ui.radio_value(&mut choices.0, "intelligence".to_string(), localizer.t("intelligence_plus_two"));
                        ui.radio_value(&mut choices.0, "charisma".to_string(), localizer.t("charisma_plus_two"));
                    });
                    ui.add_space(15.0);

                    ui.label(localizer.t("select_ability_reward"));
                    let abilities = character.abilities.clone();
                    if abilities.is_empty() {
                        ui.label(localizer.t("no_spells_studied"));
                    } else {
                        for (idx, abi) in abilities.iter().enumerate() {
                            ui.radio_value(&mut choices.1, idx, &abi.name);
                        }
                    }
                    ui.add_space(25.0);

                    if ui.button(localizer.t("finalize_progression")).clicked() {
                        sfx_writer.send(SoundEffect::LevelUp);
                        
                        // Apply chosen stat increase
                        match choices.0.as_str() {
                            "strength" => character.spent_stats.strength += 2,
                            "dexterity" => character.spent_stats.dexterity += 2,
                            "intelligence" => character.spent_stats.intelligence += 2,
                            "charisma" => character.spent_stats.charisma += 2,
                            _ => {}
                        }

                        // Ability selection (equips it)
                        if !abilities.is_empty() {
                            let chosen_name = abilities[choices.1].name.clone();
                            if !character.equipped_abilities.contains(&chosen_name) {
                                if character.equipped_abilities.len() >= 3 {
                                    character.equipped_abilities.remove(0); // evict oldest
                                }
                                character.equipped_abilities.push(chosen_name);
                            }
                        }

                        // Close level phase
                        character.level += 1;
                        character.ap = character.max_ap;
                        
                        // Heal back full vitals on level up
                        character.current_health = character.max_health();
                        character.current_mana = character.max_mana();

                        *show_resolve_modal = false;

                        // Trigger Auto-Save if enabled
                        if audio_manager.current_settings.auto_save {
                            let _ = PersistenceManager::save_character(character);
                            println!("Auto-saved character profile.");
                        }
                    }
                });
            });
    }
}
