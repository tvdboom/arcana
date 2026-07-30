//! Character creation interface for identity, race, class, perks, and starting gear.

use bevy::input::ButtonState;
use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::{all_abilities, all_perks, all_weapons};
use crate::core::catalog::equipment::Kind;
use crate::core::classes::{
    Ajah, AssassinPath, BardStyle, Class, ClassSpecialization, MonkSchool, PetChoice, WarriorPath,
};
use crate::core::constants::*;
use crate::core::deities::{Deity, EthicalAlignment, MoralAlignment};
use crate::core::localization::*;
use crate::core::menu::buttons::*;
use crate::core::menu::utils::{add_root_node, add_text, recolor, reimage};
use crate::core::monsters::MonsterKind;
use crate::core::player::{AgeStage, Attribute, Player, Sex};
use crate::core::races::{ElfHeritage, Race};
use crate::core::settings::{Language, Settings};
use crate::core::states::GameState;
use crate::core::ui::scrollbar::{
    on_scrollbar_thumb_drag_x, HorizontalWheelScroll, ScrollableContainer, ScrollbarThumbX,
    ScrollbarTrackX,
};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::window::SystemCursorIcon;
use rand::prelude::IteratorRandom;
use rand::{rng, RngExt};

const AGE_SLIDER_WIDTH: f32 = 280.0;
const AGE_VALUE_WIDTH: f32 = 240.0;
const MAX_CHARACTER_NAME_CHARS: usize = 16;
const SELECTION_DRAG_SUPPRESSION_SECONDS: f64 = 0.25;
#[cfg(target_arch = "wasm32")]
const MOBILE_NAME_EDITOR_ID: &str = "arcana-name-editor";
#[cfg(target_arch = "wasm32")]
const MOBILE_NAME_INPUT_ID: &str = "arcana-character-name";

#[derive(Component, Clone, Copy)]
pub enum CreationLayoutNode {
    CharacterScreen,
    CharacterTitle,
    CharacterContent,
    IdentityColumn,
    AttributesColumn,
    CharacterFooter,
    SelectionTitle,
    SelectionWrapper,
    SelectionViewport {
        center_cards: bool,
    },
    SelectionCard,
    SelectionFooter,
    DeityScreen,
    DeityTitle,
    DeityCards,
    DeityCard,
    DeityFooter,
}

#[derive(Resource, Default)]
pub struct SelectionGestureState {
    suppress_click_until: f64,
}

impl SelectionGestureState {
    /// Suppresses selection clicks briefly after a card drag.
    pub(crate) fn suppress_after_drag(&mut self, now: f64) {
        self.suppress_click_until = now + SELECTION_DRAG_SUPPRESSION_SECONDS;
    }

    /// Returns whether a drag should still block the synthetic release click.
    pub(crate) fn suppresses_click(&self, now: f64) -> bool {
        now < self.suppress_click_until
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct SexButton(pub Sex);

#[derive(Component)]
pub struct CharacterNameText;

#[derive(Component)]
struct CharacterNameField;

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

#[derive(Component)]
pub struct AgeSliderHandle;

#[derive(Component)]
pub struct AgeSliderTrack;

#[derive(Component)]
pub struct AgeValueText;

#[derive(Component)]
pub struct AgeValueNode;

#[derive(Component, Clone, Copy)]
pub struct AgeStageButton(pub u32);

#[derive(Component, Clone, Copy)]
struct DeityCardImage(MoralAlignment);

#[derive(Component, Clone, Copy)]
struct DeityCardName(MoralAlignment);

#[derive(Component, Clone, Copy)]
struct DeityCardAlignment(MoralAlignment);

#[derive(Component, Clone, Copy)]
struct DeityCardDescription(MoralAlignment);

#[derive(Component, Clone, Copy)]
struct DeityChoiceButton(Deity);

/// Performs the creation attribute value operation.
fn creation_attribute_value(player: &Player, attr: Attribute) -> u32 {
    let value = match attr {
        Attribute::Strength => {
            player.strength as i32 + player.sex.characteristic_mod(Attribute::Strength)
        },
        Attribute::Dexterity => player.dexterity as i32,
        Attribute::Constitution => {
            player.constitution as i32 + player.stage.characteristic_mod(Attribute::Constitution)
        },
        Attribute::Intelligence => player.intelligence as i32,
        Attribute::Wisdom => {
            player.wisdom as i32 + player.stage.characteristic_mod(Attribute::Wisdom)
        },
        Attribute::Charisma => {
            player.charisma as i32 + player.sex.characteristic_mod(Attribute::Charisma)
        },
    };

    value.max(0) as u32
}

/// Updates sex button colors.
pub fn update_sex_button_colors(
    player: Res<Player>,
    mut btn_q: Query<(&SexButton, &Interaction, &mut BackgroundColor)>,
) {
    for (btn, interaction, mut bg) in &mut btn_q {
        bg.0 = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON_COLOR,
            Interaction::Hovered => HOVERED_BUTTON_COLOR,
            Interaction::None if player.sex == btn.0 => HOVERED_BUTTON_COLOR,
            Interaction::None => NORMAL_BUTTON_COLOR,
        };
    }
}

/// Spawns sex button.
fn spawn_sex_button(
    parent: &mut ChildSpawnerCommands,
    sex: Sex,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    let label = localization.get(sex.to_lowername(), lang);
    parent
        .spawn((
            Node {
                min_width: Val::Px(120.),
                height: Val::Px(38.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(12.)),
                border: UiRect::all(Val::Px(2.)),
                border_radius: BorderRadius::all(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Button,
            Interaction::default(),
            SexButton(sex),
        ))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(on_sex_button_click)
        .with_children(|parent| {
            parent.spawn((
                add_text(label, "bold", BUTTON_TEXT_SIZE - 0.5, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(sex.to_lowername()),
            ));
        });
}

/// Handles sex button click.
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
                let val = creation_attribute_value(&player, val_attr.0);
                text.0 = format!("{}", val);
            }
        }
    }
}

/// Appends valid characters without exceeding the character-name limit.
fn append_character_name_text(name: &mut String, input: &str) {
    let remaining = MAX_CHARACTER_NAME_CHARS.saturating_sub(name.chars().count());
    name.extend(sanitize_character_name(input).chars().take(remaining));
}

/// Filters browser text input to the character-name contract.
fn sanitize_character_name(input: &str) -> String {
    input
        .chars()
        .filter(|character| character.is_alphanumeric() || *character == ' ')
        .take(MAX_CHARACTER_NAME_CHARS)
        .collect()
}

/// Handles keyboard name input on native and hardware-keyboard web sessions.
pub fn handle_name_input(
    mut events: MessageReader<KeyboardInput>,
    mut player: ResMut<Player>,
    mut text_q: Query<&mut Text, With<CharacterNameText>>,
) {
    let mut changed = false;
    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Character(c) => {
                let old_len = player.name.len();
                append_character_name_text(&mut player.name, c);
                changed |= player.name.len() != old_len;
            },
            Key::Backspace => {
                changed |= player.name.pop().is_some();
            },
            Key::Space => {
                let old_len = player.name.len();
                append_character_name_text(&mut player.name, " ");
                changed |= player.name.len() != old_len;
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

/// Opens the browser's native character-name editor after tapping the name field.
fn on_character_name_field_click(_: On<Pointer<Click>>, player: Res<Player>) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Some(editor) = document.get_element_by_id(MOBILE_NAME_EDITOR_ID) else {
            return;
        };
        let Some(input) = document
            .get_element_by_id(MOBILE_NAME_INPUT_ID)
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };

        input.set_value(&player.name);
        let _ = editor.remove_attribute("hidden");
        let _ = input.focus();
        input.select();
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = player;
}

