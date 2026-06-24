pub mod actions;
mod assets;
mod audio;
mod camera;
mod catalog;
pub mod classes;
mod combat;
mod constants;
pub mod game_state;
pub mod localization;
mod menu;
mod monsters;
#[cfg(not(target_arch = "wasm32"))]
mod persistence;
mod player;
mod races;
mod settings;
mod states;
mod systems;
mod ui;
mod utils;

use crate::core::actions::craft::{setup_craft_ui, update_craft_ui, CraftSeed};
use crate::core::actions::duel::setup_duel_ui;
use crate::core::actions::hunt::{apply_pending_hunt_xp, setup_hunt_ui, update_hunt_ui, PendingHuntXp};
use crate::core::actions::quest::{apply_pending_quest_xp, setup_quest_ui, update_quest_ui, PendingQuestXp};
use crate::core::combat::{setup_combat_ui, CombatCmp};
use crate::core::actions::rest::{setup_rest_ui, update_rest_ui};
use crate::core::actions::shop::*;
use crate::core::actions::study::{setup_study_ui, update_study_ui, StudySliderState};
use crate::core::actions::train::{setup_train_ui, update_train_ui, TrainSliderState};
use crate::core::actions::work::{setup_work_ui, update_work_ui, WorkSliderState};
use crate::core::assets::WorldAssets;
use crate::core::audio::*;
use crate::core::camera::*;
use crate::core::game_state::ShopUiState;
use crate::core::localization::{update_localized_text, Localization};
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::persistence::*;
use crate::core::player::Player;
use crate::core::settings::Settings;
use crate::core::states::{is_panel_state, AppState, GameState};
use crate::core::systems::*;
use crate::core::ui::creation::*;
use crate::core::ui::dropdown::{shop_close_dropdown_on_outside_click, OpenDropdown};
use crate::core::ui::level_up::{apply_level_up_system, ApplyLevelUpMsg, LevelUpOverlayCmp};
use crate::core::ui::modal::{modal_input_system, ActiveModal};
use crate::core::ui::playing::*;
use crate::core::ui::scrollbar::{scroll_system, update_scrollbar_system};
use crate::core::ui::toast::{tick_gold_toasts, GoldToast};
use crate::core::ui::utils::cleanup_panel_ui;
use crate::core::utils::{despawn, reset_cursor};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;
use strum::IntoEnumIterator;

pub struct GamePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InPlayingSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct InCombatSet;

