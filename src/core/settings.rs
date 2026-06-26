use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    English,
    Spanish,
    Dutch,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub enum AudioSettings {
    Mute,
    #[default]
    Sfx,
    Music,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub language: Language,
    pub audio: AudioSettings,
    pub autosave: bool,
    #[serde(default = "default_volume")]
    pub volume: f32,
}

fn default_volume() -> f32 {
    1.0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::default(),
            audio: AudioSettings::default(),
            autosave: false,
            volume: default_volume(),
        }
    }
}
