use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use serde::{Deserialize, Serialize};

use crate::core::actions::shop::WeaponTypeFilter;
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::equipment::Kind;
use crate::core::catalog::weapons::{Category, Hand};
use crate::core::constants::*;
use crate::core::game_state::ShopUiState;
use crate::core::menu::utils::{add_text, recolor};
use crate::core::utils::cursor;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenDropdown {
    #[default]
    None,
    Hand,
    Type,
    Category,
    Kind,
}

#[derive(Component, Clone, Copy)]
pub struct ShopDropdownButton(pub OpenDropdown);

#[derive(Component, Clone, Copy)]
pub struct ShopDropdownOptionHand(pub Option<Hand>);

#[derive(Component, Clone, Copy)]
pub struct ShopDropdownOptionType(pub WeaponTypeFilter);

#[derive(Component, Clone, Copy)]
pub struct ShopDropdownOptionCategory(pub Option<Category>);

#[derive(Component, Clone, Copy)]
pub struct ShopDropdownOptionKind(pub Option<Kind>);

pub fn handle_shop_dropdown_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&ShopDropdownButton>,
    mut open_dropdown: ResMut<OpenDropdown>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        if *open_dropdown == btn.0 {
            *open_dropdown = OpenDropdown::None;
        } else {
            *open_dropdown = btn.0;
        }
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn handle_shop_dropdown_option_hand(
    event: On<Pointer<Click>>,
    opt_q: Query<&ShopDropdownOptionHand>,
    mut shop_ui_state: ResMut<ShopUiState>,
    mut open_dropdown: ResMut<OpenDropdown>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(opt) = opt_q.get(event.entity) {
        let active_tab = shop_ui_state.active_tab;
        shop_ui_state.state_for_mut(active_tab).weapon_hand = opt.0;
        *open_dropdown = OpenDropdown::None;
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn handle_shop_dropdown_option_type(
    event: On<Pointer<Click>>,
    opt_q: Query<&ShopDropdownOptionType>,
    mut shop_ui_state: ResMut<ShopUiState>,
    mut open_dropdown: ResMut<OpenDropdown>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(opt) = opt_q.get(event.entity) {
        let active_tab = shop_ui_state.active_tab;
        shop_ui_state.state_for_mut(active_tab).weapon_type = opt.0;
        *open_dropdown = OpenDropdown::None;
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn handle_shop_dropdown_option_category(
    event: On<Pointer<Click>>,
    opt_q: Query<&ShopDropdownOptionCategory>,
    mut shop_ui_state: ResMut<ShopUiState>,
    mut open_dropdown: ResMut<OpenDropdown>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(opt) = opt_q.get(event.entity) {
        let active_tab = shop_ui_state.active_tab;
        shop_ui_state.state_for_mut(active_tab).weapon_category = opt.0;
        *open_dropdown = OpenDropdown::None;
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn handle_shop_dropdown_option_kind(
    event: On<Pointer<Click>>,
    opt_q: Query<&ShopDropdownOptionKind>,
    mut shop_ui_state: ResMut<ShopUiState>,
    mut open_dropdown: ResMut<OpenDropdown>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(opt) = opt_q.get(event.entity) {
        let active_tab = shop_ui_state.active_tab;
        shop_ui_state.state_for_mut(active_tab).kind = opt.0;
        *open_dropdown = OpenDropdown::None;
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn spawn_dropdown_hand(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    current_val: Option<Hand>,
    open_dropdown: OpenDropdown,
) {
    let text_str = match current_val {
        None => "Hand: All",
        Some(Hand::OneHand) => "Hand: 1-H",
        Some(Hand::TwoHand) => "Hand: 2-H",
    };

    let is_open = open_dropdown == OpenDropdown::Hand;

    parent
        .spawn(Node {
            width: percent(100.),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        padding: UiRect::axes(Val::Px(4.), Val::Px(6.)),
                        border: UiRect::all(Val::Px(1.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(if is_open {
                        HOVERED_BUTTON_COLOR
                    } else {
                        NORMAL_BUTTON_COLOR
                    }),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    ShopDropdownButton(OpenDropdown::Hand),
                ))
                .observe(handle_shop_dropdown_click)
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(text_str.to_string(), "bold", 1.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            if is_open {
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Relative,
                            width: percent(100.),
                            margin: UiRect::top(Val::Px(4.)),
                            flex_direction: FlexDirection::Column,
                            border: UiRect::all(Val::Px(1.)),
                            padding: UiRect::vertical(Val::Px(2.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(15, 15, 25, 255)),
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        ZIndex(100),
                    ))
                    .with_children(|parent| {
                        for (opt, label) in [
                            (None, "All"),
                            (Some(Hand::OneHand), "1-Handed"),
                            (Some(Hand::TwoHand), "2-Handed"),
                        ] {
                            parent
                                .spawn((
                                    Node {
                                        width: percent(100.),
                                        padding: UiRect::axes(Val::Px(6.), Val::Px(4.)),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(NORMAL_BUTTON_COLOR),
                                    Button,
                                    Interaction::default(),
                                    Pickable::default(),
                                    ShopDropdownOptionHand(opt),
                                ))
                                .observe(handle_shop_dropdown_option_hand)
                                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(label.to_string(), "bold", 1.3, assets),
                                        TextColor(BUTTON_TEXT_COLOR),
                                    ));
                                });
                        }
                    });
            }
        });
}

pub fn spawn_dropdown_type(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    current_val: WeaponTypeFilter,
    open_dropdown: OpenDropdown,
) {
    let text_str = match current_val {
        WeaponTypeFilter::All => "Type: All",
        WeaponTypeFilter::Weapons => "Type: Wep",
        WeaponTypeFilter::Shields => "Type: Shld",
        WeaponTypeFilter::Books => "Type: Book",
    };

    let is_open = open_dropdown == OpenDropdown::Type;

    parent
        .spawn(Node {
            width: percent(100.),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        padding: UiRect::axes(Val::Px(4.), Val::Px(6.)),
                        border: UiRect::all(Val::Px(1.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(if is_open {
                        HOVERED_BUTTON_COLOR
                    } else {
                        NORMAL_BUTTON_COLOR
                    }),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    ShopDropdownButton(OpenDropdown::Type),
                ))
                .observe(handle_shop_dropdown_click)
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(text_str.to_string(), "bold", 1.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            if is_open {
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Relative,
                            width: percent(100.),
                            margin: UiRect::top(Val::Px(4.)),
                            flex_direction: FlexDirection::Column,
                            border: UiRect::all(Val::Px(1.)),
                            padding: UiRect::vertical(Val::Px(2.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(15, 15, 25, 255)),
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        ZIndex(100),
                    ))
                    .with_children(|parent| {
                        for (opt, label) in [
                            (WeaponTypeFilter::All, "All"),
                            (WeaponTypeFilter::Weapons, "Weapons"),
                            (WeaponTypeFilter::Shields, "Shields"),
                            (WeaponTypeFilter::Books, "Books"),
                        ] {
                            parent
                                .spawn((
                                    Node {
                                        width: percent(100.),
                                        padding: UiRect::axes(Val::Px(6.), Val::Px(4.)),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(NORMAL_BUTTON_COLOR),
                                    Button,
                                    Interaction::default(),
                                    Pickable::default(),
                                    ShopDropdownOptionType(opt),
                                ))
                                .observe(handle_shop_dropdown_option_type)
                                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(label.to_string(), "bold", 1.3, assets),
                                        TextColor(BUTTON_TEXT_COLOR),
                                    ));
                                });
                        }
                    });
            }
        });
}

pub fn spawn_dropdown_category(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    current_val: Option<Category>,
    open_dropdown: OpenDropdown,
) {
    let text_str = match current_val {
        None => "Cat: All",
        Some(Category::Finesse) => "Cat: Fin",
        Some(Category::Magical) => "Cat: Mag",
        Some(Category::Melee) => "Cat: Mel",
        Some(Category::Range) => "Cat: Rng",
        Some(Category::Shield) => "Cat: Shld",
        Some(Category::Book) => "Cat: Book",
    };

    let is_open = open_dropdown == OpenDropdown::Category;

    parent
        .spawn(Node {
            width: percent(100.),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        padding: UiRect::axes(Val::Px(4.), Val::Px(6.)),
                        border: UiRect::all(Val::Px(1.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(if is_open {
                        HOVERED_BUTTON_COLOR
                    } else {
                        NORMAL_BUTTON_COLOR
                    }),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    ShopDropdownButton(OpenDropdown::Category),
                ))
                .observe(handle_shop_dropdown_click)
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(text_str.to_string(), "bold", 1.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            if is_open {
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Relative,
                            width: percent(100.),
                            margin: UiRect::top(Val::Px(4.)),
                            flex_direction: FlexDirection::Column,
                            border: UiRect::all(Val::Px(1.)),
                            padding: UiRect::vertical(Val::Px(2.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(15, 15, 25, 255)),
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        ZIndex(100),
                    ))
                    .with_children(|parent| {
                        for (opt, label) in [
                            (None, "All"),
                            (Some(Category::Finesse), "Finesse"),
                            (Some(Category::Magical), "Magical"),
                            (Some(Category::Melee), "Melee"),
                            (Some(Category::Range), "Ranged"),
                            (Some(Category::Shield), "Shield"),
                            (Some(Category::Book), "Book"),
                        ] {
                            parent
                                .spawn((
                                    Node {
                                        width: percent(100.),
                                        padding: UiRect::axes(Val::Px(6.), Val::Px(4.)),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(NORMAL_BUTTON_COLOR),
                                    Button,
                                    Interaction::default(),
                                    Pickable::default(),
                                    ShopDropdownOptionCategory(opt),
                                ))
                                .observe(handle_shop_dropdown_option_category)
                                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(label.to_string(), "bold", 1.3, assets),
                                        TextColor(BUTTON_TEXT_COLOR),
                                    ));
                                });
                        }
                    });
            }
        });
}

