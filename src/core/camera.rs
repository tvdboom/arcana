//! Creation and scaling behavior for the game's primary 2D camera.

use bevy::prelude::*;

#[derive(Component)]
pub struct MainCamera;

/// Sets up camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off, MainCamera));
}
