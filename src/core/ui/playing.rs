use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::Class;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::player::{Attribute, Player};
use crate::core::settings::{Language, Settings};
use crate::core::ui::creation::SelectionItem;
use crate::core::utils::cursor;
use crate::core::weapons::Weapon;
use crate::utils::NameFromEnum;
use bevy::window::SystemCursorIcon;

const PANEL_COLOR: Color = Color::srgba_u8(10, 18, 45, 230);
const BAR_BG_COLOR: Color = Color::srgba_u8(0, 0, 0, 160);
const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const MANA_COLOR: Color = Color::srgb_u8(40, 80, 185);
const PLACEHOLDER_COLOR: Color = Color::srgba_u8(40, 40, 55, 220);

#[derive(Component)]
pub struct PlayingCmp;

#[derive(Component)]
pub struct ActionButton(pub &'static str);

/// Simple text stats that are refreshed every frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayingStat {
    ClassLine,
    CharRace,
    CharClass,
    CharSex,
    CharAge,
    CharHeight,
    CharWeight,
    Health,
    Mana,
    Money,
    Attack,
    Armor,
    Initiative,
    ActionPoints,
}

#[derive(Component)]
pub struct StatLabel(pub PlayingStat);

#[derive(Component)]
pub struct AttrValue(pub Attribute);

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct ManaBarFill;

#[derive(Component)]
pub struct EquipmentList;

#[derive(Component)]
pub struct AbilitiesList;

#[derive(Component)]
pub struct PerksList;

/// The five equipment image-slots overlaid on the character portrait.
#[derive(Component, Clone, Copy)]
pub enum EquipSlot {
    Helmet,
    Weapon,
    Offhand,
    Armor,
    Boots,
}

#[derive(Component)]
pub struct PetImage;

#[derive(Component)]
pub struct TooltipNode;

fn portrait_key(player: &Player) -> String {
    match player.class {
        Class::Mage(ajah) => ajah.get_image_key(player),
        _ => player.class.get_image_key(player),
    }
}

fn class_line(player: &Player, localization: &Localization, lang: Language) -> String {
    format!("{} {}", localization.get("level", lang), player.level)
}

/// Format the bonus characteristics of a weapon, e.g. "+6 attack | +10 crit | 1.2 as".
fn weapon_stat_lines(
    weapon: &Weapon,
    player: &Player,
    localization: &Localization,
    lang: Language,
) -> Vec<String> {
    let stats = weapon.stats();
    let mut parts = Vec::new();
    let mut push = |val: i32, key: &str| {
        if val != 0 {
            let sign = if val > 0 {
                "+"
            } else {
                ""
            };
            parts.push(format!("{}{} {}", sign, val, localization.get(key, lang).to_lowercase()));
        }
    };
    push(stats.attack, "attack");
    push(stats.armor, "armor");
    push(stats.crit, "crit");
    push(stats.initiative, "initiative");
    if stats.attack_speed > 0.0 {
        parts.push(format!("{:.1} as", player.weapon_attack_speed(weapon)));
    }
    if parts.is_empty() {
        vec![]
    } else {
        vec![parts.join(" | ")]
    }
}

fn name_with_level(name: String, level: u8) -> String {
    format!("{} (lv. {})", name, level)
}

/// A bordered placeholder box (used wherever an item/ability image will go later).
fn spawn_placeholder(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, size: Val) {
    parent.spawn((
        Node {
            width: size,
            height: size,
            flex_shrink: 0.,
            border: UiRect::all(Val::Px(2.)),
            ..default()
        },
        BackgroundColor(PLACEHOLDER_COLOR),
        BorderColor::all(BUTTON_BORDER_COLOR),
        ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
    ));
}