/// Synchronizes the browser text field into the Bevy player and label.
#[cfg(target_arch = "wasm32")]
pub fn sync_mobile_name_input(
    mut player: ResMut<Player>,
    mut text_q: Query<&mut Text, With<CharacterNameText>>,
) {
    use wasm_bindgen::JsCast;

    let Some(input) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(MOBILE_NAME_INPUT_ID))
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };

    let browser_value = input.value();
    let sanitized = sanitize_character_name(&browser_value);
    if sanitized != browser_value {
        input.set_value(&sanitized);
    }
    if sanitized == player.name {
        return;
    }

    player.name.clone_from(&sanitized);
    for mut text in &mut text_q {
        text.0.clone_from(&sanitized);
    }
}

/// No-op counterpart for native builds without an HTML text field.
#[cfg(not(target_arch = "wasm32"))]
pub fn sync_mobile_name_input() {}

/// Hides and unfocuses the browser character-name editor.
#[cfg(target_arch = "wasm32")]
pub fn close_mobile_name_editor() {
    use wasm_bindgen::JsCast;

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(input) = document
        .get_element_by_id(MOBILE_NAME_INPUT_ID)
        .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
    {
        let _ = input.blur();
    }
    if let Some(editor) = document.get_element_by_id(MOBILE_NAME_EDITOR_ID) {
        let _ = editor.set_attribute("hidden", "");
    }
}

/// No-op counterpart for native builds without an HTML text field.
#[cfg(not(target_arch = "wasm32"))]
pub fn close_mobile_name_editor() {}

/// Marks a card drag so its release cannot activate the card.
fn suppress_selection_click_after_drag(
    _: On<Pointer<Drag>>,
    time: Res<Time>,
    mut gesture: ResMut<SelectionGestureState>,
) {
    gesture.suppress_after_drag(time.elapsed_secs_f64());
}

/// Handles attribute button click.
fn on_attribute_button_click(
    event: On<Pointer<Click>>,
    btn_q: Query<(Option<&DisabledButton>, &AttributeAction)>,
    mut player: ResMut<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut text_q: Query<(&mut Text, Option<&AttributeValueText>, Option<&PointsRemainingText>)>,
) {
    let Ok((disabled, action)) = btn_q.get(event.entity) else {
        return;
    };
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

                if *val < START_CHARACTERISTIC + 3 {
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

            if *val > START_CHARACTERISTIC.saturating_sub(3) {
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
            let val = creation_attribute_value(&player, val_attr.0);
            text.0 = format!("{}", val as i32);
        } else if remaining_text.is_some() {
            let points_label = localization.get("points remaining", settings.language);
            text.0 = format!("{}: {}", points_label, new_remaining);
        }
    }
}

/// Handles continue click.
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

/// Updates character creation continue btn.
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

/// Updates attribute buttons.
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
                val <= START_CHARACTERISTIC.saturating_sub(3)
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
                val >= START_CHARACTERISTIC + 3 || remaining <= 0
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

/// Spawns attribute button.
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

