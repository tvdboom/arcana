use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy_kira_audio::prelude::*;

use crate::core::assets::WorldAssets;
use crate::core::constants::{NORMAL_BUTTON_COLOR, PRESSED_BUTTON_COLOR};
use crate::core::menu::settings::SettingsBtn;
use crate::core::settings::{AudioSettings, Settings};
use crate::core::utils::cursor;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PlayingAudio(pub HashMap<String, Handle<AudioInstance>>);

impl PlayingAudio {
    pub const DEFAULT_VOLUME: f32 = 0.6;
    pub const DEFAULT_MUSIC_VOLUME: f32 = -20.;
    pub const TWEEN: AudioTween = AudioTween::new(Duration::from_secs(2), AudioEasing::OutPowi(2));
}

#[derive(Message, Clone)]
pub struct PlayAudioMsg {
    pub name: String,
    pub volume: f32,
    pub is_background: bool,
}

impl PlayAudioMsg {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            volume: PlayingAudio::DEFAULT_VOLUME,
            is_background: false,
        }
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn background(mut self) -> Self {
        self.is_background = true;
        self
    }
}

#[derive(Message, Clone)]
pub struct PauseAudioMsg {
    pub name: String,
}

impl PauseAudioMsg {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[derive(Message, Clone)]
pub struct StopAudioMsg {
    pub name: String,
}

impl StopAudioMsg {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[derive(Message, Clone)]
pub struct MuteAudioMsg;

#[derive(Component)]
pub struct MusicBtnCmp;

#[derive(Message, Deref)]
pub struct ChangeAudioMsg(pub Option<AudioSettings>);

pub fn setup_audio(mut commands: Commands, assets: Local<WorldAssets>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(5.),
                height: percent(5.),
                right: percent(0.),
                top: percent(2.),
                ..default()
            },
            GlobalZIndex(10000),
        ))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .with_children(|parent| {
            parent.spawn((ImageNode::new(assets.image("sound")), MusicBtnCmp)).observe(
                |_: On<Pointer<Click>>, mut commands: Commands| {
                    commands.queue(|w: &mut World| {
                        w.write_message(PlayAudioMsg::new("button"));
                        w.write_message(ChangeAudioMsg(None));
                    })
                },
            );
        });
}

pub fn update_audio(
    mut change_audio_msg: MessageReader<ChangeAudioMsg>,
    mut btn_q: Query<&mut ImageNode, With<MusicBtnCmp>>,
    mut settings_btn: Query<(&mut BackgroundColor, &SettingsBtn)>,
    mut settings: ResMut<Settings>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut pause_audio_msg: MessageWriter<PauseAudioMsg>,
    mut stop_audio_msg: MessageWriter<StopAudioMsg>,
    mut mute_audio_msg: MessageWriter<MuteAudioMsg>,
    assets: Res<WorldAssets>,
) {
    for msg in change_audio_msg.read() {
        settings.audio = msg.unwrap_or(match settings.audio {
            AudioSettings::Mute => AudioSettings::Sfx,
            AudioSettings::Sfx => AudioSettings::Music,
            AudioSettings::Music => AudioSettings::Mute,
        });

        if let Ok(mut node) = btn_q.single_mut() {
            node.image = match settings.audio {
                AudioSettings::Mute => {
                    mute_audio_msg.write(MuteAudioMsg);
                    assets.image("mute")
                },
                AudioSettings::Sfx => {
                    pause_audio_msg.write(PauseAudioMsg::new("music"));
                    stop_audio_msg.write(StopAudioMsg::new("drums"));
                    assets.image("sound")
                },
                AudioSettings::Music => {
                    play_audio_msg.write(
                        PlayAudioMsg::new("music")
                            .volume(PlayingAudio::DEFAULT_MUSIC_VOLUME)
                            .background(),
                    );
                    assets.image("music")
                },
            };
        }

        for (mut bgcolor, setting) in &mut settings_btn {
            if matches!(setting, SettingsBtn::Mute | SettingsBtn::Sound | SettingsBtn::Music) {
                bgcolor.0 = if (*setting == SettingsBtn::Mute
                    && settings.audio == AudioSettings::Mute)
                    || (*setting == SettingsBtn::Sound && settings.audio == AudioSettings::Sfx)
                    || (*setting == SettingsBtn::Music && settings.audio == AudioSettings::Music)
                {
                    PRESSED_BUTTON_COLOR
                } else {
                    NORMAL_BUTTON_COLOR
                };
            }
        }
    }
}

