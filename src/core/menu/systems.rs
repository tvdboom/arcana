use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::{Ajah, Class};
use crate::core::constants::*;
use crate::core::localization::*;
use crate::core::menu::buttons::*;
use crate::core::menu::settings::{spawn_label, SettingsBtn};
use crate::core::menu::utils::{add_root_node, add_text, recolor, reimage};
use crate::core::pets::Pet;
use crate::core::player::{Attribute, Player};
use crate::core::races::Race;
use crate::core::settings::{Language, Settings};
use crate::core::weapons::Weapon;
use crate::core::consumables::Consumable;
use crate::core::states::{AppState, GameState};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::input::keyboard::{Key, KeyboardInput};
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

                        // Spacer to push the back button lower down
                        parent.spawn(Node {
                            height: Val::Px(50.),
                            ..default()
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
        parent.spawn((
            Node {
                width: Val::Px(550.),
                height: Val::Px(560.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(25.)),
                ..default()
            },
            ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
        )).with_children(|parent| {
            spawn_menu_button(parent, MenuBtn::Continue, &assets, &localization, lang);
            #[cfg(not(target_arch = "wasm32"))]
            spawn_menu_button(parent, MenuBtn::SaveCharacter, &assets, &localization, lang);
            spawn_menu_button(parent, MenuBtn::Settings, &assets, &localization, lang);
            spawn_menu_button(parent, MenuBtn::Quit, &assets, &localization, lang);
        });
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
            .spawn((
                Node {
                    width: Val::Px(580.),
                    height: Val::Px(680.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(25.)),
                    ..default()
                },
                ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
            ))
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

                // Spacer to push the back button lower down
                parent.spawn(Node {
                    height: Val::Px(75.),
                    ..default()
                });

                spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);
            });
    });
}

pub fn start_new_game_message(
    mut start_new_char_msg: MessageReader<StartNewCharacterMsg>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut player: ResMut<Player>,
) {
    if !start_new_char_msg.is_empty() {
        *player = Player::default();
        next_game_state.set(GameState::default());
        next_app_state.set(AppState::Game);

        start_new_char_msg.clear();
    }
}

#[derive(Component)]
pub struct CharacterNameText;

#[derive(Component, Clone, Copy)]
pub enum AttributeAction {
    Plus(Attribute),
    Minus(Attribute),
}

#[derive(Component)]
pub struct AttributeValueText(pub Attribute);

#[derive(Component)]
pub struct PointsRemainingText;

#[derive(Component)]
pub struct CreateCharacterContinueBtn;

pub fn handle_name_input(
    mut events: MessageReader<KeyboardInput>,
    mut player: ResMut<Player>,
    mut text_q: Query<&mut Text, With<CharacterNameText>>,
) {
    let mut changed = false;
    for event in events.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(c) => {
                // limit name to 16 characters
                if player.name.len() < 16 {
                    // Only allow alphanumeric characters or spaces
                    if c.chars().all(|ch| ch.is_alphanumeric() || ch == ' ') {
                        player.name.push_str(c);
                        changed = true;
                    }
                }
            },
            Key::Backspace => {
                player.name.pop();
                changed = true;
            },
            Key::Space => {
                if player.name.len() < 16 {
                    player.name.push(' ');
                    changed = true;
                }
            },
            _ => {},
        }
    }

    if changed {
        for mut text in &mut text_q {
            text.0 = player.name.clone();
        }
    }
}

fn on_attribute_button_click(
    event: On<Pointer<Click>>,
    btn_q: Query<(Option<&DisabledButton>, &AttributeAction)>,
    mut player: ResMut<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut text_q: Query<(&mut Text, Option<&AttributeValueText>, Option<&PointsRemainingText>)>,
) {
    let (disabled, action) = btn_q.get(event.entity).unwrap();
    if disabled.is_some() {
        return;
    }
    play_audio_msg.write(PlayAudioMsg::new("button"));

    // Calculate sum of attributes to find remaining points
    let current_sum = (player.strength
        + player.dexterity
        + player.constitution
        + player.intelligence
        + player.wisdom
        + player.charisma) as i32;
    let remaining = 60 - current_sum;

    match action {
        AttributeAction::Plus(attr) => {
            if remaining > 0 {
                let val = match attr {
                    Attribute::Strength => &mut player.strength,
                    Attribute::Dexterity => &mut player.dexterity,
                    Attribute::Constitution => &mut player.constitution,
                    Attribute::Intelligence => &mut player.intelligence,
                    Attribute::Wisdom => &mut player.wisdom,
                    Attribute::Charisma => &mut player.charisma,
                };

                if *val < 15 {
                    *val += 1;
                }
            }
        },
        AttributeAction::Minus(attr) => {
            let val = match attr {
                Attribute::Strength => &mut player.strength,
                Attribute::Dexterity => &mut player.dexterity,
                Attribute::Constitution => &mut player.constitution,
                Attribute::Intelligence => &mut player.intelligence,
                Attribute::Wisdom => &mut player.wisdom,
                Attribute::Charisma => &mut player.charisma,
            };

            if *val > 5 {
                *val -= 1;
            }
        },
    }

    // Now update all UI texts
    let new_sum = (player.strength
        + player.dexterity
        + player.constitution
        + player.intelligence
        + player.wisdom
        + player.charisma) as i32;
    let new_remaining = 60 - new_sum;

    for (mut text, val_attr, remaining_text) in &mut text_q {
        if let Some(val_attr) = val_attr {
            let val = match val_attr.0 {
                Attribute::Strength => player.strength,
                Attribute::Dexterity => player.dexterity,
                Attribute::Constitution => player.constitution,
                Attribute::Intelligence => player.intelligence,
                Attribute::Wisdom => player.wisdom,
                Attribute::Charisma => player.charisma,
            };
            text.0 = format!("{}", val as i32);
        } else if remaining_text.is_some() {
            let points_label = localization.get("points remaining", settings.language);
            text.0 = format!("{}: {}", points_label, new_remaining);
        }
    }
}