/// Spawns continue button.
fn spawn_continue_button(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    parent
        .spawn((
            Node {
                min_width: Val::Px(200.),
                height: Val::Px(45.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(16.)),
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

/// Sets up character creation.
pub fn setup_character_creation(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
    let lang = settings.language;
    let (mut root_node, mut pickable) = add_root_node(true);
    root_node.justify_content = JustifyContent::FlexStart;
    pickable.is_hoverable = true;

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode {
                image: assets.image("bg2"),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.55, 0.55, 0.55, 1.0),
                ..default()
            },
            MenuCmp,
            CreationLayoutNode::CharacterScreen,
            ScrollableContainer,
            ScrollPosition::default(),
            Interaction::default(),
            bevy::ui::RelativeCursorPosition::default(),
        ))
        .with_children(|parent| {
            // Title container
            parent
                .spawn((
                    Node {
                        margin: UiRect {
                            top: percent(5.),
                            bottom: percent(3.),
                            ..default()
                        },
                        ..default()
                    },
                    CreationLayoutNode::CharacterTitle,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get("general.create your character", lang),
                            "bold",
                            TITLE_TEXT_SIZE,
                            &assets,
                        ),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText("general.create your character".to_string()),
                    ));
                });

            // Main container (Horizontal row with name selection on the left, attributes on the right)
            parent
                .spawn((
                    Node {
                        width: percent(55.),
                        height: percent(65.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    CreationLayoutNode::CharacterContent,
                ))
                .with_children(|parent| {
                    // Left Column: Name selection
                    parent
                        .spawn((
                            Node {
                                width: percent(45.),
                                height: percent(100.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            CreationLayoutNode::IdentityColumn,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    localization.get("general.name", lang),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("general.name".to_string()),
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
                                    Button,
                                    Interaction::default(),
                                    Pickable::default(),
                                    CharacterNameField,
                                ))
                                .observe(cursor::<Over>(SystemCursorIcon::Text))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .observe(on_character_name_field_click)
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
                                    localization.get("general.change name hint", lang),
                                    "medium",
                                    LABEL_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(Color::srgba_u8(180, 180, 180, 255)),
                                LocalizedText("general.change name hint".to_string()),
                                Node {
                                    margin: UiRect::top(percent(3.)),
                                    ..default()
                                },
                            ));

                            // Sex selection (Male/Female buttons)
                            parent.spawn((
                                add_text(
                                    localization.get("general.sex", lang),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("general.sex".to_string()),
                                Node {
                                    margin: UiRect {
                                        top: percent(5.),
                                        bottom: percent(2.),
                                        ..default()
                                    },
                                    ..default()
                                },
                            ));

                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Px(260.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    spawn_sex_button(
                                        parent,
                                        Sex::Man,
                                        &assets,
                                        &localization,
                                        lang,
                                    );
                                    spawn_sex_button(
                                        parent,
                                        Sex::Woman,
                                        &assets,
                                        &localization,
                                        lang,
                                    );
                                });

                            // Age stage selection
                            parent.spawn((
                                add_text(
                                    localization.get("general.age", lang),
                                    "bold",
                                    SUBTITLE_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("general.age".to_string()),
                                Node {
                                    margin: UiRect {
                                        top: percent(5.),
                                        bottom: percent(2.),
                                        ..default()
                                    },
                                    ..default()
                                },
                            ));

                            // Slider block centered below the Age title.
                            parent
                                .spawn(Node {
                                    width: Val::Px(AGE_SLIDER_WIDTH),
                                    height: Val::Px(76.),
                                    position_type: PositionType::Relative,
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    // Slider track - the whole area is interactive
                                    parent
                                        .spawn((
                                            Node {
                                                width: Val::Px(AGE_SLIDER_WIDTH),
                                                height: Val::Px(44.),
                                                position_type: PositionType::Relative,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            Button,
                                            Interaction::default(),
                                            Pickable::default(),
                                            BackgroundColor(Color::srgba(0., 0., 0., 0.01)),
                                            AgeSliderTrack,
                                        ))
                                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                                        .observe(on_age_slider_click)
                                        .observe(on_age_slider_drag)
                                        .observe(on_age_slider_release)
                                        .with_children(|parent| {
                                            // Track visual bar
                                            parent.spawn((
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: Val::Px(0.),
                                                    top: Val::Px(19.),
                                                    width: percent(100.),
                                                    height: Val::Px(6.),
                                                    border_radius: BorderRadius::all(Val::Px(3.)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgba_u8(60, 60, 80, 200)),
                                                Pickable::IGNORE,
                                            ));

                                            // Notch markers
                                            for i in 0..5 {
                                                let notch_x = (i as f32 / 4.0) * AGE_SLIDER_WIDTH;
                                                parent.spawn((
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        left: Val::Px(notch_x - 2.),
                                                        top: Val::Px(12.),
                                                        width: Val::Px(4.),
                                                        height: Val::Px(20.),
                                                        border_radius: BorderRadius::all(Val::Px(
                                                            2.,
                                                        )),
                                                        ..default()
                                                    },
                                                    BackgroundColor(BUTTON_BORDER_COLOR),
                                                    Pickable::IGNORE,
                                                ));
                                            }

                                            // Invisible but pickable zones for each stage. These make
                                            // the slider work even when the parent track's cursor math
                                            // or Interaction state is not updated by the UI picker.
                                            for i in 0..5 {
                                                let (left, width) = match i {
                                                    0 => (0., AGE_SLIDER_WIDTH / 8.),
                                                    4 => (
                                                        AGE_SLIDER_WIDTH * 7. / 8.,
                                                        AGE_SLIDER_WIDTH / 8.,
                                                    ),
                                                    _ => (
                                                        (i as f32 - 0.5) * AGE_SLIDER_WIDTH / 4.,
                                                        AGE_SLIDER_WIDTH / 4.,
                                                    ),
                                                };

                                                parent
                                                    .spawn((
                                                        Node {
                                                            position_type: PositionType::Absolute,
                                                            left: Val::Px(left),
                                                            top: Val::Px(0.),
                                                            width: Val::Px(width),
                                                            height: Val::Px(44.),
                                                            ..default()
                                                        },
                                                        Button,
                                                        Interaction::default(),
                                                        Pickable::default(),
                                                        BackgroundColor(Color::srgba(
                                                            0., 0., 0., 0.01,
                                                        )),
                                                        AgeStageButton(i),
                                                    ))
                                                    .observe(cursor::<Over>(
                                                        SystemCursorIcon::Pointer,
                                                    ))
                                                    .observe(cursor::<Out>(
                                                        SystemCursorIcon::Default,
                                                    ))
                                                    .observe(on_age_stage_click)
                                                    .observe(on_age_slider_drag)
                                                    .observe(on_age_slider_release);
                                            }

                                            // Handle (visual only)
                                            parent
                                                .spawn((
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        width: Val::Px(24.),
                                                        height: Val::Px(24.),
                                                        top: Val::Px(10.),
                                                        left: Val::Px(
                                                            player.stage.frac() * AGE_SLIDER_WIDTH
                                                                - 12.,
                                                        ),
                                                        border: UiRect::all(Val::Px(2.)),
                                                        border_radius: BorderRadius::all(Val::Px(
                                                            12.,
                                                        )),
                                                        ..default()
                                                    },
                                                    BackgroundColor(BUTTON_TEXT_COLOR),
                                                    BorderColor::all(BUTTON_BORDER_COLOR),
                                                    Button,
                                                    Interaction::default(),
                                                    Pickable::default(),
                                                    AgeSliderHandle,
                                                ))
                                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                                .observe(on_age_slider_drag)
                                                .observe(on_age_slider_release);
                                        });

                                    // Label showing current stage, positioned below the selected point.
                                    parent
                                        .spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                top: Val::Px(50.),
                                                left: Val::Px(
                                                    AGE_SLIDER_WIDTH / 2. - AGE_VALUE_WIDTH / 2.,
                                                ),
                                                width: Val::Px(AGE_VALUE_WIDTH),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            AgeValueNode,
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                add_text(
                                                    localization.get(
                                                        format!(
                                                            "general.{}",
                                                            player
                                                                .stage
                                                                .to_lowername()
                                                                .replace(" ", "_")
                                                        ),
                                                        lang,
                                                    ),
                                                    "bold",
                                                    BUTTON_TEXT_SIZE,
                                                    &assets,
                                                ),
                                                TextColor(BUTTON_TEXT_COLOR),
                                                TextLayout::justify(Justify::Center),
                                                AgeValueText,
                                            ));
                                        });
                                });
                        });

                    // Right Column: Attribute allocation
                    parent
                        .spawn((
                            Node {
                                width: percent(45.),
                                height: percent(100.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            CreationLayoutNode::AttributesColumn,
                        ))
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

                            let points_label = localization.get("general.points_remaining", lang);
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
                                        let translated_attr_name = localization.get(
                                            format!("attribute.{}", attr.to_lowername()),
                                            lang,
                                        );
                                        let val = creation_attribute_value(&player, attr) as i32;

                                        // Row for this attribute
                                        parent
                                            .spawn(Node {
                                                width: percent(88.),
                                                height: Val::Px(45.),
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::SpaceBetween,
                                                column_gap: Val::Px(18.),
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
                                                    LocalizedText(format!(
                                                        "attribute.{}",
                                                        attr.to_lowername()
                                                    )),
                                                    Node {
                                                        width: percent(55.),
                                                        ..default()
                                                    },
                                                ));

                                                // Controls (Minus, Value, Plus)
                                                parent
                                                    .spawn(Node {
                                                        width: percent(42.),
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
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(100.),
                        bottom: percent(4.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    CreationLayoutNode::CharacterFooter,
                ))
                .with_children(|parent| {
                    // Back button
                    spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);

                    // Continue button
                    spawn_continue_button(parent, &assets, &localization, lang);
                });
        });
}

/// Handles age slider drag.
fn on_age_slider_drag(
    ev: On<Pointer<Drag>>,
    mut player: ResMut<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut handle_q: Query<&mut Node, (With<AgeSliderHandle>, Without<AgeSliderTrack>)>,
    mut value_node_q: Query<
        &mut Node,
        (With<AgeValueNode>, Without<AgeSliderHandle>, Without<AgeSliderTrack>),
    >,
    mut text_q: Query<&mut Text, (With<AgeValueText>, Without<AttributeValueText>)>,
    mut attr_text_q: Query<(&mut Text, &AttributeValueText), Without<AgeValueText>>,
) {
    let current_left = {
        let Ok(handle_node) = handle_q.single_mut() else {
            return;
        };
        match handle_node.left {
            Val::Px(px) => px,
            _ => -12.,
        }
    };
    let relative_x = current_left + 12. + ev.delta.x;
    set_age_slider_position(&mut handle_q, &mut value_node_q, relative_x);
    let stage = age_stage_from_relative_x(relative_x);
    set_age_value_position(&mut value_node_q, stage as f32 / 4.0 * AGE_SLIDER_WIDTH);

    // Generate random age within the range for this race and stage
    let age_stage = AgeStage::from_u32(stage);
    let (min_age, max_age) = player.race.age_stage_range(age_stage);
    player.stage = age_stage;
    player.age = rng().random_range(min_age..=max_age);

    if let Ok(mut text) = text_q.single_mut() {
        text.0 = localization.get(
            format!("general.{}", age_stage.to_lowername().replace(" ", "_")),
            settings.language,
        );
    }

    for (mut text, val_attr) in attr_text_q.iter_mut() {
        let val = creation_attribute_value(&player, val_attr.0);
        text.0 = format!("{}", val);
    }
}

/// Handles age slider release.
fn on_age_slider_release(
    _: On<Pointer<DragEnd>>,
    mut player: ResMut<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut handle_q: Query<&mut Node, (With<AgeSliderHandle>, Without<AgeSliderTrack>)>,
    mut value_node_q: Query<
        &mut Node,
        (With<AgeValueNode>, Without<AgeSliderHandle>, Without<AgeSliderTrack>),
    >,
    mut text_q: Query<&mut Text, (With<AgeValueText>, Without<AttributeValueText>)>,
    mut attr_text_q: Query<(&mut Text, &AttributeValueText), Without<AgeValueText>>,
) {
    let relative_x = {
        let Ok(handle_node) = handle_q.single_mut() else {
            return;
        };
        match handle_node.left {
            Val::Px(px) => px + 12.,
            _ => player.stage.frac() * AGE_SLIDER_WIDTH,
        }
    };
    let stage = age_stage_from_relative_x(relative_x);
    apply_age_stage(
        stage,
        &mut player,
        &settings,
        &localization,
        &mut handle_q,
        &mut value_node_q,
        &mut text_q,
        &mut attr_text_q,
    );
}