pub fn spawn_dropdown_kind(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    current_val: Option<Kind>,
    open_dropdown: OpenDropdown,
) {
    let text_str = match current_val {
        None => "Kind: All",
        Some(Kind::Physical) => "Kind: Phys",
        Some(Kind::Fire) => "Kind: Fire",
        Some(Kind::Ice) => "Kind: Ice",
        Some(Kind::Nature) => "Kind: Nat",
        Some(Kind::Holy) => "Kind: Holy",
        Some(Kind::Shadow) => "Kind: Shad",
    };

    let is_open = open_dropdown == OpenDropdown::Kind;

    parent
        .spawn(Node {
            width: percent(100.),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        padding: UiRect::axes(Val::Px(4.), Val::Px(6.)),
                        border: UiRect::all(Val::Px(1.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(if is_open {
                        HOVERED_BUTTON_COLOR
                    } else {
                        NORMAL_BUTTON_COLOR
                    }),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    ShopDropdownButton(OpenDropdown::Kind),
                ))
                .observe(handle_shop_dropdown_click)
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(text_str.to_string(), "bold", 1.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            if is_open {
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Relative,
                            width: percent(100.),
                            margin: UiRect::top(Val::Px(4.)),
                            flex_direction: FlexDirection::Column,
                            border: UiRect::all(Val::Px(1.)),
                            padding: UiRect::vertical(Val::Px(2.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(15, 15, 25, 255)),
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        ZIndex(100),
                    ))
                    .with_children(|parent| {
                        for (opt, label) in [
                            (None, "All"),
                            (Some(Kind::Physical), "Physical"),
                            (Some(Kind::Fire), "Fire"),
                            (Some(Kind::Ice), "Ice"),
                            (Some(Kind::Nature), "Nature"),
                            (Some(Kind::Holy), "Holy"),
                            (Some(Kind::Shadow), "Shadow"),
                        ] {
                            parent
                                .spawn((
                                    Node {
                                        width: percent(100.),
                                        padding: UiRect::axes(Val::Px(6.), Val::Px(4.)),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(NORMAL_BUTTON_COLOR),
                                    Button,
                                    Interaction::default(),
                                    Pickable::default(),
                                    ShopDropdownOptionKind(opt),
                                ))
                                .observe(handle_shop_dropdown_option_kind)
                                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(label.to_string(), "bold", 1.3, assets),
                                        TextColor(BUTTON_TEXT_COLOR),
                                    ));
                                });
                        }
                    });
            }
        });
}

pub fn shop_close_dropdown_on_outside_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut open_dropdown: ResMut<OpenDropdown>,
    dropdown_elements_q: Query<
        &Interaction,
        Or<(
            With<ShopDropdownButton>,
            With<ShopDropdownOptionHand>,
            With<ShopDropdownOptionType>,
            With<ShopDropdownOptionCategory>,
            With<ShopDropdownOptionKind>,
        )>,
    >,
) {
    if *open_dropdown == OpenDropdown::None {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let clicked_inside = dropdown_elements_q.iter().any(|interaction| {
            *interaction == Interaction::Hovered || *interaction == Interaction::Pressed
        });
        if !clicked_inside {
            *open_dropdown = OpenDropdown::None;
        }
    }
}
