use std::fmt::Debug;

use crate::core::assets::WorldAssets;
use crate::core::audio::ChangeAudioMsg;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::utils::add_text;
use crate::core::settings::{AudioSettings, Language, Settings};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

#[derive(Component, Clone, Debug, PartialEq)]
pub enum SettingsBtn {
    English,
    Spanish,
    Mute,
    Sound,
    Music,
    True,
    False,
}

fn match_setting(button: &SettingsBtn, settings: &Settings) -> bool {
    match button {
        SettingsBtn::English => settings.language == Language::English,
        SettingsBtn::Spanish => settings.language == Language::Spanish,
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
) {
    match btn_q.get(event.entity).unwrap().1 {
        SettingsBtn::English => settings.language = Language::English,
        SettingsBtn::Spanish => settings.language = Language::Spanish,
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
            height: percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            padding: UiRect {
                top: percent(3.),
                left: percent(5.),
                right: percent(5.),
                bottom: percent(7.),
            },
            ..default()
        })
        .with_children(|parent| {
            for item in buttons.iter() {
                let key = item.to_lowername();
                let label = localization.get(&key, settings.language);
                parent
                    .spawn((
                        Node {
                            width: percent(30.),
                            height: percent(100.),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::all(percent(1.)),
                            border: UiRect::all(Val::Px(2.)),
                            border_radius: BorderRadius::all(Val::Px(4.)),
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