/// A list entry: placeholder image on the left, a name and detail lines on the right.
fn spawn_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    name: String,
    name_key: Option<String>,
    lines: Vec<String>,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.),
                padding: UiRect::all(Val::Px(6.)),
                margin: UiRect::bottom(Val::Px(6.)),
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            spawn_placeholder(parent, assets, Val::Px(40.));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    let mut name_cmd = parent
                        .spawn((add_text(name, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));
                    if let Some(key) = name_key {
                        name_cmd.insert(LocalizedText(key));
                    }

                    for line in lines {
                        parent.spawn((
                            add_text(line, "medium", 1.6, assets),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}

/// One of the three combat-stat boxes (attack / armor / initiative).
fn spawn_combat_stat(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    label_key: &str,
    image_key: &str,
    stat: PlayingStat,
) {
    parent
        .spawn((
            Node {
                width: percent(30.),
                aspect_ratio: Some(1.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(4.),
                border: UiRect::all(Val::Px(2.)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(PLACEHOLDER_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            // Semi-transparent icon background
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
                ImageNode {
                    image: assets.image(image_key),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(1., 1., 1., 0.3),
                    ..default()
                },
            ));
            parent.spawn((
                add_text(localization.get(label_key, lang), "medium", 2.2, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(label_key.to_string()),
            ));
            parent.spawn((
                add_text("", "bold", 4.5, assets),
                TextColor(Color::WHITE),
                StatLabel(stat),
            ));
        });
}

pub fn setup_playing_screen(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
    let lang = settings.language;

    let (mut root_node, pickable) = add_root_node(true);
    root_node.justify_content = JustifyContent::FlexStart;
    root_node.padding = UiRect::all(Val::Px(0.));
    root_node.row_gap = Val::Px(4.);

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode::new(assets.image("bg3")).with_mode(NodeImageMode::Stretch),
            PlayingCmp,
        ))
        .with_children(|parent| {
            // Character name, top centered.
            parent
                .spawn(Node {
                    width: percent(100.),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::vertical(Val::Px(8.)),
                    margin: UiRect::bottom(Val::Px(8.)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(&player.name, "bold", TITLE_TEXT_SIZE, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            // Three main columns.
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_grow: 1.,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(2.),
                    padding: UiRect::horizontal(Val::Px(26.)),
                    ..default()
                })
                .with_children(|parent| {
                    // Column 1: Character portrait image
                    spawn_image_column(parent, &assets, &player);

                    // Column 2: Stats (level, bars, characteristics, attributes, combat)
                    spawn_stats_column(parent, &assets, &localization, lang, &player);

                    // Column 3: Scrollable equipment, abilities and perks
                    spawn_right_column(parent, &assets, &localization, lang);
                });

            // Bottom row: Action buttons
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: Val::Px(115.),
                    flex_shrink: 0.,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.),
                    padding: UiRect::vertical(Val::Px(2.)),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_playing_action_button(parent, "hunt", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "shop", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "quest", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "train", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "craft", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "rest", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "inventory", &assets, &localization, lang);
                });
        });
}

/// Column 1: Character portrait image with equipment slot overlays and pet.
fn spawn_image_column(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, player: &Player) {
    parent
        .spawn((
            Node {
                width: percent(28.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(6.)),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|parent| {
            // Portrait (relative container for equipment slot / pet overlays)
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        aspect_ratio: Some(1.),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(3.)),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(portrait_key(player)))
                        .with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    // Equipment slots stacked on the top-right, scaling with image
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            right: Val::Percent(2.),
                            top: Val::Percent(2.),
                            width: percent(16.),
                            height: percent(96.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            row_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in [
                                EquipSlot::Helmet,
                                EquipSlot::Weapon,
                                EquipSlot::Offhand,
                                EquipSlot::Armor,
                                EquipSlot::Boots,
                            ] {
                                parent.spawn((
                                    Node {
                                        width: percent(100.),
                                        aspect_ratio: Some(1.),
                                        border: UiRect::all(Val::Px(1.)),
                                        ..default()
                                    },
                                    BackgroundColor(PLACEHOLDER_COLOR),
                                    BorderColor::all(BUTTON_BORDER_COLOR),
                                    ImageNode::new(assets.image("stone"))
                                        .with_mode(NodeImageMode::Stretch),
                                    Interaction::default(),
                                    slot,
                                ));
                            }
                        });

                    // Pet image, bottom-left overlay — larger
                    if player.pet.is_some() {
                        parent.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(3.),
                                bottom: Val::Px(3.),
                                width: percent(50.),
                                aspect_ratio: Some(1.),
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            BorderColor::all(BUTTON_BORDER_COLOR),
                            ImageNode::new(assets.image(player.pet.unwrap().to_lowername()))
                                .with_mode(NodeImageMode::Stretch),
                            PetImage,
                        ));
                    }
                });
        });
}