fn on_continue_click(
    _: On<Pointer<Click>>,
    player: Res<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let current_sum = (player.strength
        + player.dexterity
        + player.constitution
        + player.intelligence
        + player.wisdom
        + player.charisma) as i32;

    if !player.name.trim().is_empty() && current_sum == 60 {
        play_audio_msg.write(PlayAudioMsg::new("button"));
        next_game_state.set(GameState::ChooseRace);
    }
}

pub fn update_character_creation_continue_btn(
    player: Res<Player>,
    mut btn_q: Query<
        (Entity, &mut BackgroundColor, &mut BorderColor, Option<&DisabledButton>),
        With<CreateCharacterContinueBtn>,
    >,
    mut commands: Commands,
) {
    let sum = (player.strength
        + player.dexterity
        + player.constitution
        + player.intelligence
        + player.wisdom
        + player.charisma) as i32;
    let is_valid = !player.name.trim().is_empty() && sum == 60;

    for (entity, mut bg, mut border, disabled) in &mut btn_q {
        if is_valid {
            if disabled.is_some() {
                commands.entity(entity).remove::<DisabledButton>();
                bg.0 = NORMAL_BUTTON_COLOR;
                *border = BorderColor::all(BUTTON_BORDER_COLOR);
            }
        } else {
            if disabled.is_none() {
                commands.entity(entity).insert(DisabledButton);
                bg.0 = DISABLED_BUTTON_COLOR;
                *border = BorderColor::all(DISABLED_BORDER_COLOR);
            }
        }
    }
}

pub fn update_attribute_buttons(
    player: Res<Player>,
    mut btn_q: Query<(
        Entity,
        &AttributeAction,
        &mut BackgroundColor,
        &mut BorderColor,
        Option<&DisabledButton>,
    )>,
    mut commands: Commands,
) {
    let current_sum = (player.strength
        + player.dexterity
        + player.constitution
        + player.intelligence
        + player.wisdom
        + player.charisma) as i32;
    let remaining = 60 - current_sum;

    for (entity, action, mut bg, mut border, disabled) in &mut btn_q {
        let is_disabled = match action {
            AttributeAction::Minus(attr) => {
                let val = match attr {
                    Attribute::Strength => player.strength,
                    Attribute::Dexterity => player.dexterity,
                    Attribute::Constitution => player.constitution,
                    Attribute::Intelligence => player.intelligence,
                    Attribute::Wisdom => player.wisdom,
                    Attribute::Charisma => player.charisma,
                };
                val <= 5
            },
            AttributeAction::Plus(attr) => {
                let val = match attr {
                    Attribute::Strength => player.strength,
                    Attribute::Dexterity => player.dexterity,
                    Attribute::Constitution => player.constitution,
                    Attribute::Intelligence => player.intelligence,
                    Attribute::Wisdom => player.wisdom,
                    Attribute::Charisma => player.charisma,
                };
                val >= 15 || remaining <= 0
            },
        };

        if is_disabled {
            if disabled.is_none() {
                commands.entity(entity).insert(DisabledButton);
                bg.0 = DISABLED_BUTTON_COLOR;
                *border = BorderColor::all(DISABLED_BORDER_COLOR);
            }
        } else {
            if disabled.is_some() {
                commands.entity(entity).remove::<DisabledButton>();
                bg.0 = NORMAL_BUTTON_COLOR;
                *border = BorderColor::all(BUTTON_BORDER_COLOR);
            }
        }
    }
}

