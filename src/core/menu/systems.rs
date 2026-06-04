use bevy::prelude::*;

use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::buttons::*;
use crate::core::menu::settings::{spawn_label, SettingsBtn};
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};

#[derive(Message)]
pub struct StartNewCharacterMsg;

pub fn setup_menu(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    let lang = settings.language;
    commands
        .spawn((
            add_root_node(true),
            ImageNode::new(assets.image("bg")).with_mode(NodeImageMode::Stretch),
            MenuCmp,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::top(percent(12.)),
                    ..default()
                })
                .with_children(|parent| match app_state.get() {
                    AppState::MainMenu => {
                        spawn_menu_button(parent, MenuBtn::NewCharacter, &assets, &localization, lang);
                        spawn_menu_button(parent, MenuBtn::LoadCharacter, &assets, &localization, lang);
                        spawn_menu_button(parent, MenuBtn::Settings, &assets, &localization, lang);
                        #[cfg(not(target_arch = "wasm32"))]
                        spawn_menu_button(parent, MenuBtn::Quit, &assets, &localization, lang);
                    },
                    AppState::Settings => {
                        parent
                            .spawn((Node {
                                width: percent(40.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },))
                            .with_children(|parent| {
                                spawn_label(
                                    parent,
                                    "language",
                                    vec![SettingsBtn::English, SettingsBtn::Spanish],
                                    &settings,
                                    &assets,
                                    &localization,
                                );
                                spawn_label(
                                    parent,
                                    "audio",
                                    vec![SettingsBtn::Mute, SettingsBtn::Sound, SettingsBtn::Music],
                                    &settings,
                                    &assets,
                                    &localization,
                                );
                                spawn_label(
                                    parent,
                                    "autosave",
                                    vec![SettingsBtn::True, SettingsBtn::False],
                                    &settings,
                                    &assets,
                                    &localization,
                                );
                            });

                        spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);
                    },
                    _ => (),
                });

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: percent(3.),
                    bottom: percent(6.),
                    ..default()
                })
                .with_children(|parent| {
                    let credit = localization.get("created_by", lang);
                    parent.spawn((
                        add_text(credit, "medium", TITLE_TEXT_SIZE, &assets),
                        LocalizedText("created_by".to_string()),
                    ));
                });
        });
}

pub fn setup_game_menu(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    let lang = settings.language;
    commands.spawn((add_root_node(true), MenuCmp)).with_children(|parent| {
        spawn_menu_button(parent, MenuBtn::Continue, &assets, &localization, lang);
        spawn_menu_button(parent, MenuBtn::Settings, &assets, &localization, lang);
        spawn_menu_button(parent, MenuBtn::Quit, &assets, &localization, lang);
    });
}

pub fn setup_game_settings(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    let lang = settings.language;
    commands.spawn((add_root_node(true), MenuCmp)).with_children(|parent| {
        parent
            .spawn((Node {
                width: percent(40.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::ZERO.with_top(percent(30.)),
                ..default()
            },))
            .with_children(|parent| {
                spawn_label(
                    parent,
                    "language",
                    vec![SettingsBtn::English, SettingsBtn::Spanish],
                    &settings,
                    &assets,
                    &localization,
                );
                spawn_label(
                    parent,
                    "audio",
                    vec![SettingsBtn::Mute, SettingsBtn::Sound, SettingsBtn::Music],
                    &settings,
                    &assets,
                    &localization,
                );
                spawn_label(
                    parent,
                    "autosave",
                    vec![SettingsBtn::True, SettingsBtn::False],
                    &settings,
                    &assets,
                    &localization,
                );
            });

        spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);
    });
}

pub fn start_new_game_message(
    mut start_new_char_msg: MessageReader<StartNewCharacterMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if !start_new_char_msg.is_empty() {
        next_game_state.set(GameState::default());
        next_app_state.set(AppState::Game);

        start_new_char_msg.clear();
    }
}
