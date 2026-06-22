use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

use crate::core::actions::shop::ShopInventory;
use crate::core::audio::ChangeAudioMsg;
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

fn save_to_bin(file_path: &str, data: &SaveAll) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    let buffer = encode_to_vec(data, standard()).expect("Failed to serialize data.");
    file.write_all(&buffer)?;

    Ok(())
}

fn load_from_bin(file_path: &str) -> io::Result<SaveAll> {
    let mut file = File::open(file_path)?;

    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    let (data, _) = decode_from_slice(&buffer, standard()).expect("Failed to deserialize data.");
    Ok(data)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_game(
    mut commands: Commands,
    mut load_game_msg: MessageReader<LoadCharacterMsg>,
    mut change_audio_msg: MessageWriter<ChangeAudioMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for _ in load_game_msg.read() {
        if let Some(file_path) = FileDialog::new().pick_file() {
            let file_path_str = file_path.to_string_lossy().to_string();
            let data = load_from_bin(&file_path_str).expect("Failed to load the game.");

            change_audio_msg.write(ChangeAudioMsg(Some(data.settings.audio)));

            commands.insert_resource(data.settings);
            commands.insert_resource(data.player);
            commands.insert_resource(data.shop_inventory);

            next_game_state.set(GameState::Playing);
            next_app_state.set(AppState::Game);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

            save_to_bin(&file_path_str, &data).expect("Failed to save the game.");
        }
    }
}

pub fn run_autosave(settings: Res<Settings>, mut save_game_msg: MessageWriter<SaveCharacterMsg>) {
    if settings.autosave {
        save_game_msg.write(SaveCharacterMsg(true));
    }
}