pub fn toggle_audio(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut change_audio_msg: MessageWriter<ChangeAudioMsg>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        change_audio_msg.write(ChangeAudioMsg(None));
    }
}

pub fn play_music(mut play_audio_msg: MessageWriter<PlayAudioMsg>) {
    play_audio_msg
        .write(PlayAudioMsg::new("music").volume(PlayingAudio::DEFAULT_MUSIC_VOLUME).background());
}

pub fn play_audio(
    mut play_audio_msg: MessageReader<PlayAudioMsg>,
    mut playing_audio: ResMut<PlayingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    settings: Res<Settings>,
    audio: Res<Audio>,
    assets: Res<WorldAssets>,
) {
    for msg in play_audio_msg.read() {
        if settings.audio != AudioSettings::Mute {
            let mut new_sound = false;

            if let Some(handle) = playing_audio.get(&msg.name) {
                if let Some(mut instance) = audio_instances.get_mut(handle) {
                    if msg.is_background
                        && matches!(
                            instance.state(),
                            PlaybackState::Paused { .. } | PlaybackState::Pausing { .. }
                        )
                    {
                        if settings.audio != AudioSettings::Sfx {
                            instance.resume(PlayingAudio::TWEEN);
                        }
                    } else if !msg.is_background
                        || !matches!(
                            instance.state(),
                            PlaybackState::Playing { .. }
                                | PlaybackState::WaitingToResume { .. }
                                | PlaybackState::Resuming { .. }
                        )
                    {
                        new_sound = true; // Audio finished playing
                    }
                } else {
                    new_sound = true; // Handle exists but instance was cleaned up / finished
                }
            } else if msg.is_background {
                if settings.audio != AudioSettings::Sfx {
                    playing_audio.insert(
                        msg.name.clone(),
                        audio
                            .play(assets.audio(&msg.name))
                            .fade_in(PlayingAudio::TWEEN)
                            .with_volume(msg.volume)
                            .looped()
                            .handle(),
                    );
                }
            } else {
                new_sound = true;
            }

            if new_sound {
                playing_audio.insert(
                    msg.name.clone(),
                    audio.play(assets.audio(&msg.name)).with_volume(msg.volume).handle(),
                );
            }
        }
    }
}

pub fn pause_audio(
    mut pause_audio_msg: MessageReader<PauseAudioMsg>,
    playing_audio: Res<PlayingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for msg in pause_audio_msg.read() {
        if let Some(handle) = playing_audio.get(&msg.name) {
            if let Some(mut instance) = audio_instances.get_mut(handle) {
                instance.pause(PlayingAudio::TWEEN);
            }
        }
    }
}

pub fn stop_audio(
    mut stop_audio_msg: MessageReader<StopAudioMsg>,
    mut playing_audio: ResMut<PlayingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for msg in stop_audio_msg.read() {
        if let Some(handle) = playing_audio.get(&msg.name) {
            if let Some(mut instance) = audio_instances.get_mut(handle) {
                instance.stop(PlayingAudio::TWEEN);
                playing_audio.remove(&msg.name);
            }
        }
    }
}

pub fn mute_audio(
    mut mute_audio_msg: MessageReader<MuteAudioMsg>,
    playing_audio: Res<PlayingAudio>,
    mut pause_audio_msg: MessageWriter<PauseAudioMsg>,
    mut stop_audio_msg: MessageWriter<StopAudioMsg>,
) {
    for _ in mute_audio_msg.read() {
        for name in playing_audio.keys() {
            if *name == "music" {
                pause_audio_msg.write(PauseAudioMsg::new(name));
            } else {
                stop_audio_msg.write(StopAudioMsg::new(name));
            }
        }
    }
}