/// Handles age slider click.
fn on_age_slider_click(
    _: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    track_q: Query<&GlobalTransform, With<AgeSliderTrack>>,
    mut handle_q: Query<&mut Node, (With<AgeSliderHandle>, Without<AgeSliderTrack>)>,
    mut value_node_q: Query<
        &mut Node,
        (With<AgeValueNode>, Without<AgeSliderHandle>, Without<AgeSliderTrack>),
    >,
    mut text_q: Query<&mut Text, (With<AgeValueText>, Without<AttributeValueText>)>,
    mut attr_text_q: Query<(&mut Text, &AttributeValueText), Without<AgeValueText>>,
    windows: Query<&Window>,
) {
    let Ok(track_transform) = track_q.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let stage = age_stage_from_cursor(track_transform, window, cursor_pos.x);
    apply_age_stage(
        stage,
        &mut player,
        &settings,
        &localization,
        &mut handle_q,
        &mut value_node_q,
        &mut text_q,
        &mut attr_text_q,
    );
}

/// Handles age stage click.
fn on_age_stage_click(
    event: On<Pointer<Click>>,
    stage_q: Query<&AgeStageButton>,
    mut player: ResMut<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut handle_q: Query<&mut Node, (With<AgeSliderHandle>, Without<AgeSliderTrack>)>,
    mut value_node_q: Query<
        &mut Node,
        (With<AgeValueNode>, Without<AgeSliderHandle>, Without<AgeSliderTrack>),
    >,
    mut text_q: Query<&mut Text, (With<AgeValueText>, Without<AttributeValueText>)>,
    mut attr_text_q: Query<(&mut Text, &AttributeValueText), Without<AgeValueText>>,
) {
    let Ok(stage) = stage_q.get(event.entity) else {
        return;
    };

    apply_age_stage(
        stage.0,
        &mut player,
        &settings,
        &localization,
        &mut handle_q,
        &mut value_node_q,
        &mut text_q,
        &mut attr_text_q,
    );
}

/// Performs the age stage from cursor operation.
fn age_stage_from_cursor(track_transform: &GlobalTransform, window: &Window, cursor_x: f32) -> u32 {
    age_stage_from_relative_x(age_relative_x_from_cursor(track_transform, window, cursor_x))
}

/// Performs the age relative x from cursor operation.
fn age_relative_x_from_cursor(
    track_transform: &GlobalTransform,
    window: &Window,
    cursor_x: f32,
) -> f32 {
    // UI transforms are centered around the window, while cursor positions start at the
    // window's left edge. Convert the track center into cursor-space.
    let track_center_x = track_transform.translation().x + window.width() / 2.0;
    let track_left = track_center_x - AGE_SLIDER_WIDTH / 2.0;
    (cursor_x - track_left).clamp(0., AGE_SLIDER_WIDTH)
}

/// Performs the age stage from relative x operation.
fn age_stage_from_relative_x(relative_x: f32) -> u32 {
    // Snap to nearest of 5 positions.
    let frac = relative_x / AGE_SLIDER_WIDTH;
    ((frac * 4.0).round() as u32).clamp(0, 4)
}

/// Performs the set age slider position operation.
fn set_age_slider_position(
    handle_q: &mut Query<&mut Node, (With<AgeSliderHandle>, Without<AgeSliderTrack>)>,
    value_node_q: &mut Query<
        &mut Node,
        (With<AgeValueNode>, Without<AgeSliderHandle>, Without<AgeSliderTrack>),
    >,
    relative_x: f32,
) {
    let relative_x = relative_x.clamp(0., AGE_SLIDER_WIDTH);
    if let Ok(mut handle_node) = handle_q.single_mut() {
        handle_node.left = Val::Px(relative_x - 12.);
    }
    set_age_value_position(value_node_q, relative_x);
}

/// Performs the set age value position operation.
fn set_age_value_position(
    value_node_q: &mut Query<
        &mut Node,
        (With<AgeValueNode>, Without<AgeSliderHandle>, Without<AgeSliderTrack>),
    >,
    relative_x: f32,
) {
    let relative_x = relative_x.clamp(0., AGE_SLIDER_WIDTH);
    if let Ok(mut value_node) = value_node_q.single_mut() {
        value_node.left = Val::Px(relative_x - AGE_VALUE_WIDTH / 2.);
    }
}

/// Applies age stage.
fn apply_age_stage(
    stage: u32,
    player: &mut Player,
    settings: &Settings,
    localization: &Localization,
    handle_q: &mut Query<&mut Node, (With<AgeSliderHandle>, Without<AgeSliderTrack>)>,
    value_node_q: &mut Query<
        &mut Node,
        (With<AgeValueNode>, Without<AgeSliderHandle>, Without<AgeSliderTrack>),
    >,
    text_q: &mut Query<&mut Text, (With<AgeValueText>, Without<AttributeValueText>)>,
    attr_text_q: &mut Query<(&mut Text, &AttributeValueText), Without<AgeValueText>>,
) {
    // Generate random age within the range for this race and stage
    let age_stage = AgeStage::from_u32(stage);
    let (min_age, max_age) = player.race.age_stage_range(age_stage);
    let new_age = rng().random_range(min_age..=max_age);
    player.stage = age_stage;
    player.age = new_age;
    let snapped_frac = stage as f32 / 4.0;
    set_age_slider_position(handle_q, value_node_q, snapped_frac * AGE_SLIDER_WIDTH);

    if let Ok(mut text) = text_q.single_mut() {
        text.0 = localization.get(
            format!("general.{}", age_stage.to_lowername().replace(" ", "_")),
            settings.language,
        );
    }

    for (mut text, val_attr) in attr_text_q.iter_mut() {
        let val = creation_attribute_value(player, val_attr.0);
        text.0 = format!("{}", val);
    }
}

pub trait SelectionItem:
    'static + NameFromEnum + Copy + Clone + Send + Sync + IntoEnumIterator
{
    type DescComponent: Component;
    /// Returns description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String;
    /// Creates desc component.
    fn create_desc_component(&self) -> Self::DescComponent;
    /// Handles select.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>);
    /// Returns image key.
    fn get_image_key(&self, _player: &Player) -> String {
        self.to_lowername()
    }
    /// Performs the items operation.
    fn items() -> Vec<Self>
    where
        Self: Sized,
    {
        Self::iter().collect()
    }
}

impl SelectionItem for Race {
    type DescComponent = LocalizedRaceDesc;

    /// Returns description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_race_description(*self, lang, localization)
    }

    /// Creates desc component.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedRaceDesc(*self)
    }

    /// Handles select.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        let stage = player.stage;
        player.race = *self;
        let (min_age, max_age) = player.race.age_stage_range(stage);
        player.age = rng().random_range(min_age..=max_age);
        if *self == Race::Elf {
            next_game_state.set(GameState::ChooseElfHeritage);
        } else {
            next_game_state.set(GameState::ChooseClass);
        }
    }

    /// Returns image key.
    fn get_image_key(&self, player: &Player) -> String {
        format!("{}_{}", self.to_lowername(), player.sex.to_lowername())
    }
}

impl SelectionItem for ElfHeritage {
    type DescComponent = LocalizedElfHeritageDesc;

