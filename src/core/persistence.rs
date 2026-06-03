use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::localization::Language;
use crate::core::player::Character;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameSettings {
    pub audio_mode: AudioMode,
    pub auto_save: bool,
    pub language: Language,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioMode {
    Mute,
    SfxOnly,
    #[default]
    SfxAndMusic,
}

pub struct PersistenceManager;

impl PersistenceManager {
    fn save_dir() -> PathBuf {
        let mut path = PathBuf::from(".");
        path.push("saves");
        let _ = create_dir_all(&path);
        path
    }

    pub fn list_saves() -> Vec<String> {
        let dir = Self::save_dir();
        let mut saves = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json") && name != "settings.json" {
                                saves.push(name.replace(".json", ""));
                            }
                        }
                    }
                }
            }
        }
        saves
    }

    pub fn save_character(character: &Character) -> Result<(), String> {
        let mut path = Self::save_dir();
        path.push(format!("{}.json", character.name));
        
        let serialized = serde_json::to_string_pretty(character)
            .map_err(|e| format!("Failed to serialize character: {}", e))?;
        
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create save file: {}", e))?;
            
        file.write_all(serialized.as_bytes())
            .map_err(|e| format!("Failed to write save file: {}", e))?;
            
        Ok(())
    }

    pub fn load_character(name: &str) -> Result<Character, String> {
        let mut path = Self::save_dir();
        path.push(format!("{}.json", name));
        
        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open save file: {}", e))?;
            
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read save file: {}", e))?;
            
        let character: Character = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to deserialize character: {}", e))?;
            
        Ok(character)
    }

    pub fn save_settings(settings: &GameSettings) -> Result<(), String> {
        let mut path = Self::save_dir();
        path.push("settings.json");
        
        let serialized = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
            
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create settings file: {}", e))?;
            
        file.write_all(serialized.as_bytes())
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
            
        Ok(())
    }

    pub fn load_settings() -> GameSettings {
        let mut path = Self::save_dir();
        path.push("settings.json");
        
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(settings) = serde_json::from_str::<GameSettings>(&contents) {
                    return settings;
                }
            }
        }
        GameSettings::default()
    }
}