/// Column 2: Level, health/mana bars, characteristics, attributes, combat stats.
fn spawn_stats_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) {
    parent
        .spawn((
            Node {
                width: percent(30.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(8.)),
                row_gap: Val::Px(4.),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|parent| {
            // Level / class text + AP on the right
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(class_line(player, localization, lang), "bold", 2.6, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        StatLabel(PlayingStat::ClassLine),
                    ));
                    parent.spawn((
                        add_text(format!("AP: {}", player.ap), "bold", 2.2, assets),
                        TextColor(Color::WHITE),
                        StatLabel(PlayingStat::ActionPoints),
                    ));
                });

            // Health bar
            spawn_bar(parent, assets, true);
            // Mana bar (same height as health)
            spawn_bar(parent, assets, false);

            // Spacer between bars and characteristics
            parent.spawn(Node {
                height: Val::Px(10.),
                ..default()
            });

            // Characteristics and Attributes side by side
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(36.),
                    ..default()
                })
                .with_children(|parent| {
                    // Left: Characteristics
                    parent
                        .spawn(Node {
                            width: percent(45.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    localization.get("characteristics", lang),
                                    "bold",
                                    2.2,
                                    assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("characteristics".to_string()),
                            ));
                            parent
                                .spawn(Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    let char_rows = [
                                        ("race", PlayingStat::CharRace),
                                        ("class", PlayingStat::CharClass),
                                        ("sex", PlayingStat::CharSex),
                                        ("age", PlayingStat::CharAge),
                                        ("height", PlayingStat::CharHeight),
                                        ("weight", PlayingStat::CharWeight),
                                    ];
                                    for (key, stat) in char_rows {
                                        parent
                                            .spawn(Node {
                                                width: percent(100.),
                                                flex_direction: FlexDirection::Row,
                                                justify_content: JustifyContent::SpaceBetween,
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    add_text(
                                                        localization.get(key, lang),
                                                        "medium",
                                                        1.8,
                                                        assets,
                                                    ),
                                                    TextColor(Color::WHITE),
                                                    LocalizedText(key.to_string()),
                                                ));
                                                parent.spawn((
                                                    add_text("", "bold", 1.8, assets),
                                                    TextColor(Color::WHITE),
                                                    StatLabel(stat),
                                                ));
                                            });
                                    }
                                });
                        });

                    // Right: Attributes
                    parent
                        .spawn(Node {
                            width: percent(45.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(localization.get("attributes", lang), "bold", 2.2, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("attributes".to_string()),
                            ));
                            parent
                                .spawn(Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    for attr in Attribute::iter() {
                                        parent
                                            .spawn(Node {
                                                width: percent(100.),
                                                flex_direction: FlexDirection::Row,
                                                justify_content: JustifyContent::SpaceBetween,
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    add_text(
                                                        localization
                                                            .get(&attr.to_lowername(), lang),
                                                        "medium",
                                                        1.8,
                                                        assets,
                                                    ),
                                                    TextColor(Color::WHITE),
                                                    LocalizedText(attr.to_lowername()),
                                                ));
                                                parent.spawn((
                                                    add_text("", "bold", 1.8, assets),
                                                    TextColor(Color::WHITE),
                                                    AttrValue(attr),
                                                ));
                                            });
                                    }
                                });
                        });
                });

            // Spacer between attributes and combat stats
            parent.spawn(Node {
                height: Val::Px(10.),
                ..default()
            });

            // Combat stats: attack / armor / initiative
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        "attack",
                        "attack_icon",
                        PlayingStat::Attack,
                    );
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        "armor",
                        "armor_icon",
                        PlayingStat::Armor,
                    );
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        "initiative",
                        "initiative_icon",
                        PlayingStat::Initiative,
                    );
                });
        });
}