fn spawn_attribute_button(
    parent: &mut ChildSpawnerCommands,
    action: AttributeAction,
    label: &str,
    assets: &WorldAssets,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(32.),
                height: Val::Px(32.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.)),
                border_radius: BorderRadius::all(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            action,
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(on_attribute_button_click)
        .with_children(|parent| {
            parent.spawn((
                add_text(label, "bold", BUTTON_TEXT_SIZE - 0.5, assets),
                TextColor(BUTTON_TEXT_COLOR),
            ));
        });
}

fn spawn_continue_button(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(200.),
                height: Val::Px(45.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Px(8.)),
                border: UiRect::all(Val::Px(2.)),
                border_radius: BorderRadius::all(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            CreateCharacterContinueBtn,
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(cursor::<Release>(SystemCursorIcon::Default))
        .observe(on_continue_click)
        .with_children(|parent| {
            parent.spawn((
                add_text(localization.get("continue", lang), "bold", BUTTON_TEXT_SIZE, assets),
                TextColor(BUTTON_TEXT_COLOR),
            ));
        });
}

pub fn setup_character_creation(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
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
            parent
                .spawn(Node {
                    margin: UiRect {
                        top: percent(5.),
                        bottom: percent(3.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get("create your character", lang),
                            "bold",
                            TITLE_TEXT_SIZE,
                            &assets,
                        ),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText("create your character".to_string()),
                    ));
                });

            // Main container (Horizontal row with name selection on the left, attributes on the right)
            parent
                .spawn(Node {
                    width: percent(55.),
                    height: percent(65.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // Left Column: Name selection
                    parent
                        .spawn(Node {
                            width: percent(45.),
                            height: percent(100.),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    localization.get("character name", lang),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("character name".to_string()),
                                Node {
                                    margin: UiRect::bottom(percent(5.)),
                                    ..default()
                                },
                            ));

                            // Text display box
                            parent
                                .spawn((
                                    Node {
                                        width: percent(80.),
                                        height: Val::Px(60.),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(3.)),
                                        border_radius: BorderRadius::all(Val::Px(6.)),
                                        ..default()
                                    },
                                    BackgroundColor(NORMAL_BUTTON_COLOR),
                                    BorderColor::all(BUTTON_BORDER_COLOR),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(
                                            player.name.clone(),
                                            "medium",
                                            BUTTON_TEXT_SIZE,
                                            &assets,
                                        ),
                                        TextColor(Color::WHITE),
                                        CharacterNameText,
                                    ));
                                });

                            parent.spawn((
                                add_text(
                                    localization.get("change name hint", lang),
                                    "medium",
                                    LABEL_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(Color::srgba_u8(180, 180, 180, 255)),
                                LocalizedText("change name hint".to_string()),
                                Node {
                                    margin: UiRect::top(percent(3.)),
                                    ..default()
                                },
                            ));
                        });

                    // Right Column: Attribute allocation
                    parent
                        .spawn(Node {
                            width: percent(45.),
                            height: percent(100.),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            // Points remaining
                            let current_sum = (player.strength
                                + player.dexterity
                                + player.constitution
                                + player.intelligence
                                + player.wisdom
                                + player.charisma)
                                as i32;
                            let remaining = 60 - current_sum;

                            let points_label = localization.get("points remaining", lang);
                            parent.spawn((
                                add_text(
                                    format!("{}: {}", points_label, remaining),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                Node {
                                    margin: UiRect::bottom(percent(4.)),
                                    ..default()
                                },
                                PointsRemainingText,
                            ));

                            // Attributes grid/stack
                            parent
                                .spawn(Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    for attr in Attribute::iter() {
                                        let translated_attr_name =
                                            localization.get(attr.to_lowername().as_str(), lang);
                                        let val = match attr {
                                            Attribute::Strength => player.strength,
                                            Attribute::Dexterity => player.dexterity,
                                            Attribute::Constitution => player.constitution,
                                            Attribute::Intelligence => player.intelligence,
                                            Attribute::Wisdom => player.wisdom,
                                            Attribute::Charisma => player.charisma,
                                        } as i32;

                                        // Row for this attribute
                                        parent
                                            .spawn(Node {
                                                width: percent(75.),
                                                height: Val::Px(45.),
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::SpaceBetween,
                                                margin: UiRect::vertical(Val::Px(5.)),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                // Name label
                                                parent.spawn((
                                                    add_text(
                                                        translated_attr_name,
                                                        "medium",
                                                        BUTTON_TEXT_SIZE - 0.5,
                                                        &assets,
                                                    ),
                                                    TextColor(BUTTON_TEXT_COLOR),
                                                    LocalizedText(attr.to_lowername()),
                                                    Node {
                                                        width: percent(45.),
                                                        ..default()
                                                    },
                                                ));

                                                // Controls (Minus, Value, Plus)
                                                parent
                                                    .spawn(Node {
                                                        width: percent(50.),
                                                        flex_direction: FlexDirection::Row,
                                                        align_items: AlignItems::Center,
                                                        justify_content: JustifyContent::End,
                                                        ..default()
                                                    })
                                                    .with_children(|parent| {
                                                        // Minus button
                                                        spawn_attribute_button(
                                                            parent,
                                                            AttributeAction::Minus(attr),
                                                            "-",
                                                            &assets,
                                                        );

                                                        // Value container (fixed width to align buttons even for numbers below 10)
                                                        parent
                                                            .spawn((Node {
                                                                width: Val::Px(55.),
                                                                justify_content:
                                                                    JustifyContent::Center,
                                                                align_items: AlignItems::Center,
                                                                ..default()
                                                            },))
                                                            .with_children(|parent| {
                                                                parent.spawn((
                                                                    add_text(
                                                                        format!("{}", val),
                                                                        "bold",
                                                                        BUTTON_TEXT_SIZE,
                                                                        &assets,
                                                                    ),
                                                                    TextColor(BUTTON_TEXT_COLOR),
                                                                    AttributeValueText(attr),
                                                                ));
                                                            });

                                                        // Plus button
                                                        spawn_attribute_button(
                                                            parent,
                                                            AttributeAction::Plus(attr),
                                                            "+",
                                                            &assets,
                                                        );
                                                    });
                                            });
                                    }
                                });
                        });
                });

            // Bottom Buttons (Back and Continue)
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.),
                    bottom: percent(4.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // Back button
                    spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);

                    // Continue button
                    spawn_continue_button(parent, &assets, &localization, lang);
                });
        });
}

