mod assets;
mod audio;
mod camera;
pub mod classes;
mod constants;
pub mod catalog;
pub mod localization;
mod menu;
#[cfg(not(target_arch = "wasm32"))]
mod persistence;
mod pets;
mod player;
mod races;
mod settings;
mod states;
mod systems;
mod ui;
mod utils;

use crate::core::assets::WorldAssets;
use crate::core::audio::*;
use crate::core::camera::*;
use crate::core::localization::{update_localized_text, Localization};
use crate::core::menu::buttons::MenuCmp;
use crate::core::menu::systems::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::persistence::{
    load_game, run_autosave, save_game, LoadCharacterMsg, SaveCharacterMsg,
};
use crate::core::player::Player;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use crate::core::systems::*;
use crate::core::ui::creation::*;
use crate::core::ui::playing::*;
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
            // Resources
            .init_resource::<WorldAssets>()
            .init_resource::<PlayingAudio>()
            .init_resource::<Localization>()
            .init_resource::<Settings>()
            .init_resource::<Player>();

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
        for state in AppState::iter().filter(|s| *s != AppState::Game) {
            app.add_systems(OnEnter(state), (reset_cursor, setup_menu))
                .add_systems(OnExit(state), despawn::<MenuCmp>);
        }
        for state in GameState::iter() {
            app.add_systems(OnEnter(state), reset_cursor);
        }
        app.add_systems(Update, start_new_game_message.run_if(not(in_state(AppState::Game))));

        app
            // Utilities
            .add_systems(
                Update,
                (
                    check_keys_menu,
                    update_localized_text.run_if(resource_changed::<Settings>),
                    update_playing_screen.run_if(resource_changed::<Settings>),
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
                (setup_playing_screen, rebuild_playing_lists).chain(),
            )
            .add_systems(OnExit(GameState::Playing), despawn::<TooltipNode>)
            .add_systems(OnExit(AppState::Game), (despawn::<PlayingCmp>, despawn::<TooltipNode>))
            .add_systems(
                Update,
                (
                    update_playing_screen,
                    handle_equipment_interactions,
                    scroll_system,
                    update_right_scrollbar_system.after(scroll_system),
                    equip_slot_tooltip_system,
                    info_tooltip_system,
                    tooltip_follow_cursor_system,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                rebuild_playing_lists
                    .run_if(in_state(AppState::Game))
                    .run_if(resource_changed::<Player>.or_else(resource_changed::<Settings>)),
            )
            .add_systems(OnEnter(GameState::GameMenu), setup_game_menu)
            .add_systems(OnExit(GameState::GameMenu), despawn::<MenuCmp>)
            .add_systems(OnEnter(GameState::Settings), setup_game_settings)
            .add_systems(OnExit(GameState::Settings), despawn::<MenuCmp>);

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
