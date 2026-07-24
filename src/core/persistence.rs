//! Character and settings persistence for native builds and browser storage.

use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

use crate::core::actions::shop::ShopInventory;
use crate::core::audio::ChangeAudioMsg;
use crate::core::menu::systems::PendingGameStart;
use crate::core::player::Player;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use bevy::prelude::*;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveAll {
    pub settings: Settings,
    pub player: Player,
    pub shop_inventory: ShopInventory,
}

#[derive(Message)]
pub struct LoadCharacterMsg;

#[derive(Message)]
pub struct SaveCharacterMsg(pub bool);

/// Serializes a complete save to the requested native file.
fn save_to_bin(file_path: &str, data: &SaveAll) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    let buffer = encode_to_vec(data, standard()).map_err(io::Error::other)?;
    file.write_all(&buffer)?;

    Ok(())
}

/// Deserializes a complete save while reporting corrupt or incompatible data as I/O errors.
fn load_from_bin(file_path: &str) -> io::Result<SaveAll> {
    let mut file = File::open(file_path)?;

    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    let (data, _) = decode_from_slice(&buffer, standard())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(data)
}

#[cfg(not(target_arch = "wasm32"))]
/// Loads game.
pub fn load_game(
    mut commands: Commands,
    mut load_game_msg: MessageReader<LoadCharacterMsg>,
    mut change_audio_msg: MessageWriter<ChangeAudioMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for _ in load_game_msg.read() {
        if let Some(file_path) = FileDialog::new().pick_file() {
            let file_path_str = file_path.to_string_lossy().to_string();
            let data = match load_from_bin(&file_path_str) {
                Ok(data) => data,
                Err(error) => {
                    error!("Failed to load save {}: {error}", file_path.display());
                    continue;
                },
            };

            change_audio_msg.write(ChangeAudioMsg(Some(data.settings.audio)));

            commands.insert_resource(data.settings);
            commands.insert_resource(data.player);
            commands.insert_resource(data.shop_inventory);
            commands.insert_resource(PendingGameStart {
                target_game_state: GameState::Playing,
            });

            next_app_state.set(AppState::Loading);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Saves game.
pub fn save_game(
    mut save_game_msg: MessageReader<SaveCharacterMsg>,
    settings: Res<Settings>,
    player: Res<Player>,
    shop_inventory: Res<ShopInventory>,
) {
    for msg in save_game_msg.read() {
        let file_path = if msg.0 {
            let path = current_dir().expect("Failed to get current directory.");
            Some(path.join(&player.name))
        } else {
            FileDialog::new().set_file_name(player.name.clone()).save_file()
        };

        if let Some(mut file_path) = file_path {
            if !file_path.extension().map(|e| e == "bin").unwrap_or(false) {
                file_path.set_extension("bin");
            }

            let file_path_str = file_path.to_string_lossy().to_string();
            let data = SaveAll {
                settings: settings.clone(),
                player: player.clone(),
                shop_inventory: shop_inventory.clone(),
            };

            if let Err(error) = save_to_bin(&file_path_str, &data) {
                error!("Failed to save game to {}: {error}", file_path.display());
            }
        }
    }
}

/// Performs the run autosave operation.
pub fn run_autosave(settings: Res<Settings>, mut save_game_msg: MessageWriter<SaveCharacterMsg>) {
    if settings.autosave {
        save_game_msg.write(SaveCharacterMsg(true));
    }
}
