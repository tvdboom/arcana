use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::Class;
use crate::core::constants::*;
use crate::core::localization::{
    format_class_description, format_race_description, Localization, LocalizedClassDesc,
    LocalizedRaceDesc, LocalizedText,
};
use crate::core::menu::buttons::*;
use crate::core::menu::settings::{spawn_label, SettingsBtn};
use crate::core::menu::utils::{add_root_node, add_text, reimage};
use crate::core::player::Player;
use crate::core::races::Race;
use crate::core::settings::{Settings, Language};
use crate::core::states::{AppState, GameState};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::window::SystemCursorIcon;

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
                        spawn_menu_button(
                            parent,
                            MenuBtn::NewCharacter,
                            &assets,
                            &localization,
                            lang,
                        );
                        spawn_menu_button(
                            parent,
                            MenuBtn::LoadCharacter,
                            &assets,
                            &localization,
                            lang,
                        );
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
                    bottom: percent(4.),
                    ..default()
                })
                .with_children(|parent| {
                    let credit = localization.get("created_by", lang);
                    parent.spawn((
                        add_text(credit, "medium", SUBTITLE_TEXT_SIZE, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
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
    mut commands: Commands,
    mut start_new_char_msg: MessageReader<StartNewCharacterMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if !start_new_char_msg.is_empty() {
        commands.insert_resource(Player::default());
        next_game_state.set(GameState::default());
        next_app_state.set(AppState::Game);

        start_new_char_msg.clear();
    }
}

pub trait SelectionItem: NameFromEnum + Copy + Clone + Send + Sync + 'static {
    type DescComponent: Component;
    fn get_description(&self, lang: Language, localization: &Localization) -> String;
    fn create_desc_component(&self) -> Self::DescComponent;
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>);
}

impl SelectionItem for Race {
    type DescComponent = LocalizedRaceDesc;

    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_race_description(*self, lang, localization)
    }

    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedRaceDesc(*self)
    }

    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        player.race = *self;
        next_game_state.set(GameState::ChooseClass);
    }
}

impl SelectionItem for Class {
    type DescComponent = LocalizedClassDesc;

    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_class_description(*self, lang, localization)
    }

    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedClassDesc(*self)
    }

    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        player.class = *self;
        player.abilities = vec![self.starting_ability()];
        next_game_state.set(GameState::ChooseSubClass);
    }
}

pub fn setup_selection_screen<T>(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    title_key: &'static str,
    has_back_button: bool,
) where
    T: SelectionItem + IntoEnumIterator,
{
    let lang = settings.language;
    let (mut root_node, pickable) = add_root_node(true);
    root_node.justify_content = JustifyContent::FlexStart;

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode::new(assets.image("bg2")).with_mode(NodeImageMode::Stretch),
            MenuCmp,
        ))
        .with_children(|parent| {
            // Title container
            parent.spawn(Node {
                margin: UiRect {
                    top: percent(3.),
                    bottom: percent(3.),
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    add_text(localization.get(title_key, lang), "bold", TITLE_TEXT_SIZE, &assets),
                    TextColor(BUTTON_TEXT_COLOR),
                    LocalizedText(title_key.to_string()),
                ));
            });

            // Container for the cards
            parent
                .spawn(Node {
                    width: percent(96.),
                    height: percent(70.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    for item in T::iter() {
                        let item_key = item.to_lowername();
                        let item_name = localization.get(&item_key, lang);

                        // Card
                        parent
                            .spawn((
                                Node {
                                    width: percent(22.),
                                    height: percent(98.),
                                    position_type: PositionType::Relative,
                                    margin: UiRect::horizontal(percent(1.5)),
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON_COLOR),
                            ))
                            .with_children(|parent| {
                                // Content container (padded so text/illustration are nicely inset)
                                parent.spawn(Node {
                                    width: percent(100.),
                                    height: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::FlexStart,
                                    padding: UiRect::all(percent(1.5)),
                                    ..default()
                                }).with_children(|parent| {
                                    // Illustration image
                                    parent.spawn((
                                        Node {
                                            width: percent(100.),
                                            height: percent(50.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image(item_key.clone())).with_mode(NodeImageMode::Stretch),
                                    ));

                                    // Stone background container for Name and Description
                                    parent
                                        .spawn((
                                            Node {
                                                width: percent(100.),
                                                height: percent(50.),
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::FlexStart,
                                                ..default()
                                            },
                                            ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
                                        ))
                                        .with_children(|parent| {
                                            // Name
                                            parent.spawn((
                                                Node {
                                                    margin: UiRect::vertical(percent(4.5)),
                                                    ..default()
                                                },
                                                add_text(item_name, "bold", SUBTITLE_TEXT_SIZE, &assets),
                                                TextColor(BUTTON_TEXT_COLOR),
                                                LocalizedText(item_key.clone()),
                                            ));

                                            // Description
                                            parent.spawn((
                                                Node {
                                                    width: percent(85.),
                                                    margin: UiRect::horizontal(percent(7.5)),
                                                    ..default()
                                                },
                                                add_text(item.get_description(lang, &localization), "medium", 1.8, &assets),
                                                TextColor(Color::WHITE),
                                                item.create_desc_component(),
                                            ));
                                        });
                                });

                                // Border Overlay (absolutely positioned on top and covering the card perfectly)
                                parent
                                    .spawn((
                                        Node {
                                            position_type: PositionType::Absolute,
                                            width: percent(110.),
                                            height: percent(110.),
                                            left: percent(-5.),
                                            top: percent(-5.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image("border")).with_mode(NodeImageMode::Stretch),
                                    ))
                                    .observe(reimage::<Over>(assets.image("border_hover")))
                                    .observe(reimage::<Out>(assets.image("border")))
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(move |
                                        _: On<Pointer<Click>>,
                                        mut player: ResMut<Player>,
                                        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
                                        mut next_game_state: ResMut<NextState<GameState>>| {
                                        play_audio_msg.write(PlayAudioMsg::new("button"));

                                        item.on_select(&mut player, &mut next_game_state);
                                    });
                            });
                    }
                });

            // Back button container centered horizontally at the bottom of the screen
            if has_back_button {
                parent
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        width: percent(100.),
                        bottom: percent(3.),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|parent| {
                        spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);
                    });
            }
        });
}

pub fn setup_race_selection(
    commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    setup_selection_screen::<Race>(commands, settings, assets, localization, "choose race", false);
}

pub fn setup_class_selection(
    commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
) {
    setup_selection_screen::<Class>(commands, settings, assets, localization, "choose class", true);
}