fn spawn_bar(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, is_health: bool) {
    let bar_height = Val::Px(32.);
    let font_size = 1.9;
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: bar_height,
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            // Fill.
            let mut fill = parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
                BackgroundColor(if is_health {
                    HEALTH_COLOR
                } else {
                    MANA_COLOR
                }),
            ));
            if is_health {
                fill.insert(HealthBarFill);
            } else {
                fill.insert(ManaBarFill);
            }

            // Text overlay.
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.),
                    height: percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text("", "bold", font_size, assets),
                        TextColor(Color::WHITE),
                        StatLabel(if is_health {
                            PlayingStat::Health
                        } else {
                            PlayingStat::Mana
                        }),
                    ));
                });
        });
}

#[derive(Component)]
pub struct ScrollableContainer;

pub fn scroll_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut ScrollPosition, &Node), With<ScrollableContainer>>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll, _node) in &mut query {
            // Scroll offset speed factor
            scroll.y -= event.y * 30.0;
            if scroll.y < 0.0 {
                scroll.y = 0.0;
            }
        }
    }
}

/// Shows/hides a tooltip when hovering over equipment slots.
pub fn equip_slot_tooltip_system(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    slot_q: Query<(&Interaction, &EquipSlot), Changed<Interaction>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    let lang = settings.language;

    for (interaction, slot) in &slot_q {
        // Despawn any existing tooltip on any change
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }

        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let equipped = match slot {
                EquipSlot::Helmet => player.helmet,
                EquipSlot::Weapon => player.weapon_2h.or(player.weapon_lh),
                EquipSlot::Offhand => player.weapon_rh,
                EquipSlot::Armor => player.armor,
                EquipSlot::Boots => player.boots,
            };

            if let Some(weapon) = equipped {
                let name =
                    name_with_level(localization.get(&weapon.to_lowername(), lang), weapon.level());
                let stat_lines = weapon_stat_lines(&weapon, &player, &localization, lang);

                let (left, top) = if let Ok(window) = windows.single() {
                    if let Some(cursor) = window.cursor_position() {
                        (cursor.x, cursor.y)
                    } else {
                        (100., 100.)
                    }
                } else {
                    (100., 100.)
                };

                commands
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(left),
                            top: Val::Px(top),
                            padding: UiRect::all(Val::Px(10.)),
                            border: UiRect::all(Val::Px(2.)),
                            width: Val::Auto,
                            height: Val::Auto,
                            max_width: Val::Px(280.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(10, 18, 45, 245)),
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        GlobalZIndex(200),
                        TooltipNode,
                    ))
                    .with_children(|parent| {
                        // Weapon name in gold/title color
                        parent.spawn((
                            add_text(name, "bold", 1.9, &assets),
                            TextColor(BUTTON_TEXT_COLOR),
                        ));
                        // Stats in white with medium font
                        if !stat_lines.is_empty() {
                            parent.spawn((
                                add_text(stat_lines.join("\n"), "medium", 1.6, &assets),
                                TextColor(Color::WHITE),
                            ));
                        }
                    });
            }
        }
    }
}

/// Moves the tooltip to follow the mouse cursor.
pub fn tooltip_follow_cursor_system(
    mut tooltip_q: Query<&mut Node, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        if let Some(cursor) = window.cursor_position() {
            for mut node in &mut tooltip_q {
                node.left = Val::Px(cursor.x);
                node.top = Val::Px(cursor.y);
            }
        }
    }
}

fn spawn_right_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    parent
        .spawn((
            Node {
                width: percent(34.),
                height: percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(8.)),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        height: percent(100.),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    ScrollableContainer,
                    ScrollPosition::default(),
                ))
                .with_children(|parent| {
                    // Title row: "Equipment" on the left, gold icon + amount on the right.
                    parent
                        .spawn(Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            margin: UiRect::bottom(Val::Px(4.)),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(localization.get("equipment", lang), "bold", 2.4, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("equipment".to_string()),
                            ));

                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(30.),
                                            height: Val::Px(30.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image("gold"))
                                            .with_mode(NodeImageMode::Stretch),
                                    ));
                                    parent.spawn((
                                        add_text("", "bold", 2.4, assets),
                                        TextColor(BUTTON_TEXT_COLOR),
                                        StatLabel(PlayingStat::Money),
                                    ));
                                });
                        });

                    // Dynamic equipment list
                    parent.spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::bottom(Val::Px(15.)),
                            ..default()
                        },
                        EquipmentList,
                    ));

                    // Title: Abilities
                    parent.spawn((
                        add_text(localization.get("abilities", lang), "bold", 2.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText("abilities".to_string()),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::bottom(Val::Px(15.)),
                            ..default()
                        },
                        AbilitiesList,
                    ));

                    // Title: Perks
                    parent.spawn((
                        add_text(localization.get("perks", lang), "bold", 2.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText("perks".to_string()),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        PerksList,
                    ));
                });
        });
}