    /// Returns this heritage's localized gameplay description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_elf_heritage_description(*self, lang, localization)
    }

    /// Creates the component used to refresh this description after a language change.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedElfHeritageDesc(*self)
    }

    /// Stores the selected elven heritage and advances to class selection.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        player.elf_heritage = *self;
        next_game_state.set(GameState::ChooseClass);
    }

    /// Returns the portrait dedicated to this heritage and selected sex.
    fn get_image_key(&self, player: &Player) -> String {
        format!("elf_{}_{}", self.to_lowername().replace(' ', "_"), player.sex.to_lowername())
    }
}

impl SelectionItem for Class {
    type DescComponent = LocalizedClassDesc;

    /// Returns description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_class_description(*self, lang, localization)
    }

    /// Creates desc component.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedClassDesc(*self)
    }

    /// Handles select.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        let mut random = rng();
        let lowest_ability_level = all_abilities()
            .iter()
            .filter(|ability| self.accepts_starting_ability(ability.kind))
            .map(|ability| ability.level)
            .min();
        let ability = lowest_ability_level.and_then(|level| {
            all_abilities()
                .iter()
                .filter(|ability| {
                    ability.level == level && self.accepts_starting_ability(ability.kind)
                })
                .choose(&mut random)
        });
        let perk = all_perks().iter().filter(|perk| perk.level == 1).choose(&mut random);
        let weapon = all_weapons()
            .iter()
            .filter(|weapon| weapon.level < 3 && self.accepts_starting_weapon(weapon.category))
            .choose(&mut random);
        let (Some(ability), Some(perk), Some(weapon)) = (ability, perk, weapon) else {
            return;
        };

        player.class = *self;
        player.specialization = self.default_specialization();
        player.pet = None;
        player.abilities = vec![ability.name.clone()];
        player.perks = vec![perk.name.clone()];
        player.weapon_lh = Some(weapon.name.clone());
        player.missing_health = 0;
        player.missing_mana = 0;

        next_game_state.set(GameState::ChooseSubClass);
    }

    /// Returns image key.
    fn get_image_key(&self, player: &Player) -> String {
        let race_key = player.race.to_lowername();
        let sex_key = player.sex.to_lowername();
        match self {
            Class::Mage(_) => format!("mage_{}_{}", race_key, sex_key),
            Class::Warrior => format!("warrior_{}_{}", race_key, sex_key),
            Class::Assassin => format!("assassin_{}_{}", race_key, sex_key),
            Class::Druid => format!("druid_{}_{}", race_key, sex_key),
            Class::Monk => format!("monk_{}_{}", race_key, sex_key),
            Class::Bard => format!("bard_{}_{}", race_key, sex_key),
        }
    }
}

impl SelectionItem for Ajah {
    type DescComponent = LocalizedAjahDesc;

    /// Returns description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_ajah_description(*self, lang, localization)
    }

    /// Creates desc component.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedAjahDesc(*self)
    }

    /// Handles select.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        player.class = Class::Mage(*self);
        player.specialization = ClassSpecialization::Mage(*self);
        player.abilities.truncate(1);

        let ability = all_abilities()
            .iter()
            .filter(|a| {
                a.level < 3
                    && match *self {
                        Ajah::Black => a.kind == Kind::Shadow,
                        Ajah::Green => a.kind == Kind::Nature,
                        Ajah::Red => a.kind == Kind::Fire,
                        Ajah::White => a.kind == Kind::Ice,
                    }
                    && !player.abilities.contains(&a.name)
            })
            .choose(&mut rng())
            .unwrap();

        player.abilities.push(ability.name.clone());
        next_game_state.set(GameState::ChooseDeity);
    }

    /// Returns image key.
    fn get_image_key(&self, player: &Player) -> String {
        let race_key = player.race.to_lowername();
        let sex_key = match player.sex {
            Sex::Man => "man",
            Sex::Woman => "woman",
        };
        match self {
            Ajah::Black => format!("mage_black_{}_{}", race_key, sex_key),
            Ajah::Red => format!("mage_red_{}_{}", race_key, sex_key),
            Ajah::Green => format!("mage_green_{}_{}", race_key, sex_key),
            Ajah::White => format!("mage_white_{}_{}", race_key, sex_key),
        }
    }
}

/// Returns the portrait used by a non-mage specialization card.
fn specialization_portrait_key(
    class_key: &str,
    specialization_key: Option<&str>,
    player: &Player,
) -> String {
    let race = player.race.to_lowername();
    let sex = player.sex.to_lowername();
    match specialization_key {
        Some(specialization) => format!("{class_key}_{specialization}_{race}_{sex}"),
        None => format!("{class_key}_{race}_{sex}"),
    }
}

/// Stores a specialization and advances to deity selection.
fn choose_specialization(
    player: &mut Player,
    specialization: ClassSpecialization,
    next_game_state: &mut NextState<GameState>,
) {
    player.specialization = specialization;
    next_game_state.set(GameState::ChooseDeity);
}

impl SelectionItem for PetChoice {
    type DescComponent = LocalizedPetDesc;

    /// Returns description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_pet_description(*self, lang, localization)
    }

    /// Creates desc component.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedPetDesc(*self)
    }

    /// Handles select.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        if let Some(pet_monster) = crate::core::catalog::catalog::get_monster(self.monster_name()) {
            player.pet = Some(pet_monster);
        }
        choose_specialization(player, ClassSpecialization::Druid(*self), next_game_state);
    }

    /// Performs the items operation.
    fn items() -> Vec<Self> {
        vec![
            PetChoice::Rat,
            PetChoice::Owl,
            PetChoice::Snake,
            PetChoice::Weasel,
            PetChoice::Fox,
            PetChoice::Raven,
        ]
    }
}

macro_rules! impl_specialization_selection {
    ($ty:ty, $variant:ident, $class_key:literal, $specific_portrait:literal) => {
        impl SelectionItem for $ty {
            type DescComponent = LocalizedSpecializationDesc;

            /// Returns this specialization's localized gameplay description.
            fn get_description(&self, lang: Language, localization: &Localization) -> String {
                format_specialization_description(
                    ClassSpecialization::$variant(*self),
                    lang,
                    localization,
                )
            }

            /// Creates the component used to refresh this description after a language change.
            fn create_desc_component(&self) -> Self::DescComponent {
                LocalizedSpecializationDesc(ClassSpecialization::$variant(*self))
            }

            /// Stores this specialization and advances to deity selection.
            fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
                choose_specialization(
                    player,
                    ClassSpecialization::$variant(*self),
                    next_game_state,
                );
            }

            /// Returns the selected race and sex portrait for this class.
            fn get_image_key(&self, player: &Player) -> String {
                let specialization = self.to_lowername().replace(' ', "_");
                specialization_portrait_key(
                    $class_key,
                    $specific_portrait.then_some(specialization.as_str()),
                    player,
                )
            }
        }
    };
}

impl_specialization_selection!(AssassinPath, Assassin, "assassin", true);
impl_specialization_selection!(BardStyle, Bard, "bard", true);

impl SelectionItem for WarriorPath {
    type DescComponent = LocalizedSpecializationDesc;

    /// Returns this warrior path's localized gameplay description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_specialization_description(ClassSpecialization::Warrior(*self), lang, localization)
    }

    /// Creates the component used to refresh this description after a language change.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedSpecializationDesc(ClassSpecialization::Warrior(*self))
    }

    /// Stores this warrior path and advances to deity selection.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        choose_specialization(player, ClassSpecialization::Warrior(*self), next_game_state);
    }

    /// Returns the dedicated race- and sex-specific portrait for this warrior path.
    fn get_image_key(&self, player: &Player) -> String {
        let specialization = match self {
            WarriorPath::Paladin => Some("paladin"),
            WarriorPath::Templar => Some("templar"),
            WarriorPath::Berserker => Some("berserker"),
            WarriorPath::Warden => Some("warden"),
        };
        specialization_portrait_key("warrior", specialization, player)
    }
}