macro_rules! configure_stages {
    ($app:expr, $set:ident, $run_if:expr) => {
        $app.configure_sets(First, $set.run_if($run_if))
            .configure_sets(PreUpdate, $set.run_if($run_if))
            .configure_sets(Update, $set.run_if($run_if))
            .configure_sets(PostUpdate, $set.run_if($run_if))
            .configure_sets(Last, $set.run_if($run_if));
    };
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .init_state::<AppState>()
            .init_state::<GameState>()
            // Messages
            .add_message::<PlayAudioMsg>()
            .add_message::<PauseAudioMsg>()
            .add_message::<StopAudioMsg>()
            .add_message::<MuteAudioMsg>()
            .add_message::<ChangeAudioMsg>()
            .add_message::<StartNewCharacterMsg>()
            .add_message::<ApplyLevelUpMsg>()
            // Resources
            .init_resource::<WorldAssets>()
            .init_resource::<PlayingAudio>()
            .init_resource::<Localization>()
            .init_resource::<Settings>()
            .init_resource::<Player>()
            .init_resource::<LevelUpPending>()
            .init_resource::<ActiveModal>()
            .init_resource::<ShopInventory>()
            .init_resource::<ShopUiState>()
            .init_resource::<OpenDropdown>()
            .init_resource::<ShopTabClickGuard>()
            .init_resource::<WorkSliderState>()
            .init_resource::<StudySliderState>()
            .init_resource::<TrainSliderState>()
            .init_resource::<CraftSeed>()
            .init_resource::<PendingHuntXp>()
            .init_resource::<PendingQuestXp>()
            .init_resource::<RightTab>()
            .init_resource::<RightTabScroll>();

        // Sets
        configure_stages!(app, InGameSet, in_state(AppState::Game));
        configure_stages!(
            app,
            InPlayingSet,
            in_state(GameState::Playing).and_then(in_state(AppState::Game))
        );
        configure_stages!(
            app,
            InCombatSet,
            in_state(GameState::Combat).and_then(in_state(AppState::Game))
        );

        app
            // Camera
            .add_systems(Startup, setup_camera)
            // Audio
            .add_systems(Startup, setup_audio)
            .add_systems(OnEnter(GameState::Playing), play_music)
            .add_systems(
                Update,
                (toggle_audio, update_audio, play_audio, pause_audio, stop_audio, mute_audio),
            );

        // Menu
        app.add_systems(OnEnter(AppState::MainMenu), (reset_cursor, setup_menu))
            .add_systems(OnExit(AppState::MainMenu), despawn::<MenuCmp>)
            .add_systems(OnEnter(AppState::Settings), (reset_cursor, setup_menu))
            .add_systems(OnExit(AppState::Settings), despawn::<MenuCmp>)
            .add_systems(OnEnter(AppState::Loading), (reset_cursor, setup_loading_screen))
            .add_systems(OnExit(AppState::Loading), despawn::<LoadingCmp>);
        for state in GameState::iter() {
            if !is_panel_state(state) {
                app.add_systems(OnEnter(state), reset_cursor);
            }
        }
        app.add_systems(Update, start_new_game_message.run_if(not(in_state(AppState::Game))));

        app
            // Utilities
            .add_systems(
                Update,
                (
                    check_keys_menu,
                    apply_level_up_system,
                    modal_input_system,
                    update_localized_text.run_if(resource_changed::<Settings>),
                    update_playing_screen.run_if(resource_changed::<Settings>),
                    scroll_system.before(update_scrollbar_system).run_if(in_state(AppState::Game)),
                    update_scrollbar_system.run_if(in_state(AppState::Game)),
                    reveal_menu_content_when_bg_ready
                        .run_if(in_state(AppState::MainMenu).or_else(in_state(AppState::Settings))),
                    animate_loading_text.run_if(in_state(AppState::Loading)),
                    complete_loading_when_ready.run_if(in_state(AppState::Loading)),
                ),
            )
            .add_systems(OnEnter(GameState::CreateCharacter), setup_character_creation)
            .add_systems(OnExit(GameState::CreateCharacter), despawn::<MenuCmp>)
            .add_systems(
                Update,
                (
                    handle_name_input,
                    update_character_creation_continue_btn,
                    update_attribute_buttons,
                    update_sex_button_colors,
                )
                    .run_if(in_state(GameState::CreateCharacter)),
            )
            .add_systems(OnEnter(GameState::ChooseRace), setup_race_selection)
            .add_systems(OnExit(GameState::ChooseRace), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::ChooseClass), setup_class_selection)
            .add_systems(OnExit(GameState::ChooseClass), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::ChooseSubClass), setup_subclass_selection)
            .add_systems(OnExit(GameState::ChooseSubClass), despawn::<MenuCmp>)
            .add_systems(Update, handle_pet_name_input.run_if(in_state(GameState::ChooseSubClass)))
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    apply_pending_hunt_xp,
                    apply_pending_quest_xp,
                    setup_playing_screen,
                    rebuild_playing_lists,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (despawn::<TooltipNode>, despawn::<GoldToast>, despawn::<LevelUpOverlayCmp>),
            )
            .add_systems(OnExit(AppState::Game), (despawn::<PlayingCmp>, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_playing_screen,
                    update_action_buttons,
                    tab_button_hover_system,
                    equip_slot_tooltip_system,
                    right_column_tooltip_system,
                    info_tooltip_system,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    manage_level_up_overlay,
                    update_active_hotkey_slots,
                    active_hotkey_slot_tooltip_system,
                    restore_tab_scroll,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                rebuild_playing_lists.run_if(in_state(AppState::Game)).run_if(
                    resource_changed::<Player>
                        .or_else(resource_changed::<Settings>)
                        .or_else(resource_changed::<RightTab>),
                ),
            )
            .add_systems(OnEnter(GameState::GameMenu), setup_game_menu)
            .add_systems(OnExit(GameState::GameMenu), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::Settings), setup_game_settings)
            .add_systems(OnExit(GameState::Settings), despawn::<MenuCmp>)
            // Shop Systems
            .add_systems(OnEnter(GameState::Shop), setup_shop_ui)
            .add_systems(
                OnExit(GameState::Shop),
                (
                    remember_shop_scroll_position,
                    cleanup_panel_ui,
                    despawn::<TooltipNode>,
                ),
            )
            .add_systems(
                Update,
                (
                    update_shop_ui,
                    update_shop_gold_system,
                    shop_tooltip_system,
                    shop_tab_button_system,
                    shop_close_dropdown_on_outside_click,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Shop)),
            )
            // Work Systems
            .add_systems(OnEnter(GameState::Work), setup_work_ui)
            .add_systems(OnExit(GameState::Work), (cleanup_panel_ui, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_work_ui,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Work)),
            )
            // Study Systems
            .add_systems(OnEnter(GameState::Study), setup_study_ui)
            .add_systems(OnExit(GameState::Study), (cleanup_panel_ui, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_study_ui,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Study)),
            )
            // Rest Systems
            .add_systems(OnEnter(GameState::Rest), setup_rest_ui)
            .add_systems(OnExit(GameState::Rest), (cleanup_panel_ui, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_rest_ui,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Rest)),
            )
            // Train Systems
            .add_systems(OnEnter(GameState::Train), setup_train_ui)
            .add_systems(OnExit(GameState::Train), (cleanup_panel_ui, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_train_ui,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Train)),
            )
            // Craft Systems
            .add_systems(OnEnter(GameState::Craft), setup_craft_ui)
            .add_systems(OnExit(GameState::Craft), (cleanup_panel_ui, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_craft_ui,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Craft)),
            );
        app
            // Hunt Systems
            .add_systems(OnEnter(GameState::Hunt), setup_hunt_ui)
            .add_systems(OnExit(GameState::Hunt), (cleanup_panel_ui, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_hunt_ui,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Hunt)),
            )
            // Combat Systems
            .add_systems(
                OnEnter(GameState::Combat),
                (despawn::<PlayingCmp>, setup_combat_ui).chain(),
            )
            .add_systems(
                Update,
                (
                    update_playing_screen,
                    tooltip_follow_cursor_system,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Combat)),
            )
            .add_systems(
                OnExit(GameState::Combat),
                (despawn::<CombatCmp>, despawn::<TooltipNode>),
            )
            // Quest Systems
            .add_systems(OnEnter(GameState::Quest), setup_quest_ui)
            .add_systems(OnExit(GameState::Quest), (cleanup_panel_ui, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_quest_ui,
                    tooltip_follow_cursor_system,
                    tick_gold_toasts,
                    right_column_tooltip_system,
                    equip_slot_tooltip_system,
                )
                    .run_if(in_state(GameState::Quest)),
            )
            // Duel Systems
            .add_systems(OnEnter(GameState::Duel), setup_duel_ui)
            .add_systems(OnExit(GameState::Duel), (cleanup_panel_ui, despawn::<TooltipNode>));

        #[cfg(not(target_arch = "wasm32"))]
        app
            // Persistence
            .add_message::<SaveCharacterMsg>()
            .add_message::<LoadCharacterMsg>()
            .add_systems(
                Update,
                (
                    load_game,
                    save_game,
                    run_autosave.run_if(on_timer(Duration::from_secs(10))).in_set(InPlayingSet),
                ),
            );
    }
}
