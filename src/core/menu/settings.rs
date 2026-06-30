use std::fmt::Debug;

use crate::core::assets::WorldAssets;
use crate::core::audio::{ChangeAudioMsg, PlayAudioMsg};
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::utils::add_text;
use crate::core::settings::{AudioSettings, Language, Settings};
use crate::core::ui::utils::SLIDER_WIDTH;
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::SystemCursorIcon;

#[derive(Component, Clone, Debug, PartialEq)]
pub enum SettingsBtn {
    English,
    Spanish,
    Dutch,
    Mute,
    Sound,
    Music,
    True,
    False,
}

#[derive(Component)]
pub struct VolumeSliderRow;

#[derive(Component)]
pub struct VolumeSliderTrack;

#[derive(Component)]
pub struct VolumeSliderHandle;

#[derive(Component)]
pub struct VolumeSliderFill;

#[derive(Component)]
pub struct VolumeSliderText;

fn match_setting(button: &SettingsBtn, settings: &Settings) -> bool {
    match button {
        SettingsBtn::English => settings.language == Language::English,
        SettingsBtn::Spanish => settings.language == Language::Spanish,
        SettingsBtn::Dutch => settings.language == Language::Dutch,
        SettingsBtn::Mute => settings.audio == AudioSettings::Mute,
        SettingsBtn::Sound => settings.audio == AudioSettings::Sfx,
        SettingsBtn::Music => settings.audio == AudioSettings::Music,
        SettingsBtn::True => settings.autosave,
        SettingsBtn::False => !settings.autosave,
    }
}

pub fn recolor_label<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(On<Pointer<E>>, Query<(&mut BackgroundColor, &SettingsBtn)>, ResMut<Settings>) {
    move |ev, mut bgcolor_q, settings| {
        if let Ok((mut bgcolor, button)) = bgcolor_q.get_mut(ev.entity) {
            // Don't change the color of selected buttons
            if !match_setting(button, &settings) {
                bgcolor.0 = color;
            }
        };
    }
}

pub fn on_click_label_button(
    event: On<Pointer<Click>>,
    mut btn_q: Query<(&mut BackgroundColor, &SettingsBtn)>,
    mut settings: ResMut<Settings>,
    mut change_audio_msg: MessageWriter<ChangeAudioMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    play_audio_msg.write(PlayAudioMsg::new("button"));
    match btn_q.get(event.entity).unwrap().1 {
        SettingsBtn::English => settings.language = Language::English,
        SettingsBtn::Spanish => settings.language = Language::Spanish,
        SettingsBtn::Dutch => settings.language = Language::Dutch,
        SettingsBtn::Mute => {
            settings.audio = AudioSettings::Mute;
            change_audio_msg.write(ChangeAudioMsg(Some(AudioSettings::Mute)));
        },
        SettingsBtn::Sound => {
            settings.audio = AudioSettings::Sfx;
            change_audio_msg.write(ChangeAudioMsg(Some(AudioSettings::Sfx)));
        },
        SettingsBtn::Music => {
            settings.audio = AudioSettings::Music;
            change_audio_msg.write(ChangeAudioMsg(Some(AudioSettings::Music)));
        },
        SettingsBtn::True => settings.autosave = true,
        SettingsBtn::False => settings.autosave = false,
    }

    // Reset the color of the other buttons
    for (mut bgcolor, setting) in &mut btn_q {
        if !match_setting(setting, &settings) {
            bgcolor.0 = NORMAL_BUTTON_COLOR;
        }
    }
}

