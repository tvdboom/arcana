use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::{Ajah, Class};
use crate::core::constants::*;
use crate::core::localization::*;
use crate::core::menu::buttons::*;
use crate::core::menu::utils::{add_root_node, add_text, recolor, reimage};
use crate::core::pets::Pet;
use crate::core::player::{Attribute, Player, Sex};
use crate::core::races::Race;
use crate::core::settings::{Language, Settings};
use crate::core::weapons::Weapon;
use crate::core::states::GameState;
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::window::SystemCursorIcon;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct SexButton(pub Sex);

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

pub const AGE_TRACK_WIDTH: f32 = 260.0;

#[derive(Component)]
pub struct AgeSliderHandle;

#[derive(Component)]
pub struct AgeValueText;

pub fn update_sex_button_colors(
    player: Res<Player>,
    mut btn_q: Query<(&SexButton, &mut BackgroundColor)>,
) {
    for (btn, mut bg) in &mut btn_q {
        if player.sex == btn.0 {
            bg.0 = HOVERED_BUTTON_COLOR;
        } else {
            bg.0 = NORMAL_BUTTON_COLOR;
        }
    }
}

fn spawn_sex_button(
    parent: &mut ChildSpawnerCommands,
    sex: Sex,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    let label = match sex {
        Sex::Male => localization.get("male", lang),
        Sex::Female => localization.get("female", lang),
    };
    let key_loc = match sex {
        Sex::Male => "male".to_string(),
        Sex::Female => "female".to_string(),
    };

    parent
        .spawn((
            Node {
                width: Val::Px(120.),
                height: Val::Px(38.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.)),
                border_radius: BorderRadius::all(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            SexButton(sex),
        ))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(on_sex_button_click)
        .with_children(|parent| {
            parent.spawn((
                add_text(label, "bold", BUTTON_TEXT_SIZE - 0.5, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(key_loc),
            ));
        });
}

fn on_sex_button_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&SexButton>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut text_q: Query<(&mut Text, &AttributeValueText)>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        if player.sex != btn.0 {
            play_audio_msg.write(PlayAudioMsg::new("button"));
            player.sex = btn.0;

            for (mut text, val_attr) in &mut text_q {
                let val = match val_attr.0 {
                    Attribute::Strength => player.strength(),
                    Attribute::Dexterity => player.dexterity(),
                    Attribute::Constitution => player.constitution(),
                    Attribute::Intelligence => player.intelligence(),
                    Attribute::Wisdom => player.wisdom(),
                    Attribute::Charisma => player.charisma(),
                };
                text.0 = format!("{}", val);
            }
        }
    }
}

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
                Attribute::Strength => player.strength(),
                Attribute::Dexterity => player.dexterity(),
                Attribute::Constitution => player.constitution(),
                Attribute::Intelligence => player.intelligence(),
                Attribute::Wisdom => player.wisdom(),
                Attribute::Charisma => player.charisma(),
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
    mut player: ResMut<Player>,
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
        player.health = player.max_health();
        player.mana = player.max_mana();
        next_game_state.set(GameState::Playing);
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
                                    localization.get("name", lang),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("name".to_string()),
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

                            // Sex selection (Male/Female buttons)
                            parent.spawn((
                                add_text(
                                    localization.get("sex", lang),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("sex".to_string()),
                                Node {
                                    margin: UiRect {
                                        top: percent(5.),
                                        bottom: percent(2.),
                                        ..default()
                                    },
                                    ..default()
                                },
                            ));

                            parent.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                width: Val::Px(260.),
                                ..default()
                            }).with_children(|parent| {
                                spawn_sex_button(parent, Sex::Male, &assets, &localization, lang);
                                spawn_sex_button(parent, Sex::Female, &assets, &localization, lang);
                            });

                            // Age selection
                            let (age_min, age_max) = player.race.age_range();
                            let age = player.age.clamp(age_min, age_max);
                            let frac = if age_max > age_min {
                                (age - age_min) as f32 / (age_max - age_min) as f32
                            } else {
                                0.0
                            };

                            parent.spawn((
                                add_text(
                                    localization.get("age", lang),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("age".to_string()),
                                Node {
                                    margin: UiRect {
                                        top: percent(5.),
                                        bottom: percent(2.),
                                        ..default()
                                    },
                                    ..default()
                                },
                            ));

                            parent.spawn((
                                add_text(
                                    format!("{}", age),
                                    "bold",
                                    BUTTON_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(Color::WHITE),
                                AgeValueText,
                                Node {
                                    margin: UiRect::bottom(percent(2.)),
                                    ..default()
                                },
                            ));

                            // Slider track
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(AGE_TRACK_WIDTH),
                                        height: Val::Px(10.),
                                        border_radius: BorderRadius::all(Val::Px(5.)),
                                        ..default()
                                    },
                                    BackgroundColor(NORMAL_BUTTON_COLOR),
                                ))
                                .with_children(|parent| {
                                    // Draggable handle
                                    parent
                                        .spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                width: Val::Px(24.),
                                                height: Val::Px(24.),
                                                top: Val::Px(-7.),
                                                left: Val::Px(frac * AGE_TRACK_WIDTH - 12.),
                                                border: UiRect::all(Val::Px(2.)),
                                                border_radius: BorderRadius::all(Val::Px(12.)),
                                                ..default()
                                            },
                                            BackgroundColor(BUTTON_TEXT_COLOR),
                                            BorderColor::all(BUTTON_BORDER_COLOR),
                                            AgeSliderHandle,
                                        ))
                                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                                        .observe(on_age_slider_drag);
                                });
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
                                            Attribute::Strength => player.strength(),
                                            Attribute::Dexterity => player.dexterity(),
                                            Attribute::Constitution => player.constitution(),
                                            Attribute::Intelligence => player.intelligence(),
                                            Attribute::Wisdom => player.wisdom(),
                                            Attribute::Charisma => player.charisma(),
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

