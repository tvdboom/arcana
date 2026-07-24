#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Arcana's executable entry point and top-level Bevy application setup.
//!
//! This crate wires together the game plugins, runtime assets, window, and logging.

mod asset_pak;
mod core;
mod utils;

use bevy::asset::AssetMetaCheck;
#[cfg(target_os = "windows")]
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};
#[cfg(target_os = "windows")]
use bevy::winit::WINIT_WINDOWS;
use bevy_kira_audio::AudioPlugin;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::panic;
use std::sync::Mutex;

use crate::core::GamePlugin;
use crate::utils::NameFromEnum;

pub const TITLE: &str = "Arcana";

#[allow(dead_code)]
static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
/// Returns the generated asset directory used by native development builds.
fn asset_file_path() -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").to_string_lossy().into_owned()
}

#[cfg(any(not(debug_assertions), target_arch = "wasm32"))]
/// Returns the conventional asset directory used by packaged builds.
fn asset_file_path() -> String {
    "assets".to_string()
}

/// Runs the main entry point.
fn main() {
    #[cfg(not(debug_assertions))]
    init_panic_logger();

    let mut app = App::new();

    // Serve assets from the bundled `assets.pak` archive. Must run before
    // `AssetPlugin` (added via `DefaultPlugins`) registers the default source.
    asset_pak::register(&mut app);

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.into(),
                    mode: WindowMode::Windowed,
                    position: WindowPosition::Automatic,
                    resolution: WindowResolution::new(1600, 900),

                    // Tells Wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,

                    // Don't override browser's default behavior (ctrl+5, etc...)
                    prevent_default_event_handling: true,

                    ..default()
                }),
                ..default()
            })
            // Disable loading of asset meta since that fails on itch.io
            .set(AssetPlugin {
                file_path: asset_file_path(),
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(AudioPlugin)
    .add_plugins(GamePlugin);

    #[cfg(target_os = "windows")]
    app.add_systems(Startup, set_window_icon);

    app.run();
}

#[allow(dead_code)]
/// Performs the init panic logger operation.
fn init_panic_logger() {
    panic::set_hook(Box::new(|info| {
        let mut guard = LOG_FILE.lock().unwrap();

        if guard.is_none() {
            *guard = OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}-logs.txt", TITLE.to_lowername()))
                .ok();
        }

        if let Some(file) = guard.as_mut() {
            let _ = writeln!(file, "=== PANIC ===");
            let _ = writeln!(file, "{}", info);
            let _ = writeln!(file);
        }
    }));
}

#[cfg(target_os = "windows")]
/// Performs the set window icon operation.
fn set_window_icon(_: NonSendMarker) {
    use winit::window::Icon;

    let image = image::load_from_memory(include_bytes!("../assets-src/images/icons/favicon.png"))
        .expect("embedded window icon must be a valid image")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    let icon =
        Icon::from_rgba(rgba, width, height).expect("embedded window icon must be valid RGBA");

    WINIT_WINDOWS.with_borrow(|windows| {
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    });
}

#[cfg(all(test, debug_assertions, not(target_arch = "wasm32")))]
mod tests {
    use super::asset_file_path;
    use std::path::Path;

    #[test]
    /// Verifies that native debug builds resolve the generated repository assets.
    fn native_debug_asset_directory_exists() {
        assert!(Path::new(&asset_file_path()).is_dir());
    }
}
