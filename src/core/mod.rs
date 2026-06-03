use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_kira_audio::AudioPlugin;
use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin};

pub mod states;
pub mod player;
pub mod pet;
pub mod persistence;
pub mod audio;
pub mod localization;
pub mod compat;
pub mod network;
pub mod systems;
pub mod ui;
pub mod rules;

use crate::core::states::AppState;
use crate::core::audio::AudioSystemPlugin;
use crate::core::localization::Localizer;
use crate::core::persistence::PersistenceManager;
use crate::core::systems::network_sync::NetworkSyncPlugin;
use crate::core::ui::menu::MenuUiPlugin;
use crate::core::ui::planning::PlanningUiPlugin;
use crate::core::ui::combat::CombatUiPlugin;
use crate::core::ui::duels::DuelsUiPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Add external dependencies plugins
        app.add_plugins(EguiPlugin::default())
            .add_plugins(AudioPlugin)
            .add_plugins((
                RenetServerPlugin,
                NetcodeServerPlugin,
                RenetClientPlugin,
                NetcodeClientPlugin,
            ));

        // Insert App State
        app.init_state::<AppState>();

        // Insert resources
        app.init_resource::<Localizer>();
        app.add_systems(Startup, setup_localizer_language);

        // Add custom plugins
        app.add_plugins((
            AudioSystemPlugin,
            NetworkSyncPlugin,
            MenuUiPlugin,
            PlanningUiPlugin,
            CombatUiPlugin,
            DuelsUiPlugin,
        ));
    }
}

fn setup_localizer_language(mut localizer: ResMut<Localizer>) {
    localizer.set_language(PersistenceManager::load_settings().language);
}