pub trait SelectionItem: NameFromEnum + Copy + Clone + Send + Sync + 'static {
    type DescComponent: Component;
    fn get_description(&self, lang: Language, localization: &Localization) -> String;
    fn create_desc_component(&self) -> Self::DescComponent;
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>);
    fn get_image_key(&self, _player: &Player) -> String {
        self.to_lowername()
    }
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
        player.perks = vec![self.starting_perk()];

        match self {
            Class::Warrior => {
                player.weapon_rh = Some(Weapon::SteelSword);
            }
            Class::Mage(_) => {
                player.weapon_2h = Some(Weapon::WizardStaff);
            }
            Class::Rogue => {
                player.weapon_rh = Some(Weapon::ThiefDagger);
            }
            Class::Druid => {
                player.weapon_rh = Some(Weapon::OakWand);
            }
        }
        
        if matches!(*self, Class::Mage(_) | Class::Druid) {
            next_game_state.set(GameState::ChooseSubClass);
        } else {
            next_game_state.set(GameState::Playing);
        }
    }

    fn get_image_key(&self, player: &Player) -> String {
        let race_key = player.race.to_lowername();
        match self {
            Class::Mage(_) => format!("mage_{}", race_key),
            Class::Warrior => format!("warrior_{}", race_key),
            Class::Rogue => format!("rogue_{}", race_key),
            Class::Druid => format!("druid_{}", race_key),
        }
    }
}

impl SelectionItem for Ajah {
    type DescComponent = LocalizedAjahDesc;

    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_ajah_description(*self, lang, localization)
    }

    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedAjahDesc(*self)
    }

    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        player.class = Class::Mage(*self);
        player.abilities.push(self.special_ability());
        next_game_state.set(GameState::Playing);
    }

    fn get_image_key(&self, player: &Player) -> String {
        let race_key = player.race.to_lowername();
        match self {
            Ajah::Black => "mage_black".to_string(),
            Ajah::Red => format!("mage_red_{}", race_key),
            Ajah::Green => format!("mage_green_{}", race_key),
            Ajah::White => format!("mage_white_{}", race_key),
        }
    }
}

impl SelectionItem for Pet {
    type DescComponent = LocalizedPetDesc;

    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_pet_description(*self, lang, localization)
    }

    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedPetDesc(*self)
    }

    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        player.pet = Some(*self);
        next_game_state.set(GameState::Playing);
    }
}

pub fn setup_selection_screen<T>(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    title_key: &'static str,
    has_back_button: bool,
    player: &Player,
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
                                        ImageNode::new(assets.image(item.get_image_key(player))).with_mode(NodeImageMode::Stretch),
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
    player: Res<Player>,
) {
    setup_selection_screen::<Race>(
        commands,
        settings,
        assets,
        localization,
        "choose race",
        true,
        &player,
    );
}

pub fn setup_class_selection(
    commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
    setup_selection_screen::<Class>(
        commands,
        settings,
        assets,
        localization,
        "choose class",
        true,
        &player,
    );
}

