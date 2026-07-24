//! Character and settings persistence for native builds and browser storage.

use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::mem::size_of;

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

const SAVE_MAGIC: &[u8; 8] = b"ARCANASV";
const SAVE_VERSION: u16 = 1;

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

/// Encodes the current versioned save envelope.
fn encode_save_bytes(data: &SaveAll) -> io::Result<Vec<u8>> {
    let payload = encode_to_vec(data, standard()).map_err(io::Error::other)?;
    let mut buffer = Vec::with_capacity(SAVE_MAGIC.len() + size_of::<u16>() + payload.len());
    buffer.extend_from_slice(SAVE_MAGIC);
    buffer.extend_from_slice(&SAVE_VERSION.to_le_bytes());
    buffer.extend_from_slice(&payload);
    Ok(buffer)
}

/// Decodes current saves and migrates the legacy unversioned representation.
fn decode_save_bytes(buffer: &[u8]) -> io::Result<SaveAll> {
    if let Some(versioned) = buffer.strip_prefix(SAVE_MAGIC) {
        let version_bytes = versioned.get(..size_of::<u16>()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "save version header is truncated")
        })?;
        let version = u16::from_le_bytes([version_bytes[0], version_bytes[1]]);
        let payload = &versioned[size_of::<u16>()..];

        return match version {
            SAVE_VERSION => decode_from_slice(payload, standard())
                .map(|(data, _)| data)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported save version {version}"),
            )),
        };
    }

    decode_from_slice(buffer, standard())
        .map(|(data, _)| data)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Serializes a complete save to the requested native file.
fn save_to_bin(file_path: &str, data: &SaveAll) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    let buffer = encode_save_bytes(data)?;
    file.write_all(&buffer)?;

    Ok(())
}

/// Deserializes a complete save while reporting corrupt or incompatible data as I/O errors.
fn load_from_bin(file_path: &str) -> io::Result<SaveAll> {
    let mut file = File::open(file_path)?;

    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;

    decode_save_bytes(&buffer)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::classes::Class;
    use crate::core::races::Race;

    /// Creates a compact save fixture for codec compatibility tests.
    fn fixture() -> SaveAll {
        let player = Player {
            name: "Legacy Hero".to_string(),
            class: Class::Warrior,
            race: Race::Orc,
            ..default()
        };

        SaveAll {
            settings: Settings::default(),
            player,
            shop_inventory: ShopInventory::default(),
        }
    }

    #[test]
    /// Verifies that legacy unversioned saves still decode after enum expansion.
    fn legacy_unversioned_save_still_loads() {
        let legacy = encode_to_vec(fixture(), standard()).expect("legacy fixture encodes");
        let decoded = decode_save_bytes(&legacy).expect("legacy fixture migrates");

        assert_eq!(decoded.player.name, "Legacy Hero");
        assert_eq!(decoded.player.class, Class::Warrior);
        assert_eq!(decoded.player.race, Race::Orc);
    }

    #[test]
    /// Verifies that current saves carry and decode the explicit version envelope.
    fn current_save_uses_versioned_envelope() {
        let bytes = encode_save_bytes(&fixture()).expect("current fixture encodes");
        assert!(bytes.starts_with(SAVE_MAGIC));

        let decoded = decode_save_bytes(&bytes).expect("current fixture decodes");
        assert_eq!(decoded.player.name, "Legacy Hero");
    }
}