impl SelectionItem for MonkSchool {
    type DescComponent = LocalizedSpecializationDesc;

    /// Returns this monk school's localized gameplay description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_specialization_description(ClassSpecialization::Monk(*self), lang, localization)
    }

    /// Creates the component used to refresh this description after a language change.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedSpecializationDesc(ClassSpecialization::Monk(*self))
    }

    /// Stores this monk school and advances to deity selection.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        choose_specialization(player, ClassSpecialization::Monk(*self), next_game_state);
    }

    /// Returns the school, race, and sex-specific monk portrait.
    fn get_image_key(&self, player: &Player) -> String {
        format!(
            "monk_{}_{}_{}",
            self.to_lowername().replace(' ', "_"),
            player.race.to_lowername(),
            player.sex.to_lowername()
        )
    }
}

impl SelectionItem for MonsterKind {
    type DescComponent = LocalizedMonsterKindDesc;

    /// Returns description.
    fn get_description(&self, lang: Language, localization: &Localization) -> String {
        format_monster_kind_description(*self, lang, localization)
    }

    /// Creates desc component.
    fn create_desc_component(&self) -> Self::DescComponent {
        LocalizedMonsterKindDesc(*self)
    }

    /// Handles select.
    fn on_select(&self, player: &mut Player, next_game_state: &mut NextState<GameState>) {
        if let Some(ref mut pet) = player.pet {
            pet.kind = *self;
        }
        next_game_state.set(GameState::Playing);
    }
}

/// Sets up selection screen.
pub fn setup_selection_screen<T: SelectionItem>(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    title_key: &'static str,
    has_back_button: bool,
    player: &Player,
) {
    let lang = settings.language;
    let items = T::items();
    let center_cards = items.len() <= 3;
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
                .spawn((
                    Node {
                        margin: UiRect {
                            top: percent(3.),
                            bottom: percent(3.),
                            ..default()
                        },
                        ..default()
                    },
                    CreationLayoutNode::SelectionTitle,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get(title_key, lang),
                            "bold",
                            TITLE_TEXT_SIZE,
                            &assets,
                        ),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText(title_key.to_string()),
                    ));
                });

            // Scrollable card viewport. Four cards fit exactly; additional entries extend the
            // content width and reveal the draggable horizontal scrollbar.
            parent
                .spawn((
                    Node {
                        width: percent(96.),
                        height: percent(72.),
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    CreationLayoutNode::SelectionWrapper,
                ))
                .with_children(|wrapper| {
                    let container_entity = wrapper
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: percent(96.),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::NoWrap,
                                justify_content: if center_cards {
                                    JustifyContent::Center
                                } else {
                                    JustifyContent::FlexStart
                                },
                                align_items: AlignItems::Center,
                                overflow: Overflow::scroll_x(),
                                ..default()
                            },
                            ScrollableContainer,
                            HorizontalWheelScroll,
                            ScrollPosition::default(),
                            Interaction::default(),
                            Pickable::default(),
                            bevy::ui::RelativeCursorPosition::default(),
                            CreationLayoutNode::SelectionViewport {
                                center_cards,
                            },
                        ))
                        .with_children(|parent| {
                            for item in items {
                                let prefix = match title_key {
                                    "choose race" => "race",
                                    "choose class" => "class",
                                    "choose elf heritage" => "heritage",
                                    "choose subclass" => "ajah",
                                    "choose pet" => "pet",
                                    "choose assassin path"
                                    | "choose warrior path"
                                    | "choose monk school"
                                    | "choose bard style" => "specialization",
                                    _ => "",
                                };
                                let item_key = if prefix.is_empty() {
                                    item.to_lowername()
                                } else {
                                    format!("{}.{}", prefix, item.to_lowername())
                                };
                                let item_name = localization.get(&item_key, lang);

                                parent
                                    .spawn((
                                        Node {
                                            width: percent(22.),
                                            height: percent(94.),
                                            position_type: PositionType::Relative,
                                            margin: UiRect::horizontal(percent(1.5)),
                                            flex_shrink: 0.,
                                            ..default()
                                        },
                                        BackgroundColor(NORMAL_BUTTON_COLOR),
                                        CreationLayoutNode::SelectionCard,
                                    ))
                                    .with_children(|parent| {
                                        parent
                                            .spawn(Node {
                                                width: percent(100.),
                                                height: percent(100.),
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::FlexStart,
                                                padding: UiRect::all(percent(1.5)),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    Node {
                                                        width: percent(100.),
                                                        height: percent(50.),
                                                        ..default()
                                                    },
                                                    ImageNode::new(
                                                        assets.image(item.get_image_key(player)),
                                                    )
                                                    .with_mode(NodeImageMode::Stretch),
                                                ));

                                                parent
                                                    .spawn((
                                                        Node {
                                                            width: percent(100.),
                                                            height: percent(50.),
                                                            flex_direction: FlexDirection::Column,
                                                            align_items: AlignItems::Center,
                                                            justify_content:
                                                                JustifyContent::FlexStart,
                                                            ..default()
                                                        },
                                                        ImageNode::new(assets.image("stone"))
                                                            .with_mode(NodeImageMode::Stretch),
                                                    ))
                                                    .with_children(|parent| {
                                                        parent.spawn((
                                                            Node {
                                                                margin: UiRect::vertical(percent(
                                                                    4.5,
                                                                )),
                                                                ..default()
                                                            },
                                                            add_text(
                                                                item_name,
                                                                "bold",
                                                                SUBTITLE_TEXT_SIZE,
                                                                &assets,
                                                            ),
                                                            TextColor(BUTTON_TEXT_COLOR),
                                                            LocalizedText(item_key.clone()),
                                                        ));

                                                        parent.spawn((
                                                            Node {
                                                                width: percent(85.),
                                                                margin: UiRect::horizontal(
                                                                    percent(7.5),
                                                                ),
                                                                ..default()
                                                            },
                                                            add_text(
                                                                item.get_description(
                                                                    lang,
                                                                    &localization,
                                                                ),
                                                                "medium",
                                                                1.8,
                                                                &assets,
                                                            ),
                                                            TextColor(Color::WHITE),
                                                            item.create_desc_component(),
                                                        ));
                                                    });
                                            });

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
                                                ImageNode::new(assets.image("border"))
                                                    .with_mode(NodeImageMode::Stretch),
                                            ))
                                            .observe(reimage::<Over>(assets.image("border_hover")))
                                            .observe(reimage::<Out>(assets.image("border")))
                                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                                            .observe(suppress_selection_click_after_drag)
                                            .observe(
                                                move |_: On<Pointer<Click>>,
                                                      mut player: ResMut<Player>,
                                                      mut play_audio_msg: MessageWriter<
                                                    PlayAudioMsg,
                                                >,
                                                      mut next_game_state: ResMut<
                                                    NextState<GameState>,
                                                >,
                                                      time: Res<Time>,
                                                      gesture: Res<SelectionGestureState>| {
                                                    if gesture.suppresses_click(
                                                        time.elapsed_secs_f64(),
                                                    ) {
                                                        return;
                                                    }
                                                    play_audio_msg
                                                        .write(PlayAudioMsg::new("button"));

                                                    item.on_select(
                                                        &mut player,
                                                        &mut next_game_state,
                                                    );
                                                },
                                            );
                                    });
                            }
                        })
                        .id();

                    wrapper
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                height: Val::Px(8.),
                                left: percent(1.5),
                                right: percent(1.5),
                                bottom: Val::Px(0.),
                                border_radius: BorderRadius::all(Val::Px(4.)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba_u8(0, 0, 0, 190)),
                            Visibility::Hidden,
                            ScrollbarTrackX {
                                container: container_entity,
                            },
                        ))
                        .with_children(|track| {
                            track
                                .spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        height: percent(100.),
                                        width: Val::Px(64.),
                                        left: Val::Px(0.),
                                        border_radius: BorderRadius::all(Val::Px(4.)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba_u8(230, 205, 120, 240)),
                                    Button,
                                    Interaction::default(),
                                    Pickable::default(),
                                    ScrollbarThumbX {
                                        container: container_entity,
                                    },
                                ))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .observe(on_scrollbar_thumb_drag_x);
                        });
                });

            // Back button container centered horizontally at the bottom of the screen
            if has_back_button {
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100.),
                            bottom: percent(3.),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        CreationLayoutNode::SelectionFooter,
                    ))
                    .with_children(|parent| {
                        spawn_menu_button(parent, MenuBtn::Back, &assets, &localization, lang);
                    });
            }
        });
}