pub fn spawn_label(
    parent: &mut ChildSpawnerCommands,
    key: &str,
    buttons: Vec<SettingsBtn>,
    settings: &Settings,
    assets: &WorldAssets,
    localization: &Localization,
) {
    let title = localization.get(key, settings.language);
    parent.spawn((
        add_text(title, "bold", BUTTON_TEXT_SIZE, assets),
        TextColor(BUTTON_TEXT_COLOR),
        LocalizedText(key.to_string()),
    ));

    parent
        .spawn(Node {
            width: percent(100.),
            height: Val::Vh(7.22),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            margin: UiRect::bottom(Val::Vh(1.67)),
            ..default()
        })
        .with_children(|parent| {
            for item in buttons.iter() {
                let key = item.to_lowername();
                let label = localization.get(&key, settings.language);
                parent
                    .spawn((
                        Node {
                            width: Val::Vh(13.33),
                            height: Val::Vh(5.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::horizontal(Val::Vh(0.89)),
                            border: UiRect::all(Val::Vh(0.22)),
                            border_radius: BorderRadius::all(Val::Vh(0.44)),
                            ..default()
                        },
                        BackgroundColor(if match_setting(item, settings) {
                            PRESSED_BUTTON_COLOR
                        } else {
                            NORMAL_BUTTON_COLOR
                        }),
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        item.clone(),
                    ))
                    .observe(recolor_label::<Over>(HOVERED_BUTTON_COLOR))
                    .observe(recolor_label::<Out>(NORMAL_BUTTON_COLOR))
                    .observe(recolor_label::<Press>(PRESSED_BUTTON_COLOR))
                    .observe(recolor_label::<Release>(HOVERED_BUTTON_COLOR))
                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                    .observe(on_click_label_button)
                    .with_children(|parent| {
                        parent.spawn((
                            add_text(label, "bold", LABEL_TEXT_SIZE, assets),
                            TextColor(BUTTON_TEXT_COLOR),
                            LocalizedText(key),
                        ));
                    });
            }
        });
}

/// Spawns the master-volume slider shown beneath the audio buttons. The whole
/// row is hidden while audio is muted.
pub fn spawn_volume_slider(
    parent: &mut ChildSpawnerCommands,
    settings: &Settings,
    assets: &WorldAssets,
    localization: &Localization,
) {
    let lang = settings.language;
    let frac = settings.volume.clamp(0., 1.);

    parent
        .spawn((
            Node {
                width: percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::bottom(Val::Vh(1.67)),
                display: if settings.audio == AudioSettings::Mute {
                    Display::None
                } else {
                    Display::Flex
                },
                ..default()
            },
            VolumeSliderRow,
        ))
        .with_children(|parent| {
            parent.spawn((
                add_text(
                    localization.get("general.volume", lang),
                    "bold",
                    BUTTON_TEXT_SIZE,
                    assets,
                ),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText("general.volume".to_string()),
            ));

            parent
                .spawn(Node {
                    width: percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(14.),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(SLIDER_WIDTH),
                                height: Val::Px(30.),
                                position_type: PositionType::Relative,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Button,
                            Interaction::default(),
                            Pickable::default(),
                            RelativeCursorPosition::default(),
                            BackgroundColor(Color::srgba(0., 0., 0., 0.01)),
                            VolumeSliderTrack,
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(handle_volume_track_click)
                        .observe(handle_volume_drag)
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.),
                                    top: Val::Px(12.),
                                    width: percent(100.),
                                    height: Val::Px(6.),
                                    border_radius: BorderRadius::all(Val::Px(3.)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba_u8(60, 60, 80, 200)),
                                Pickable::IGNORE,
                            ));
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.),
                                    top: Val::Px(12.),
                                    width: Val::Px(frac * SLIDER_WIDTH),
                                    height: Val::Px(6.),
                                    border_radius: BorderRadius::all(Val::Px(3.)),
                                    ..default()
                                },
                                BackgroundColor(PRESSED_BUTTON_COLOR),
                                Pickable::IGNORE,
                                VolumeSliderFill,
                            ));
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(24.),
                                    height: Val::Px(24.),
                                    top: Val::Px(3.),
                                    left: Val::Px(frac * SLIDER_WIDTH - 12.),
                                    border: UiRect::all(Val::Px(2.)),
                                    border_radius: BorderRadius::all(Val::Px(12.)),
                                    ..default()
                                },
                                BackgroundColor(BUTTON_TEXT_COLOR),
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                Pickable::IGNORE,
                                VolumeSliderHandle,
                            ));
                        });

                    parent.spawn((
                        add_text(
                            format!("{}%", (frac * 100.).round() as i32),
                            "bold",
                            LABEL_TEXT_SIZE,
                            assets,
                        ),
                        TextColor(BUTTON_TEXT_COLOR),
                        VolumeSliderText,
                    ));
                });
        });
}

