use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::core::persistence::{AudioMode, GameSettings, PersistenceManager};

const DEFAULT_BGM_VOLUME_DB: f32 = -12.0;

#[derive(Resource, Default)]
pub struct AudioManager {
    pub current_settings: GameSettings,
    pub bgm_handle: Option<Handle<AudioInstance>>,
}

#[derive(Message)]
pub enum SoundEffect {
    Click,
    LevelUp,
    PhysicalHit,
    MagicCast,
    Heal,
    Shield,
    Victory,
    Defeat,
}

pub struct AudioSystemPlugin;

impl Plugin for AudioSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioManager::default())
            .add_message::<SoundEffect>()
            .add_systems(Startup, setup_audio)
            .add_systems(Update, (handle_settings_change, play_sfx));
    }
}

fn setup_audio(
    mut audio_manager: ResMut<AudioManager>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    let settings = PersistenceManager::load_settings();
    audio_manager.current_settings = settings;

    // Start background music looped if allowed
    if audio_manager.current_settings.audio_mode == AudioMode::SfxAndMusic {
        let music = asset_server.load("audio/music.ogg");
        let handle = audio
            .play(music)
            .looped()
            .with_volume(DEFAULT_BGM_VOLUME_DB)
            .handle();
        audio_manager.bgm_handle = Some(handle);
    }
}

fn handle_settings_change(
    mut audio_manager: ResMut<AudioManager>,
    audio: Res<Audio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    asset_server: Res<AssetServer>,
) {
    // Check if settings changed (e.g. from UI)
    let disk_settings = PersistenceManager::load_settings();
    if disk_settings.audio_mode != audio_manager.current_settings.audio_mode {
        audio_manager.current_settings = disk_settings;

        match audio_manager.current_settings.audio_mode {
            AudioMode::Mute | AudioMode::SfxOnly => {
                // Stop music if playing
                if let Some(instance) = audio_manager.bgm_handle.as_ref() {
                    if let Some(instance_mut) = audio_instances.get_mut(instance) {
                        instance_mut.stop(AudioTween::default());
                    }
                }
                audio_manager.bgm_handle = None;
            }
            AudioMode::SfxAndMusic => {
                // Start music if not playing
                if audio_manager.bgm_handle.is_none() {
                    let music = asset_server.load("audio/music.ogg");
                    let handle = audio
                        .play(music)
                        .looped()
                        .with_volume(DEFAULT_BGM_VOLUME_DB)
                        .handle();
                    audio_manager.bgm_handle = Some(handle);
                }
            }
        }
    }
}

fn play_sfx(
    mut sfx_reader: MessageReader<SoundEffect>,
    audio_manager: Res<AudioManager>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
) {
    // Only play SFX if not muted
    if audio_manager.current_settings.audio_mode == AudioMode::Mute {
        sfx_reader.clear();
        return;
    }

    for effect in sfx_reader.read() {
        let path = match effect {
            SoundEffect::Click => "audio/click.ogg",
            SoundEffect::LevelUp => "audio/victory.ogg", // Reusing victory.ogg for level up
            SoundEffect::PhysicalHit => "audio/explosion.ogg", // Physical strike
            SoundEffect::MagicCast => "audio/horn.ogg",       // Magical sound
            SoundEffect::Heal => "audio/victory.ogg",
            SoundEffect::Shield => "audio/warning.ogg",
            SoundEffect::Victory => "audio/victory.ogg",
            SoundEffect::Defeat => "audio/defeat.ogg",
        };
        audio.play(asset_server.load(path));
    }
}