/// Sets up race selection.
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

/// Sets up the heritage selection shown only after choosing Elf.
pub fn setup_elf_heritage_selection(
    commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
    setup_selection_screen::<ElfHeritage>(
        commands,
        settings,
        assets,
        localization,
        "choose elf heritage",
        true,
        &player,
    );
}

/// Sets up class selection.
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

/// Sets up subclass selection.
pub fn setup_subclass_selection(
    commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    mut player: ResMut<Player>,
) {
    match player.class {
        Class::Assassin => {
            setup_selection_screen::<AssassinPath>(
                commands,
                settings,
                assets,
                localization,
                "choose assassin path",
                true,
                &player,
            );
        },
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
            if player.pet.is_none() {
                if let Some(pet_monster) = crate::core::catalog::catalog::get_monster("Wolf") {
                    player.pet = Some(pet_monster);
                }
            }
            setup_selection_screen::<PetChoice>(
                commands,
                settings,
                assets,
                localization,
                "choose pet",
                true,
                &player,
            );
        },
        Class::Warrior => {
            setup_selection_screen::<WarriorPath>(
                commands,
                settings,
                assets,
                localization,
                "choose warrior path",
                true,
                &player,
            );
        },
        Class::Monk => {
            setup_selection_screen::<MonkSchool>(
                commands,
                settings,
                assets,
                localization,
                "choose monk school",
                true,
                &player,
            );
        },
        Class::Bard => {
            setup_selection_screen::<BardStyle>(
                commands,
                settings,
                assets,
                localization,
                "choose bard style",
                true,
                &player,
            );
        },
    }
}

/// Sets up the three-card deity alignment selector.
pub fn setup_deity_selection(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
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
            CreationLayoutNode::DeityScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    margin: UiRect::vertical(percent(2.5)),
                    ..default()
                },
                add_text(localization.get("choose deity", lang), "bold", TITLE_TEXT_SIZE, &assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText("choose deity".to_string()),
                CreationLayoutNode::DeityTitle,
            ));

            parent
                .spawn((
                    Node {
                        width: percent(90.),
                        height: percent(76.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Stretch,
                        column_gap: percent(2.),
                        ..default()
                    },
                    CreationLayoutNode::DeityCards,
                    ScrollableContainer,
                    HorizontalWheelScroll,
                    ScrollPosition::default(),
                    Interaction::default(),
                    Pickable::default(),
                    bevy::ui::RelativeCursorPosition::default(),
                ))
                .with_children(|cards| {
                    for moral in MoralAlignment::iter() {
                        let shown_deity = Deity::from_alignment(moral, EthicalAlignment::Neutral);
                        spawn_deity_alignment_card(
                            cards,
                            &assets,
                            &localization,
                            lang,
                            moral,
                            shown_deity,
                        );
                    }
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(100.),
                        bottom: percent(2.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(24.),
                        ..default()
                    },
                    CreationLayoutNode::DeityFooter,
                ))
                .with_children(|buttons| {
                    spawn_menu_button(buttons, MenuBtn::Back, &assets, &localization, lang);
                });
        });
}

/// Spawns one moral-alignment card with three ethical-alignment choices.
fn spawn_deity_alignment_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    moral: MoralAlignment,
    shown_deity: Deity,
) {
    parent
        .spawn((
            Node {
                width: percent(26.),
                height: percent(94.),
                position_type: PositionType::Relative,
                margin: UiRect::horizontal(percent(1.5)),
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            CreationLayoutNode::DeityCard,
        ))
        .with_children(|card| {
            card.spawn(Node {
                width: percent(100.),
                height: percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(percent(1.5)),
                ..default()
            })
            .with_children(|content| {
                content
                    .spawn((
                        Node {
                            width: percent(100.),
                            height: percent(48.),
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        ImageNode::new(assets.image(shown_deity.image_key()))
                            .with_mode(NodeImageMode::Stretch),
                        DeityCardImage(moral),
                    ))
                    .with_children(|portrait| {
                        portrait.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(100.),
                                left: percent(0.),
                                top: percent(3.),
                                justify_content: JustifyContent::Center,
                                padding: UiRect::horizontal(percent(4.)),
                                ..default()
                            },
                            add_text(
                                localization
                                    .get(format!("deity.{}", shown_deity.to_lowername()), lang),
                                "bold",
                                2.1,
                                assets,
                            ),
                            TextColor(BUTTON_TEXT_COLOR),
                            TextLayout::justify(Justify::Center),
                            TextShadow::default(),
                            Pickable::IGNORE,
                            DeityCardName(moral),
                        ));
                    });

                content
                    .spawn((
                        Node {
                            width: percent(100.),
                            height: percent(52.),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexStart,
                            ..default()
                        },
                        ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
                    ))
                    .with_children(|details| {
                        details.spawn((
                            Node {
                                margin: UiRect::vertical(percent(4.5)),
                                ..default()
                            },
                            add_text(
                                format_deity_alignment(shown_deity, lang, localization),
                                "bold",
                                SUBTITLE_TEXT_SIZE,
                                assets,
                            ),
                            TextColor(BUTTON_TEXT_COLOR),
                            DeityCardAlignment(moral),
                        ));

                        details.spawn((
                            Node {
                                width: percent(85.),
                                margin: UiRect::horizontal(percent(7.5)),
                                ..default()
                            },
                            add_text(
                                format_deity_description(shown_deity, lang, localization),
                                "medium",
                                1.8,
                                assets,
                            ),
                            TextColor(Color::WHITE),
                            DeityCardDescription(moral),
                        ));

                        details
                            .spawn(Node {
                                width: percent(82.),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.),
                                margin: UiRect::top(Val::Auto),
                                padding: UiRect::bottom(percent(4.)),
                                ..default()
                            })
                            .with_children(|choices| {
                                for ethical in [
                                    EthicalAlignment::Lawful,
                                    EthicalAlignment::Neutral,
                                    EthicalAlignment::Chaotic,
                                ] {
                                    spawn_deity_choice(
                                        choices,
                                        assets,
                                        localization,
                                        lang,
                                        moral,
                                        ethical,
                                    );
                                }
                            });
                    });
            });

            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(110.),
                    height: percent(110.),
                    left: percent(-5.),
                    top: percent(-5.),
                    ..default()
                },
                ImageNode::new(assets.image("border")).with_mode(NodeImageMode::Stretch),
                Pickable {
                    should_block_lower: false,
                    is_hoverable: true,
                },
            ))
            .observe(reimage::<Over>(assets.image("border_hover")))
            .observe(reimage::<Out>(assets.image("border")));
        });
}