pub fn setup_subclass_selection(
    commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
    match player.class {
        Class::Mage(_) => {
            setup_selection_screen::<Ajah>(
                commands,
                settings,
                assets,
                localization,
                "choose subclass",
                true,
                &player,
            );
        },
        Class::Druid => {
            setup_selection_screen::<Pet>(
                commands,
                settings,
                assets,
                localization,
                "choose pet",
                true,
                &player,
            );
        },
        _ => {},
    }
}

#[derive(Component)]
pub struct PlayingCmp;

#[derive(Component)]
pub struct PlayingLog(pub Vec<String>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatType {
    Level,
    Ap,
    Money,
    Helmet,
    Armor,
    Boots,
    WeaponLh,
    WeaponRh,
    Weapon2h,
    Consumables,
    Attributes,
    Abilities,
    Perks,
    Pet,
}

#[derive(Component)]
pub struct StatLabel(pub StatType);

#[derive(Component)]
pub struct LogTextContainer;

#[derive(Component)]
pub struct ActionButton(pub &'static str);

pub fn setup_playing_screen(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
    let lang = settings.language;
    commands.spawn((
        PlayingLog(vec![if lang == Language::Spanish {
            "¡Bienvenido a Arcana! Comienza tu aventura.".to_string()
        } else {
            "Welcome to Arcana! Start your adventure.".to_string()
        }]),
        PlayingCmp,
    ));

    let (root_node, pickable) = add_root_node(true);

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode::new(assets.image("bg3")).with_mode(NodeImageMode::Stretch),
            PlayingCmp,
        ))
        .with_children(|parent| {
            // Spacer node
            parent.spawn(Node {
                width: percent(100.),
                height: percent(5.),
                ..default()
            });

            // Main Columns container
            parent.spawn(Node {
                width: percent(96.),
                height: percent(85.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::horizontal(percent(2.)),
                ..default()
            }).with_children(|parent| {
                // Left Column: Character & Equipment
                parent.spawn((
                    Node {
                        width: percent(42.),
                        height: percent(95.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(percent(2.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                )).with_children(|parent| {
                    // Title info
                    parent.spawn((
                        add_text(&player.name, "bold", SUBTITLE_TEXT_SIZE, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));

                    // Portrait
                    let portrait_key = match player.class {
                        Class::Mage(ajah) => ajah.get_image_key(&player),
                        _ => player.class.get_image_key(&player),
                    };

                    parent.spawn((
                        Node {
                            width: percent(75.),
                            aspect_ratio: Some(1.0),
                            margin: UiRect::vertical(percent(1.5)),
                            border: UiRect::all(Val::Px(3.)),
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        ImageNode::new(assets.image(portrait_key)).with_mode(NodeImageMode::Stretch),
                    )).with_children(|parent| {
                        if let Some(pet) = &player.pet {
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(8.),
                                    bottom: Val::Px(8.),
                                    width: percent(38.),
                                    aspect_ratio: Some(1.0),
                                    border: UiRect::all(Val::Px(2.)),
                                    ..default()
                                },
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                ImageNode::new(assets.image(pet.get_image_key(&player))).with_mode(NodeImageMode::Stretch),
                            ));
                        }
                    });

                    // Equipment details container
                    parent.spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            padding: UiRect::left(percent(5.)),
                            ..default()
                        },
                    )).with_children(|parent| {
                        // Equipment lines
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Helmet)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Armor)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Boots)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::WeaponLh)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::WeaponRh)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Weapon2h)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Consumables)));
                    });
                });

                // Right Column: Stats, Action buttons, Logs
                parent.spawn((
                    Node {
                        width: percent(54.),
                        height: percent(95.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                )).with_children(|parent| {
                    // Top: Stats card
                    parent.spawn((
                        Node {
                            width: percent(100.),
                            height: percent(45.),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::all(percent(2.)),
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON_COLOR),
                    )).with_children(|parent| {
                        parent.spawn((add_text("", "bold", SUBTITLE_TEXT_SIZE, &assets), TextColor(BUTTON_TEXT_COLOR), StatLabel(StatType::Level)));
                        parent.spawn((add_text("", "bold", 2.2, &assets), TextColor(BUTTON_TEXT_COLOR), StatLabel(StatType::Ap)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Attributes)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Abilities)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Perks)));
                        parent.spawn((add_text("", "medium", 1.8, &assets), TextColor(Color::WHITE), StatLabel(StatType::Pet)));
                        parent.spawn((add_text("", "bold", 2.0, &assets), TextColor(BUTTON_TEXT_COLOR), StatLabel(StatType::Money)));
                    });

                    // Middle: Actions grid
                    parent.spawn(Node {
                        width: percent(100.),
                        height: percent(35.),
                        display: Display::Grid,
                        grid_template_columns: vec![GridTrack::flex(1.), GridTrack::flex(1.), GridTrack::flex(1.)],
                        grid_template_rows: vec![GridTrack::flex(1.), GridTrack::flex(1.)],
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    }).with_children(|parent| {
                        // Action buttons: Hunt, Buy, Quest, Train, Craft, Heal
                        spawn_playing_action_button(parent, "hunt", &assets, &localization, lang);
                        spawn_playing_action_button(parent, "buy", &assets, &localization, lang);
                        spawn_playing_action_button(parent, "quest", &assets, &localization, lang);
                        spawn_playing_action_button(parent, "train", &assets, &localization, lang);
                        spawn_playing_action_button(parent, "craft", &assets, &localization, lang);
                        spawn_playing_action_button(parent, "heal", &assets, &localization, lang);
                    });

                    // Bottom: Log box
                    parent.spawn((
                        Node {
                            width: percent(100.),
                            height: percent(15.),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(percent(1.)),
                            ..default()
                        },
                        BackgroundColor(NORMAL_BUTTON_COLOR),
                    )).with_children(|parent| {
                        parent.spawn((
                            add_text("", "bold", 1.8, &assets),
                            TextColor(Color::WHITE),
                            LogTextContainer,
                        ));
                    });
                });
            });
        });
}

