#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};
use bevy_egui::PrimaryEguiContext;

use crate::core::GamePlugin;

pub const TITLE: &str = "Arcana";

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.into(),
                    mode: WindowMode::Windowed,
                    position: WindowPosition::Automatic,
                    resolution: WindowResolution::new(1600, 900),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(GamePlugin)
    .add_systems(Startup, setup_scene);

    app.run();
}

fn setup_scene(mut commands: Commands) {
    // Spawn 2D camera and explicitly mark it as the primary egui context camera.
    commands.spawn((Camera2d, PrimaryEguiContext));
}