fn set_volume_visuals(
    frac: f32,
    settings: &mut Settings,
    handle_q: &mut Query<&mut Node, (With<VolumeSliderHandle>, Without<VolumeSliderFill>)>,
    fill_q: &mut Query<&mut Node, (With<VolumeSliderFill>, Without<VolumeSliderHandle>)>,
    text_q: &mut Query<&mut Text, With<VolumeSliderText>>,
) {
    let frac = frac.clamp(0., 1.);
    settings.volume = frac;
    for mut h in handle_q.iter_mut() {
        h.left = Val::Px(frac * SLIDER_WIDTH - 12.);
    }
    for mut f in fill_q.iter_mut() {
        f.width = Val::Px(frac * SLIDER_WIDTH);
    }
    for mut t in text_q.iter_mut() {
        t.0 = format!("{}%", (frac * 100.).round() as i32);
    }
}

pub fn handle_volume_track_click(
    ev: On<Pointer<Click>>,
    track_q: Query<&RelativeCursorPosition, With<VolumeSliderTrack>>,
    mut settings: ResMut<Settings>,
    mut handle_q: Query<&mut Node, (With<VolumeSliderHandle>, Without<VolumeSliderFill>)>,
    mut fill_q: Query<&mut Node, (With<VolumeSliderFill>, Without<VolumeSliderHandle>)>,
    mut text_q: Query<&mut Text, With<VolumeSliderText>>,
) {
    let Ok(rel) = track_q.get(ev.entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };
    let frac = (normalized.x + 0.5).clamp(0., 1.);
    set_volume_visuals(frac, &mut settings, &mut handle_q, &mut fill_q, &mut text_q);
}

pub fn handle_volume_drag(
    ev: On<Pointer<Drag>>,
    track_q: Query<&RelativeCursorPosition, With<VolumeSliderTrack>>,
    mut settings: ResMut<Settings>,
    mut handle_q: Query<&mut Node, (With<VolumeSliderHandle>, Without<VolumeSliderFill>)>,
    mut fill_q: Query<&mut Node, (With<VolumeSliderFill>, Without<VolumeSliderHandle>)>,
    mut text_q: Query<&mut Text, With<VolumeSliderText>>,
) {
    let Ok(rel) = track_q.get(ev.entity) else {
        return;
    };
    let Some(normalized) = rel.normalized else {
        return;
    };
    let frac = (normalized.x + 0.5).clamp(0., 1.);
    set_volume_visuals(frac, &mut settings, &mut handle_q, &mut fill_q, &mut text_q);
}

/// Keeps the volume row hidden while muted and visible otherwise.
pub fn update_volume_slider_visibility(
    settings: Res<Settings>,
    mut row_q: Query<&mut Node, With<VolumeSliderRow>>,
) {
    let muted = settings.audio == AudioSettings::Mute;
    for mut node in &mut row_q {
        let target = if muted {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != target {
            node.display = target;
        }
    }
}

/// Handle arrow keys to adjust volume by 10% increments.
pub fn handle_volume_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<Settings>,
    mut handle_q: Query<&mut Node, (With<VolumeSliderHandle>, Without<VolumeSliderFill>)>,
    mut fill_q: Query<&mut Node, (With<VolumeSliderFill>, Without<VolumeSliderHandle>)>,
    mut text_q: Query<&mut Text, With<VolumeSliderText>>,
) {
    let mut changed = false;

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        settings.volume = (settings.volume + 0.1).min(1.0);
        changed = true;
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        settings.volume = (settings.volume - 0.1).max(0.0);
        changed = true;
    }

    if changed {
        set_volume_visuals(settings.volume, &mut settings, &mut handle_q, &mut fill_q, &mut text_q);
    }
}