pub fn spawn_playing_action_button(
    parent: &mut ChildSpawnerCommands,
    action: &'static str,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    let action_label = localization.get(action, lang);
    parent.spawn((
        Node {
            margin: UiRect::all(Val::Px(4.)),
            height: Val::Px(55.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(2.)),
            border_radius: BorderRadius::all(Val::Px(4.)),
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON_COLOR),
        BorderColor::all(BUTTON_BORDER_COLOR),
        Button,
        ActionButton(action),
    ))
    .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
    .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
    .observe(cursor::<Out>(SystemCursorIcon::Default))
    .with_children(|parent| {
        parent.spawn((
            add_text(action_label, "bold", BUTTON_TEXT_SIZE, assets),
            TextColor(BUTTON_TEXT_COLOR),
            LocalizedText(action.to_string()),
        ));
    });
}

pub fn update_playing_screen(
    player: Res<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut text_q: Query<(&mut Text, &StatLabel)>,
    log_q: Query<&PlayingLog>,
    mut log_text_q: Query<&mut Text, (With<LogTextContainer>, Without<StatLabel>)>,
) {
    let lang = settings.language;
    for (mut text, stat) in &mut text_q {
        text.0 = match &stat.0 {
            StatType::Level => format!("{}: {}", localization.get("level", lang), player.level),
            StatType::Ap => format!("{}: {}", localization.get("action_points", lang), player.ap),
            StatType::Money => format!("{}: {} {}", localization.get("money", lang), player.money, localization.get("money", lang)),
            StatType::Helmet => {
                let val = player.helmet.as_ref().map(|s| localization.get(&s.to_lowername(), lang)).unwrap_or_else(|| localization.get("none", lang));
                format!("{}: {}", localization.get("helmet", lang), val)
            }
            StatType::Armor => {
                let val = player.armor.as_ref().map(|s| localization.get(&s.to_lowername(), lang)).unwrap_or_else(|| localization.get("none", lang));
                format!("{}: {}", localization.get("armor", lang), val)
            }
            StatType::Boots => {
                let val = player.boots.as_ref().map(|s| localization.get(&s.to_lowername(), lang)).unwrap_or_else(|| localization.get("none", lang));
                format!("{}: {}", localization.get("boots", lang), val)
            }
            StatType::WeaponLh => {
                let val = player.weapon_lh.as_ref().map(|s| localization.get(&s.to_lowername(), lang)).unwrap_or_else(|| localization.get("none", lang));
                format!("{}: {}", localization.get("weapon_lh", lang), val)
            }
            StatType::WeaponRh => {
                let val = player.weapon_rh.as_ref().map(|s| localization.get(&s.to_lowername(), lang)).unwrap_or_else(|| localization.get("none", lang));
                format!("{}: {}", localization.get("weapon_rh", lang), val)
            }
            StatType::Weapon2h => {
                let val = player.weapon_2h.as_ref().map(|s| localization.get(&s.to_lowername(), lang)).unwrap_or_else(|| localization.get("none", lang));
                format!("{}: {}", localization.get("weapon_2h", lang), val)
            }
            StatType::Consumables => {
                let items: Vec<String> = player.consumables.iter().map(|s| localization.get(&s.to_lowername(), lang)).collect();
                let val = if items.is_empty() { localization.get("none", lang) } else { items.join(", ") };
                format!("{}: {}", localization.get("consumables", lang), val)
            }
            StatType::Attributes => {
                let str_label = localization.get("strength", lang);
                let dex_label = localization.get("dexterity", lang);
                let con_label = localization.get("constitution", lang);
                let int_label = localization.get("intelligence", lang);
                let wis_label = localization.get("wisdom", lang);
                let cha_label = localization.get("charisma", lang);
                format!(
                    "{}: {} | {}: {} | {}: {}\n{}: {} | {}: {} | {}: {}",
                    str_label, player.strength(),
                    dex_label, player.dexterity(),
                    con_label, player.constitution(),
                    int_label, player.intelligence(),
                    wis_label, player.wisdom(),
                    cha_label, player.charisma()
                )
            }
            StatType::Abilities => {
                let list: Vec<String> = player.abilities.iter().map(|a| localization.get(&a.to_lowername(), lang)).collect();
                format!("{}: {}", localization.get("abilities", lang), list.join(", "))
            }
            StatType::Perks => {
                let list: Vec<String> = player.perks.iter().map(|p| localization.get(&p.to_lowername(), lang)).collect();
                format!("{}: {}", localization.get("perks", lang), list.join(", "))
            }
            StatType::Pet => {
                let val = player.pet.as_ref().map(|p| localization.get(&p.to_lowername(), lang)).unwrap_or_else(|| localization.get("none", lang));
                format!("{}: {}", localization.get("pet", lang), val)
            }
        };
    }

    if let Some(log) = log_q.iter().next() {
        if let Some(mut text) = log_text_q.iter_mut().next() {
            text.0 = log.0.join("\n");
        }
    }
}