/// Rebuilds the equipment, abilities and perks lists whenever the player or
/// language changes (these have a variable number of entries).
pub fn rebuild_playing_lists(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    player: Res<Player>,
    equip_q: Query<Entity, With<EquipmentList>>,
    abil_q: Query<Entity, With<AbilitiesList>>,
    perk_q: Query<Entity, With<PerksList>>,
    mut slot_q: Query<(&EquipSlot, &mut ImageNode)>,
    children_q: Query<&Children>,
) {
    let lang = settings.language;

    // Update the five equipment image-slots on the portrait.
    for (slot, mut image) in &mut slot_q {
        let equipped = match slot {
            EquipSlot::Helmet => player.helmet,
            EquipSlot::Weapon => player.weapon_2h.or(player.weapon_lh),
            EquipSlot::Offhand => player.weapon_rh,
            EquipSlot::Armor => player.armor,
            EquipSlot::Boots => player.boots,
        };
        image.image = match equipped {
            Some(weapon) => assets.image(weapon.image_key()),
            None => assets.image("stone"),
        };
    }

    let clear = |commands: &mut Commands, entity: Entity, children_q: &Query<&Children>| {
        if let Ok(children) = children_q.get(entity) {
            for child in children.iter() {
                commands.entity(child).try_despawn();
            }
        }
    };

    // Equipment list.
    if let Ok(entity) = equip_q.single() {
        clear(&mut commands, entity, &children_q);
        commands.entity(entity).with_children(|parent| {
            let mut empty = true;
            let mut slots: Vec<&Weapon> = Vec::new();
            for slot in [
                &player.helmet,
                &player.armor,
                &player.boots,
                &player.weapon_lh,
                &player.weapon_rh,
                &player.weapon_2h,
            ] {
                if let Some(weapon) = slot {
                    slots.push(weapon);
                }
            }
            for weapon in slots {
                empty = false;
                spawn_card(
                    parent,
                    &assets,
                    name_with_level(localization.get(&weapon.to_lowername(), lang), weapon.level()),
                    None,
                    weapon_stat_lines(weapon, &player, &localization, lang),
                );
            }
            for consumable in &player.consumables {
                empty = false;
                spawn_card(
                    parent,
                    &assets,
                    localization.get(&consumable.to_lowername(), lang),
                    Some(consumable.to_lowername()),
                    vec![],
                );
            }
            if empty {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
    }

    // Abilities list.
    if let Ok(entity) = abil_q.single() {
        clear(&mut commands, entity, &children_q);
        commands.entity(entity).with_children(|parent| {
            if player.abilities.is_empty() {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            for ability in &player.abilities {
                let key = ability.to_lowername();
                let stats = ability.stats();
                let type_str = format!("{:?}", stats.magic_type);
                let detail_parts =
                    [type_str, format!("{} mana", stats.mana_cost), format!("{}s", stats.cooldown)];
                spawn_card(
                    parent,
                    &assets,
                    name_with_level(localization.get(&key, lang), stats.level),
                    None,
                    vec![
                        detail_parts.join(" | "),
                        localization.get(&format!("{}_desc", key), lang),
                    ],
                );
            }
        });
    }

    // Perks list.
    if let Ok(entity) = perk_q.single() {
        clear(&mut commands, entity, &children_q);
        commands.entity(entity).with_children(|parent| {
            if player.perks.is_empty() {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            for perk in &player.perks {
                let key = perk.to_lowername();
                spawn_card(
                    parent,
                    &assets,
                    name_with_level(localization.get(&key, lang), perk.level()),
                    None,
                    vec![localization.get(&format!("{}_desc", key), lang)],
                );
            }
        });
    }
}

pub fn update_playing_screen(
    player: Res<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut text_q: Query<(&mut Text, &StatLabel)>,
    mut attr_q: Query<(&mut Text, &AttrValue), Without<StatLabel>>,
    mut hbar_q: Query<&mut Node, (With<HealthBarFill>, Without<ManaBarFill>)>,
    mut mbar_q: Query<&mut Node, (With<ManaBarFill>, Without<HealthBarFill>)>,
) {
    let lang = settings.language;

    for (mut text, stat) in &mut text_q {
        text.0 = match stat.0 {
            PlayingStat::ClassLine => class_line(&player, &localization, lang),
            PlayingStat::CharRace => localization.get(&player.race.to_lowername(), lang),
            PlayingStat::CharClass => match player.class {
                Class::Mage(ajah) => {
                    format!("{} {}", ajah.to_title(), localization.get("mage", lang))
                },
                _ => localization.get(&player.class.to_lowername(), lang),
            },
            PlayingStat::CharSex => match player.sex {
                crate::core::player::Sex::Male => localization.get("male", lang),
                crate::core::player::Sex::Female => localization.get("female", lang),
            },
            PlayingStat::CharAge => format!("{}", player.actual_age()),
            PlayingStat::CharHeight => {
                let (height, _) = player.vitals();
                format!("{} cm", height)
            },
            PlayingStat::CharWeight => {
                let (_, weight) = player.vitals();
                format!("{} kg", weight)
            },
            PlayingStat::Health => format!(
                "{} / {} {}",
                player.health.max(0.) as i32,
                player.max_health() as i32,
                localization.get("health", lang)
            ),
            PlayingStat::Mana => format!(
                "{} / {} {}",
                player.mana.max(0.) as i32,
                player.max_mana() as i32,
                localization.get("mana", lang)
            ),
            PlayingStat::Money => format!("{}", player.gold),
            PlayingStat::Attack => format!("{}", player.attack_damage()),
            PlayingStat::Armor => format!("{}", player.armor_value()),
            PlayingStat::Initiative => format!("{}", player.initiative()),
            PlayingStat::ActionPoints => format!("AP: {}", player.ap),
        };
    }

    for (mut text, attr) in &mut attr_q {
        let val = match attr.0 {
            Attribute::Strength => player.strength(),
            Attribute::Dexterity => player.dexterity(),
            Attribute::Constitution => player.constitution(),
            Attribute::Intelligence => player.intelligence(),
            Attribute::Wisdom => player.wisdom(),
            Attribute::Charisma => player.charisma(),
        };
        text.0 = format!("{}", val);
    }

    if let Ok(mut node) = hbar_q.single_mut() {
        let ratio = (player.health / player.max_health()).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Ok(mut node) = mbar_q.single_mut() {
        let ratio = (player.mana / player.max_mana()).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
}

pub fn spawn_playing_action_button(
    parent: &mut ChildSpawnerCommands,
    action: &'static str,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    let action_label = localization.get(action, lang);
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(2.),
            margin: UiRect::horizontal(Val::Px(6.)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(76.),
                        height: Val::Px(76.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.)),
                        border_radius: BorderRadius::all(percent(50.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(format!("action_{}", action)))
                        .with_mode(NodeImageMode::Stretch),
                    Button,
                    ActionButton(action),
                ))
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default));

            parent.spawn((
                add_text(action_label, "bold", 1.8, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(action.to_string()),
            ));
        });
}

pub fn handle_playing_action_clicks(
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    interaction_q: Query<(&Interaction, &ActionButton), (Changed<Interaction>, With<Button>)>,
) {
    use rand::RngExt;

    for (interaction, action) in &interaction_q {
        if *interaction == Interaction::Pressed {
            play_audio_msg.write(PlayAudioMsg::new("button"));

            let cost_gold = match action.0 {
                "craft" => 15,
                "shop" => 30,
                "rest" => 10,
                _ => 0,
            };

            if player.gold < cost_gold {
                play_audio_msg.write(PlayAudioMsg::new("error"));
                continue;
            }

            player.gold -= cost_gold;

            // Handle the specific action
            let ap_cost = match action.0 {
                "hunt" => {
                    let gold_earned = rand::rng().random_range(10..=20);
                    player.gold += gold_earned;
                    2
                },
                "shop" => {
                    use crate::core::consumables::Consumable;
                    let items = vec![
                        Consumable::HealingPotion,
                        Consumable::ManaPotion,
                        Consumable::PoisonVial,
                        Consumable::HerbBlend,
                    ];
                    use rand::seq::IndexedRandom;
                    let item = items
                        .choose(&mut rand::rng())
                        .copied()
                        .unwrap_or(Consumable::HealingPotion);
                    player.consumables.push(item);
                    1
                },
                "quest" => {
                    let gold_earned = rand::rng().random_range(20..=40);
                    if rand::rng().random_bool(0.5) {
                        let upgrade_types = vec![
                            Weapon::IronHelmet,
                            Weapon::IronChestplate,
                            Weapon::IronBoots,
                            Weapon::SteelSword,
                            Weapon::IronShield,
                            Weapon::WizardStaff,
                            Weapon::MageRobes,
                            Weapon::ClothShoes,
                            Weapon::LeatherArmor,
                            Weapon::SilentBoots,
                            Weapon::AssassinDagger,
                            Weapon::ThiefDagger,
                            Weapon::OakWand,
                            Weapon::LeafyGarb,
                        ];
                        use rand::seq::IndexedRandom;
                        if let Some(&upg) = upgrade_types.choose(&mut rand::rng()) {
                            match upg {
                                Weapon::IronHelmet => player.helmet = Some(upg),
                                Weapon::IronChestplate
                                | Weapon::MageRobes
                                | Weapon::LeatherArmor
                                | Weapon::LeafyGarb => player.armor = Some(upg),
                                Weapon::IronBoots
                                | Weapon::ClothShoes
                                | Weapon::SilentBoots
                                | Weapon::LeatherBoots => player.boots = Some(upg),
                                Weapon::WizardStaff => {
                                    player.weapon_2h = Some(upg);
                                    player.weapon_lh = None;
                                    player.weapon_rh = None;
                                },
                                Weapon::SteelSword
                                | Weapon::AssassinDagger
                                | Weapon::ThiefDagger
                                | Weapon::OakWand => {
                                    player.weapon_lh = Some(upg);
                                },
                                Weapon::IronShield => {
                                    player.weapon_rh = Some(upg);
                                },
                            }
                        }
                    }
                    player.gold += gold_earned;
                    3
                },
                "train" => {
                    let attr_idx = rand::rng().random_range(0..6);
                    match attr_idx {
                        0 => player.strength += 1,
                        1 => player.dexterity += 1,
                        2 => player.constitution += 1,
                        3 => player.intelligence += 1,
                        4 => player.wisdom += 1,
                        _ => player.charisma += 1,
                    };
                    2
                },
                "craft" => {
                    let items = vec![
                        Weapon::IronHelmet,
                        Weapon::IronChestplate,
                        Weapon::IronBoots,
                        Weapon::SteelSword,
                        Weapon::IronShield,
                    ];
                    use rand::seq::IndexedRandom;
                    let item =
                        items.choose(&mut rand::rng()).copied().unwrap_or(Weapon::IronHelmet);
                    match item {
                        Weapon::IronHelmet => player.helmet = Some(item),
                        Weapon::IronChestplate => player.armor = Some(item),
                        Weapon::IronBoots => player.boots = Some(item),
                        Weapon::SteelSword => player.weapon_lh = Some(item),
                        Weapon::IronShield => player.weapon_rh = Some(item),
                        _ => {},
                    }
                    2
                },
                "rest" => {
                    player.health = player.max_health().floor();
                    player.mana = player.max_mana().floor();
                    1
                },
                "inventory" => 0,
                _ => 0,
            };

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
            } else {
                player.ap -= ap_cost;
            }
        }
    }
}