/// Spawns one ethical-alignment preview and selection button within a deity card.
fn spawn_deity_choice(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    moral: MoralAlignment,
    ethical: EthicalAlignment,
) {
    let deity = Deity::from_alignment(moral, ethical);
    debug_assert_eq!(deity.ethical_alignment(), ethical);
    let label_key = deity_choice_label_key(moral, ethical);

    parent
        .spawn((
            Node {
                width: percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                border: UiRect::all(Val::Px(1.)),
                border_radius: BorderRadius::all(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Button,
            Interaction::default(),
            Pickable::default(),
            DeityChoiceButton(deity),
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(on_deity_choice_over)
        .observe(on_deity_choice_out)
        .observe(on_deity_choice_click)
        .with_children(|choice| {
            choice.spawn((
                add_text(localization.get(label_key, lang), "bold", 1.8, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(label_key.to_string()),
            ));
        });
}

/// Returns the ethical-choice label, using True only for the grid's center.
fn deity_choice_label_key(moral: MoralAlignment, ethical: EthicalAlignment) -> &'static str {
    if moral == MoralAlignment::Neutral && ethical == EthicalAlignment::Neutral {
        return "alignment.true";
    }

    match ethical {
        EthicalAlignment::Lawful => "alignment.lawful",
        EthicalAlignment::Neutral => "alignment.neutral",
        EthicalAlignment::Chaotic => "alignment.chaotic",
    }
}

/// Previews a deity when its alignment button is hovered.
fn on_deity_choice_over(
    event: On<Pointer<Over>>,
    choice_q: Query<&DeityChoiceButton>,
    assets: Res<WorldAssets>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut image_q: Query<(&DeityCardImage, &mut ImageNode)>,
    mut name_q: Query<(&DeityCardName, &mut Text)>,
    mut alignment_q: Query<
        (&DeityCardAlignment, &mut Text),
        (Without<DeityCardName>, Without<DeityCardDescription>),
    >,
    mut desc_q: Query<
        (&DeityCardDescription, &mut Text),
        (Without<DeityCardName>, Without<DeityCardAlignment>),
    >,
) {
    let Ok(choice) = choice_q.get(event.entity) else {
        return;
    };
    refresh_deity_card(
        choice.0,
        &assets,
        &settings,
        &localization,
        &mut image_q,
        &mut name_q,
        &mut alignment_q,
        &mut desc_q,
    );
}

/// Restores a card's neutral preview when its alignment button is no longer hovered.
fn on_deity_choice_out(
    event: On<Pointer<Out>>,
    choice_q: Query<&DeityChoiceButton>,
    assets: Res<WorldAssets>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut image_q: Query<(&DeityCardImage, &mut ImageNode)>,
    mut name_q: Query<(&DeityCardName, &mut Text)>,
    mut alignment_q: Query<
        (&DeityCardAlignment, &mut Text),
        (Without<DeityCardName>, Without<DeityCardDescription>),
    >,
    mut desc_q: Query<
        (&DeityCardDescription, &mut Text),
        (Without<DeityCardName>, Without<DeityCardAlignment>),
    >,
) {
    let Ok(choice) = choice_q.get(event.entity) else {
        return;
    };
    let neutral_deity =
        Deity::from_alignment(choice.0.moral_alignment(), EthicalAlignment::Neutral);
    refresh_deity_card(
        neutral_deity,
        &assets,
        &settings,
        &localization,
        &mut image_q,
        &mut name_q,
        &mut alignment_q,
        &mut desc_q,
    );
}

/// Refreshes one deity card with the supplied deity's image and localized text.
fn refresh_deity_card(
    deity: Deity,
    assets: &WorldAssets,
    settings: &Settings,
    localization: &Localization,
    image_q: &mut Query<(&DeityCardImage, &mut ImageNode)>,
    name_q: &mut Query<(&DeityCardName, &mut Text)>,
    alignment_q: &mut Query<
        (&DeityCardAlignment, &mut Text),
        (Without<DeityCardName>, Without<DeityCardDescription>),
    >,
    desc_q: &mut Query<
        (&DeityCardDescription, &mut Text),
        (Without<DeityCardName>, Without<DeityCardAlignment>),
    >,
) {
    let moral = deity.moral_alignment();
    for (marker, mut image) in image_q.iter_mut() {
        if marker.0 == moral {
            image.image = assets.image(deity.image_key());
        }
    }
    for (marker, mut text) in name_q.iter_mut() {
        if marker.0 == moral {
            text.0 = localization.get(format!("deity.{}", deity.to_lowername()), settings.language);
        }
    }
    for (marker, mut text) in alignment_q.iter_mut() {
        if marker.0 == moral {
            text.0 = format_deity_alignment(deity, settings.language, localization);
        }
    }
    for (marker, mut text) in desc_q.iter_mut() {
        if marker.0 == moral {
            text.0 = format_deity_description(deity, settings.language, localization);
        }
    }
}

/// Commits the clicked deity and completes character creation.
fn on_deity_choice_click(
    event: On<Pointer<Click>>,
    time: Res<Time>,
    gesture: Res<SelectionGestureState>,
    choice_q: Query<&DeityChoiceButton>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if gesture.suppresses_click(time.elapsed_secs_f64()) {
        return;
    }
    let Ok(choice) = choice_q.get(event.entity) else {
        return;
    };

    player.deity = choice.0;
    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies browser and hardware-keyboard names share filtering and character limits.
    #[test]
    fn character_name_input_is_filtered_and_character_limited() {
        assert_eq!(sanitize_character_name("Éowyn! 42"), "Éowyn 42");
        assert_eq!(sanitize_character_name("abcdefghijklmnopq"), "abcdefghijklmnop");

        let mut name = "Hero".to_string();
        append_character_name_text(&mut name, " of Arcana! 123456789");
        assert_eq!(name, "Hero of Arcana 1");
        assert_eq!(name.chars().count(), MAX_CHARACTER_NAME_CHARS);
    }

    /// Verifies a release click is ignored briefly after dragging a selection card.
    #[test]
    fn selection_drag_suppresses_only_the_release_click_window() {
        let mut gesture = SelectionGestureState::default();
        gesture.suppress_after_drag(10.0);

        assert!(gesture.suppresses_click(10.1));
        assert!(!gesture.suppresses_click(10.3));
    }

    /// Verifies every warrior calling selects a race- and sex-specific portrait.
    #[test]
    fn warrior_calling_portraits_include_calling_race_and_sex() {
        let mut player = Player {
            race: Race::Elf,
            sex: Sex::Woman,
            ..default()
        };

        assert_eq!(WarriorPath::Paladin.get_image_key(&player), "warrior_paladin_elf_woman");
        assert_eq!(WarriorPath::Templar.get_image_key(&player), "warrior_templar_elf_woman");
        assert_eq!(WarriorPath::Berserker.get_image_key(&player), "warrior_berserker_elf_woman");
        assert_eq!(WarriorPath::Warden.get_image_key(&player), "warrior_warden_elf_woman");

        player.race = Race::Dragonborn;
        player.sex = Sex::Man;
        assert_eq!(WarriorPath::Templar.get_image_key(&player), "warrior_templar_dragonborn_man");
        assert_eq!(
            WarriorPath::Berserker.get_image_key(&player),
            "warrior_berserker_dragonborn_man"
        );
        assert_eq!(WarriorPath::Warden.get_image_key(&player), "warrior_warden_dragonborn_man");
    }

    /// Verifies only the center of the alignment grid uses the True label.
    #[test]
    fn only_center_deity_choice_is_true() {
        assert_eq!(
            deity_choice_label_key(MoralAlignment::Neutral, EthicalAlignment::Neutral),
            "alignment.true"
        );
        assert_eq!(
            deity_choice_label_key(MoralAlignment::Good, EthicalAlignment::Neutral),
            "alignment.neutral"
        );
        assert_eq!(
            deity_choice_label_key(MoralAlignment::Evil, EthicalAlignment::Neutral),
            "alignment.neutral"
        );
    }
}