pub fn handle_playing_action_clicks(
    mut player: ResMut<Player>,
    mut log_q: Query<&mut PlayingLog>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    interaction_q: Query<(&Interaction, &ActionButton), (Changed<Interaction>, With<Button>)>,
) {
    use rand::RngExt;
    let mut log_updated = false;
    let mut log_msgs = Vec::new();
    let lang = settings.language;

    for (interaction, action) in &interaction_q {
        if *interaction == Interaction::Pressed {
            play_audio_msg.write(PlayAudioMsg::new("button"));
            
            let cost_gold = match action.0 {
                "craft" => 15,
                "buy" => 30,
                "heal" => 10,
                _ => 0,
            };

            if player.money < cost_gold {
                play_audio_msg.write(PlayAudioMsg::new("error"));
                let err_msg = if lang == Language::Spanish {
                    format!("¡No hay suficiente oro para {}! (Necesitas {} oro)", action.0, cost_gold)
                } else {
                    format!("Not enough gold to {}! (Need {} gold)", action.0, cost_gold)
                };
                log_msgs.push(err_msg);
                log_updated = true;
                continue;
            }

            player.money -= cost_gold;

            // Handle the specific action
            let (ap_cost, log_msg) = match action.0 {
                "hunt" => {
                    let gold_earned = rand::rng().random_range(10..=20);
                    player.money += gold_earned;
                    let msg = if lang == Language::Spanish {
                        format!("Fuiste a cazar y obtuviste {} oro.", gold_earned)
                    } else {
                        format!("You went hunting and found {} gold.", gold_earned)
                    };
                    (2, msg)
                }
                "buy" => {
                    let items = vec![Consumable::HealingPotion, Consumable::ManaPotion, Consumable::PoisonVial, Consumable::HerbBlend];
                    use rand::seq::IndexedRandom;
                    let item = items.choose(&mut rand::rng()).copied().unwrap_or(Consumable::HealingPotion);
                    player.consumables.push(item);
                    let item_name = localization.get(&item.to_lowername(), lang);
                    let msg = if lang == Language::Spanish {
                        format!("Compraste una {} en la tienda.", item_name)
                    } else {
                        format!("You bought a {} from the shop.", item_name)
                    };
                    (1, msg)
                }
                "quest" => {
                    let gold_earned = rand::rng().random_range(20..=40);
                    let mut found_item = None;
                    if rand::rng().random_bool(0.5) {
                        let upgrade_types = vec![
                            Weapon::IronHelmet, Weapon::IronChestplate, Weapon::IronBoots, Weapon::SteelSword, 
                            Weapon::IronShield, Weapon::WizardStaff, Weapon::MageRobes, Weapon::ClothShoes, 
                            Weapon::LeatherArmor, Weapon::SilentBoots, Weapon::AssassinDagger, Weapon::ThiefDagger, 
                            Weapon::OakWand, Weapon::LeafyGarb
                        ];
                        use rand::seq::IndexedRandom;
                        if let Some(&upg) = upgrade_types.choose(&mut rand::rng()) {
                            found_item = Some(upg);
                            match upg {
                                Weapon::IronHelmet => player.helmet = Some(upg),
                                Weapon::IronChestplate | Weapon::MageRobes | Weapon::LeatherArmor | Weapon::LeafyGarb => player.armor = Some(upg),
                                Weapon::IronBoots | Weapon::ClothShoes | Weapon::SilentBoots | Weapon::LeatherBoots => player.boots = Some(upg),
                                Weapon::WizardStaff => {
                                    player.weapon_2h = Some(upg);
                                    player.weapon_lh = None;
                                    player.weapon_rh = None;
                                }
                                Weapon::SteelSword | Weapon::AssassinDagger | Weapon::ThiefDagger | Weapon::OakWand => {
                                    player.weapon_lh = Some(upg);
                                }
                                Weapon::IronShield => {
                                    player.weapon_rh = Some(upg);
                                }
                            }
                        }
                    }
                    player.money += gold_earned;
                    let msg = if let Some(item) = found_item {
                        let item_name = localization.get(&item.to_lowername(), lang);
                        if lang == Language::Spanish {
                            format!("¡Misión completada! Ganaste {} oro y encontraste: {}.", gold_earned, item_name)
                        } else {
                            format!("Quest completed! Gained {} gold and found: {}.", gold_earned, item_name)
                        }
                    } else {
                        if lang == Language::Spanish {
                            format!("Misión completada. Ganaste {} oro.", gold_earned)
                        } else {
                            format!("Quest completed. Gained {} gold.", gold_earned)
                        }
                    };
                    (3, msg)
                }
                "train" => {
                    let attr_idx = rand::rng().random_range(0..6);
                    let (attr_name, new_val) = match attr_idx {
                        0 => { player.strength += 1; (localization.get("strength", lang), player.strength) }
                        1 => { player.dexterity += 1; (localization.get("dexterity", lang), player.dexterity) }
                        2 => { player.constitution += 1; (localization.get("constitution", lang), player.constitution) }
                        3 => { player.intelligence += 1; (localization.get("intelligence", lang), player.intelligence) }
                        4 => { player.wisdom += 1; (localization.get("wisdom", lang), player.wisdom) }
                        _ => { player.charisma += 1; (localization.get("charisma", lang), player.charisma) }
                    };
                    let msg = if lang == Language::Spanish {
                        format!("Entrenaste duro. Tu {} aumentó a {}.", attr_name, new_val)
                    } else {
                        format!("You trained hard. Your {} increased to {}.", attr_name, new_val)
                    };
                    (2, msg)
                }
                "craft" => {
                    let items = vec![Weapon::IronHelmet, Weapon::IronChestplate, Weapon::IronBoots, Weapon::SteelSword, Weapon::IronShield];
                    use rand::seq::IndexedRandom;
                    let item = items.choose(&mut rand::rng()).copied().unwrap_or(Weapon::IronHelmet);
                    match item {
                        Weapon::IronHelmet => player.helmet = Some(item),
                        Weapon::IronChestplate => player.armor = Some(item),
                        Weapon::IronBoots => player.boots = Some(item),
                        Weapon::SteelSword => player.weapon_lh = Some(item),
                        Weapon::IronShield => player.weapon_rh = Some(item),
                        _ => {}
                    }
                    let item_name = localization.get(&item.to_lowername(), lang);
                    let msg = if lang == Language::Spanish {
                        format!("Fabricaste una {}.", item_name)
                    } else {
                        format!("You crafted a {}.", item_name)
                    };
                    (2, msg)
                }
                "heal" => {
                    player.health = 100.;
                    let msg = if lang == Language::Spanish {
                        "Restauraste completamente tu salud.".to_string()
                    } else {
                        "You completely restored your health.".to_string()
                    };
                    (1, msg)
                }
                _ => (0, "".to_string()),
            };

            log_msgs.push(log_msg);

            // Deduct action points
            if player.ap <= ap_cost {
                player.level += 1;
                player.ap = 10 + (player.level as u32) * 2;
                player.strength += 1;
                player.dexterity += 1;
                player.constitution += 1;
                player.intelligence += 1;
                player.wisdom += 1;
                player.charisma += 1;
                play_audio_msg.write(PlayAudioMsg::new("victory"));
                let lvl_msg = if lang == Language::Spanish {
                    format!("¡SUBIDA DE NIVEL! ¡Has alcanzado el nivel {}!", player.level)
                } else {
                    format!("LEVEL UP! You reached Level {}!", player.level)
                };
                log_msgs.push(lvl_msg);
            } else {
                player.ap -= ap_cost;
            }

            log_updated = true;
        }
    }

    if log_updated {
        if let Ok(mut log) = log_q.single_mut() {
            for m in log_msgs {
                log.0.push(m);
                if log.0.len() > 3 {
                    log.0.remove(0);
                }
            }
        }
    }
}