pub fn on_age_slider_drag(
    ev: On<Pointer<Drag>>,
    mut player: ResMut<Player>,
    mut handle_q: Query<&mut Node, With<AgeSliderHandle>>,
    mut text_q: Query<&mut Text, (With<AgeValueText>, Without<AttributeValueText>)>,
    mut attr_text_q: Query<(&mut Text, &AttributeValueText), Without<AgeValueText>>,
) {
    let (age_min, age_max) = player.race.age_range();

    let Ok(mut node) = handle_q.single_mut() else {
        return;
    };

    let current_left = match node.left {
        Val::Px(px) => px,
        _ => -12.,
    };
    // Handle left is offset by -12 (half its width) from the track position.
    let mut center = current_left + 12. + ev.delta.x;
    center = center.clamp(0., AGE_TRACK_WIDTH);

    let frac = center / AGE_TRACK_WIDTH;
    let age = age_min + (frac * (age_max - age_min) as f32).round() as u32;
    let age = age.clamp(age_min, age_max);

    node.left = Val::Px(center - 12.);
    player.age = age;

    if let Ok(mut text) = text_q.single_mut() {
        text.0 = format!("{}", age);
    }

    for (mut text, val_attr) in &mut attr_text_q {
        let val = match val_attr.0 {
            Attribute::Strength => player.strength(),
            Attribute::Dexterity => player.dexterity(),
            Attribute::Constitution => player.constitution(),
            Attribute::Intelligence => player.intelligence(),
            Attribute::Wisdom => player.wisdom(),
            Attribute::Charisma => player.charisma(),
        };
        text.0 = format!("{}", val);
    }
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
        let (min, max) = self.age_range();
        player.age = (min + max) / 2;
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
            next_game_state.set(GameState::CreateCharacter);
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
        next_game_state.set(GameState::CreateCharacter);
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
        next_game_state.set(GameState::CreateCharacter);
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