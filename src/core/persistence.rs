//! Character and settings persistence for native builds and browser storage.

use std::env::current_dir;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::mem::size_of;

use crate::core::actions::shop::ShopInventory;
use crate::core::audio::ChangeAudioMsg;
use crate::core::classes::{ClassSpecialization, PetChoice};
use crate::core::deities::Deity;
use crate::core::menu::systems::PendingGameStart;
use crate::core::monsters::Monster;
use crate::core::player::{AgeStage, Player, Sex, Training};
use crate::core::races::ElfHeritage;
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use bevy::prelude::*;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

const SAVE_MAGIC: &[u8; 8] = b"ARCANASV";
const SAVE_VERSION: u16 = 2;
const PREVIOUS_SAVE_VERSION: u16 = 1;

#[derive(Serialize, Deserialize)]
pub struct SaveAll {
    pub settings: Settings,
    pub player: Player,
    pub shop_inventory: ShopInventory,
}

#[derive(Serialize, Deserialize)]
struct LegacyPlayerV1 {
    name: String,
    sex: Sex,
    race: crate::core::races::Race,
    class: crate::core::classes::Class,
    stage: AgeStage,
    age: u32,
    xp: u32,
    ap: u32,
    missing_health: u32,
    missing_mana: u32,
    bonus_max_health: u32,
    bonus_max_mana: u32,
    strength: u32,
    dexterity: u32,
    constitution: u32,
    intelligence: u32,
    wisdom: u32,
    charisma: u32,
    abilities: Vec<String>,
    active_abilities: Vec<Option<String>>,
    perks: Vec<String>,
    pet: Option<Monster>,
    helmet: Option<String>,
    armor: Option<String>,
    gloves: Option<String>,
    boots: Option<String>,
    weapon_lh: Option<String>,
    weapon_rh: Option<String>,
    accessory: Option<String>,
    accessory2: Option<String>,
    equipped_consumables: Vec<String>,
    inventory: Vec<String>,
    gold: u32,
    training: Training,
}

#[derive(Serialize, Deserialize)]
struct LegacySaveAllV1 {
    settings: Settings,
    player: LegacyPlayerV1,
    shop_inventory: ShopInventory,
}

impl From<LegacySaveAllV1> for SaveAll {
    /// Migrates a version-one save by assigning defaults for the new identity choices.
    fn from(legacy: LegacySaveAllV1) -> Self {
        let LegacyPlayerV1 {
            name,
            sex,
            race,
            class,
            stage,
            age,
            xp,
            ap,
            missing_health,
            missing_mana,
            bonus_max_health,
            bonus_max_mana,
            strength,
            dexterity,
            constitution,
            intelligence,
            wisdom,
            charisma,
            abilities,
            active_abilities,
            perks,
            pet,
            helmet,
            armor,
            gloves,
            boots,
            weapon_lh,
            weapon_rh,
            accessory,
            accessory2,
            equipped_consumables,
            inventory,
            gold,
            training,
        } = legacy.player;
        let specialization = match class {
            crate::core::classes::Class::Mage(ajah) => ClassSpecialization::Mage(ajah),
            crate::core::classes::Class::Druid => {
                let choice = pet
                    .as_ref()
                    .and_then(|companion| match companion.name.as_str() {
                        "Owl" => Some(PetChoice::Owl),
                        "Rat" => Some(PetChoice::Rat),
                        "Snake" => Some(PetChoice::Snake),
                        "Weasel" => Some(PetChoice::Weasel),
                        "Fox" => Some(PetChoice::Fox),
                        "Raven" => Some(PetChoice::Raven),
                        _ => None,
                    })
                    .unwrap_or_default();
                ClassSpecialization::Druid(choice)
            },
            _ => class.default_specialization(),
        };
        Self {
            settings: legacy.settings,
            player: Player {
                name,
                sex,
                race,
                class,
                stage,
                age,
                xp,
                ap,
                missing_health,
                missing_mana,
                bonus_max_health,
                bonus_max_mana,
                strength,
                dexterity,
                constitution,
                intelligence,
                wisdom,
                charisma,
                abilities,
                active_abilities,
                perks,
                pet,
                helmet,
                armor,
                gloves,
                boots,
                weapon_lh,
                weapon_rh,
                accessory,
                accessory2,
                equipped_consumables,
                inventory,
                gold,
                training,
                elf_heritage: ElfHeritage::default(),
                specialization,
                deity: Deity::default(),
            },
            shop_inventory: legacy.shop_inventory,
        }
    }
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
            PREVIOUS_SAVE_VERSION => decode_from_slice::<LegacySaveAllV1, _>(payload, standard())
                .map(|(data, _)| data.into())
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported save version {version}"),
            )),
        };
    }

    decode_from_slice::<LegacySaveAllV1, _>(buffer, standard())
        .map(|(data, _)| data.into())
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

    /// Converts the current fixture into the exact version-one player layout.
    fn legacy_fixture() -> LegacySaveAllV1 {
        let current = fixture();
        let player = current.player;
        LegacySaveAllV1 {
            settings: current.settings,
            player: LegacyPlayerV1 {
                name: player.name,
                sex: player.sex,
                race: player.race,
                class: player.class,
                stage: player.stage,
                age: player.age,
                xp: player.xp,
                ap: player.ap,
                missing_health: player.missing_health,
                missing_mana: player.missing_mana,
                bonus_max_health: player.bonus_max_health,
                bonus_max_mana: player.bonus_max_mana,
                strength: player.strength,
                dexterity: player.dexterity,
                constitution: player.constitution,
                intelligence: player.intelligence,
                wisdom: player.wisdom,
                charisma: player.charisma,
                abilities: player.abilities,
                active_abilities: player.active_abilities,
                perks: player.perks,
                pet: player.pet,
                helmet: player.helmet,
                armor: player.armor,
                gloves: player.gloves,
                boots: player.boots,
                weapon_lh: player.weapon_lh,
                weapon_rh: player.weapon_rh,
                accessory: player.accessory,
                accessory2: player.accessory2,
                equipped_consumables: player.equipped_consumables,
                inventory: player.inventory,
                gold: player.gold,
                training: player.training,
            },
            shop_inventory: current.shop_inventory,
        }
    }

    #[test]
    /// Verifies that legacy unversioned saves still decode after enum expansion.
    fn legacy_unversioned_save_still_loads() {
        let legacy = encode_to_vec(legacy_fixture(), standard()).expect("legacy fixture encodes");
        let decoded = decode_save_bytes(&legacy).expect("legacy fixture migrates");

        assert_eq!(decoded.player.name, "Legacy Hero");
        assert_eq!(decoded.player.class, Class::Warrior);
        assert_eq!(decoded.player.race, Race::Orc);
        assert_eq!(decoded.player.deity, Deity::Tharos);
        assert!(decoded.player.specialization_is_valid());
    }

    #[test]
    /// Verifies that an explicitly versioned v1 save migrates to the new fields.
    fn version_one_save_migrates_identity_choices() {
        let payload = encode_to_vec(legacy_fixture(), standard()).expect("legacy fixture encodes");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(SAVE_MAGIC);
        bytes.extend_from_slice(&PREVIOUS_SAVE_VERSION.to_le_bytes());
        bytes.extend_from_slice(&payload);

        let decoded = decode_save_bytes(&bytes).expect("version one fixture migrates");
        assert_eq!(decoded.player.elf_heritage, ElfHeritage::High);
        assert_eq!(decoded.player.deity, Deity::Tharos);
        assert!(decoded.player.specialization_is_valid());
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
