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

#[derive(Component)]
pub struct RightScrollbarTrack;

#[derive(Component)]
pub struct RightScrollbarThumb;

/// The six equipment image-slots overlaid on the character portrait.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Helmet,
    Accessory,
    WeaponLH,
    WeaponRH,
    Armor,
    Boots,
}

#[derive(Component)]
pub struct PetImage;

#[derive(Component)]
pub struct TooltipNode {
    width: f32,
    height: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InfoTooltip {
    Gold,
    ActionPoints,
    Combat(PlayingStat),
    Action(&'static str),
}

fn portrait_key(player: &Player) -> String {
    match player.class {
        Class::Mage(ajah) => ajah.get_image_key(player),
        _ => player.class.get_image_key(player),
    }
}

fn class_line(player: &Player, localization: &Localization, lang: Language) -> String {
    format!("{} {}", localization.get("level", lang), player.level)
}

fn playing_title(player: &Player) -> String {
    if player.pet.is_some() && !player.pet_name.trim().is_empty() {
        format!("{} & {}", player.name, player.pet_name)
    } else {
        player.name.clone()
    }
}

fn localized_class_name(player: &Player, localization: &Localization, lang: Language) -> String {
    match player.class {
        Class::Mage(ajah) => format!(
            "{} {}",
            localization.get(&ajah.to_lowername(), lang),
            localization.get("mage", lang)
        ),
        _ => localization.get(&player.class.to_lowername(), lang),
    }
}

pub fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn name_with_level(name: String, level: u8) -> String {
    format!("{} (Lv. {})", capitalize_words(&name), level)
}

fn ability_detail_line(
    stats: &crate::core::catalog::GeneratedAbility,
    localization: &Localization,
    lang: Language,
) -> String {
    let mut parts = vec![format!(
        "{}: {} | {}: {}",
        localization.get("ability_type", lang),
        localization.get(&stats.magic_type.to_lowercase(), lang),
        localization.get("mana", lang),
        stats.mana_cost
    )];

    if stats.cooldown > 0 {
        parts.push(format!(
            "{}: {}s",
            localization.get("cooldown", lang),
            stats.cooldown
        ));
    }

    parts.join(" | ")
}

/// Format the bonus characteristics of a weapon, e.g. "+6 attack | +10 crit | 1.2 as".
fn weapon_stat_lines(
    weapon: &crate::core::catalog::GeneratedEquipment,
    player: &Player,
    localization: &Localization,
    lang: Language,
) -> Vec<String> {
    let mut parts = Vec::new();
    let mut push = |val: i32, key: &str| {
        if val != 0 {
            let sign = if val > 0 {
                "+"
            } else {
                ""
            };
            parts.push(format!("{}{} {}", sign, val, localization.get(key, lang)));
        }
    };
    push(weapon.attack, "attack");
    push(weapon.armor, "armor");
    push(weapon.crit, "crit");
    push(weapon.initiative, "initiative");
    if weapon.attack_speed > 0.0 {
        parts.push(format!("{:.1} as", player.weapon_attack_speed(&weapon.name)));
    }
    if parts.is_empty() {
        vec![]
    } else {
        vec![parts.join(" | ")]
    }
}

fn signed_line(label: impl Into<String>, value: i32) -> String {
    let label = label.into();
    if value > 0 {
        format!("{}: +{}", label, value)
    } else {
        format!("{}: {}", label, value)
    }
}

fn weapon_bonus_lines(
    player: &Player,
    value_for: impl Fn(&crate::core::catalog::GeneratedEquipment) -> i32,
) -> Vec<String> {
    player
        .equipped_equipment()
        .into_iter()
        .filter_map(|weapon| {
            let value = value_for(&weapon);
            (value != 0).then(|| signed_line(weapon.name.to_string(), value))
        })
        .collect()
}

fn combat_breakdown(
    stat: PlayingStat,
    player: &Player,
    localization: &Localization,
    lang: Language,
) -> Vec<String> {
    match stat {
        PlayingStat::Attack => {
            let mut lines = vec![signed_line(localization.get("base", lang), 5)];
            lines.push(signed_line(
                localization.get("strength", lang),
                player.strength() as i32 - 10,
            ));
            lines.extend(weapon_bonus_lines(player, |weapon| {
                weapon.attack
            }));
            lines
        },
        PlayingStat::Armor => {
            let mut lines = vec![signed_line(
                localization.get("constitution", lang),
                player.constitution() as i32 / 4,
            )];
            lines.extend(weapon_bonus_lines(player, |weapon| {
                weapon.armor
            }));
            lines
        },
        PlayingStat::Initiative => {
            let mut lines = vec![signed_line(
                localization.get("dexterity", lang),
                player.dexterity() as i32 / 2,
            )];
            lines.extend(weapon_bonus_lines(player, |weapon| {
                weapon.initiative
            }));
            if matches!(player.class, Class::Rogue) {
                lines.push(signed_line(localization.get("rogue", lang), 2));
            }
            lines
        },
        _ => vec![],
    }
}

fn spawn_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    title: String,
    lines: Vec<String>,
    windows: &Query<&Window>,
) {
    let wrapped_lines: Vec<String> =
        lines.into_iter().flat_map(|line| wrap_tooltip_line(&line, 60)).collect();
    let max_chars = std::iter::once(title.as_str())
        .chain(wrapped_lines.iter().map(String::as_str))
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(12) as f32;
    let line_count = 1 + wrapped_lines.len().max(1);

    let (window_width, window_height, cursor) = if let Ok(window) = windows.single() {
        (window.width(), window.height(), window.cursor_position())
    } else {
        (1600., 900., None)
    };
    let tooltip_width = (max_chars * 9.5 + 32.).clamp(190., (window_width - 24.).max(190.));
    let tooltip_height = (line_count as f32 * 24. + 24.).clamp(64., (window_height - 24.).max(64.));
    let (left, top) = place_tooltip(
        cursor.unwrap_or(Vec2::new(100., 100.)),
        tooltip_width,
        tooltip_height,
        window_width,
        window_height,
    );

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::all(Val::Px(10.)),
                border: UiRect::all(Val::Px(2.)),
                width: Val::Px(tooltip_width),
                height: Val::Auto,
                max_width: Val::Px(tooltip_width),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(10, 18, 45, 245)),
            BorderColor::all(BUTTON_BORDER_COLOR),
            GlobalZIndex(200),
            TooltipNode {
                width: tooltip_width,
                height: tooltip_height,
            },
        ))
        .with_children(|parent| {
            parent.spawn((add_text(title, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));
            if !wrapped_lines.is_empty() {
                parent.spawn((
                    add_text(wrapped_lines.join("\n"), "medium", 1.6, assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
}

fn wrap_tooltip_line(line: &str, max_chars: usize) -> Vec<String> {
    if line.chars().count() <= max_chars {
        return vec![line.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        let next_len = current.chars().count()
            + if current.is_empty() {
                0
            } else {
                1
            }
            + word.chars().count();
        if next_len > max_chars && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn place_tooltip(
    cursor: Vec2,
    width: f32,
    height: f32,
    window_width: f32,
    window_height: f32,
) -> (f32, f32) {
    let margin = 12.;
    let mut left = cursor.x + margin;
    if left + width + margin > window_width {
        left = cursor.x - width - margin;
    }
    let mut top = cursor.y + margin;
    if top + height + margin > window_height {
        top = cursor.y - height - margin;
    }
    (
        left.clamp(margin, (window_width - width - margin).max(margin)),
        top.clamp(margin, (window_height - height - margin).max(margin)),
    )
}

/// A bordered placeholder box (used wherever an item/ability image will go later).
fn spawn_placeholder(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, image_key: &str, size: Val) {
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
        ImageNode::new(assets.image(image_key)).with_mode(NodeImageMode::Stretch),
    ));
}

/// A list entry: placeholder image on the left, a name and detail lines on the right.
fn spawn_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    image_key: &str,
    name: String,
    name_key: Option<String>,
    lines: Vec<String>,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                flex_shrink: 0.,
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
            spawn_placeholder(parent, assets, image_key, Val::Px(40.));

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
            Interaction::default(),
            Pickable::default(),
            InfoTooltip::Combat(stat),
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
    existing_screen_q: Query<Entity, With<PlayingCmp>>,
) {
    if existing_screen_q.iter().next().is_some() {
        return;
    }

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
                    padding: UiRect {
                        top: Val::Px(24.),
                        bottom: Val::Px(14.),
                        ..default()
                    },
                    margin: UiRect::bottom(Val::Px(16.)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(playing_title(&player), "bold", TITLE_TEXT_SIZE, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            // Three main columns.
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: percent(66.),
                    flex_shrink: 0.,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(0.),
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
                    height: Val::Px(135.),
                    flex_shrink: 0.,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.),
                    padding: UiRect {
                        top: Val::Px(18.),
                        bottom: Val::Px(4.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    spawn_playing_action_button(parent, "hunt", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "shop", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "quest", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "train", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "craft", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "work", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "rest", &assets, &localization, lang);
                });
        });
}

/// Column 1: Character portrait image with equipment slot overlays and pet.
fn spawn_image_column(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, player: &Player) {
    parent
        .spawn((
            Node {
                width: percent(31.),
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
                        aspect_ratio: Some(0.88),
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
                            row_gap: Val::Px(1.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in [
                                EquipSlot::Helmet,
                                EquipSlot::Accessory,
                                EquipSlot::WeaponLH,
                                EquipSlot::WeaponRH,
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
                                    Button,
                                    Pickable::default(),
                                    slot,
                                ))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default));
                            }
                        });

                    // Pet image, bottom-left overlay — larger
                    if player.pet.is_some() {
                        parent.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(3.),
                                bottom: Val::Px(3.),
                                width: percent(55.),
                                aspect_ratio: Some(0.92),
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
                width: percent(32.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect {
                    left: Val::Px(18.),
                    right: Val::Px(18.),
                    top: Val::Px(8.),
                    bottom: Val::Px(8.),
                },
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
                        add_text(class_line(player, localization, lang), "bold", 3.0, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        StatLabel(PlayingStat::ClassLine),
                    ));
                    parent.spawn((
                        add_text(format!("AP: {}", player.ap), "bold", 2.2, assets),
                        TextColor(Color::WHITE),
                        StatLabel(PlayingStat::ActionPoints),
                        Interaction::default(),
                        Pickable::default(),
                        InfoTooltip::ActionPoints,
                    ));
                });

            // Health bar
            spawn_bar(parent, assets, true);
            // Mana bar (same height as health)
            spawn_bar(parent, assets, false);

            // Spacer between bars and characteristics
            parent.spawn(Node {
                height: Val::Px(20.),
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
                height: Val::Px(20.),
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
    let bar_height = Val::Px(36.);
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
    mut query: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll, computed) in &mut query {
            // Scroll offset speed factor
            scroll.y -= event.y * 30.0;
            let max_scroll = (computed.content_size().y - computed.size().y).max(0.0);
            scroll.y = scroll.y.clamp(0.0, max_scroll);
        }
    }
}

fn on_right_scrollbar_thumb_drag(
    ev: On<Pointer<Drag>>,
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    track_q: Query<&ComputedNode, With<RightScrollbarTrack>>,
) {
    let Ok((mut scroll, scroll_node)) = scroll_q.single_mut() else {
        return;
    };
    let Ok(track_node) = track_q.single() else {
        return;
    };

    let viewport_height = scroll_node.size().y;
    let content_height = scroll_node.content_size().y;
    let max_scroll = (content_height - viewport_height).max(0.0);
    if max_scroll <= 0.0 || content_height <= 0.0 {
        scroll.y = 0.0;
        return;
    }

    let track_height = track_node.size().y;
    if track_height <= 1.0 {
        return;
    }
    let min_thumb_height = 32.0_f32.min(track_height);
    let thumb_height =
        (viewport_height / content_height * track_height).clamp(min_thumb_height, track_height);
    let max_thumb_top = (track_height - thumb_height).max(1.0);
    scroll.y = (scroll.y + ev.delta.y * max_scroll / max_thumb_top).clamp(0.0, max_scroll);
}

pub fn update_right_scrollbar_system(
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    mut track_q: Query<
        (&ComputedNode, &mut Visibility),
        (With<RightScrollbarTrack>, Without<RightScrollbarThumb>, Without<ScrollableContainer>),
    >,
    mut thumb_q: Query<
        &mut Node,
        (With<RightScrollbarThumb>, Without<RightScrollbarTrack>, Without<ScrollableContainer>),
    >,
) {
    let Ok((mut scroll, scroll_node)) = scroll_q.single_mut() else {
        return;
    };
    let Ok((track_computed, mut track_visibility)) = track_q.single_mut() else {
        return;
    };
    let Ok(mut thumb_node) = thumb_q.single_mut() else {
        return;
    };

    let viewport_height = scroll_node.size().y;
    let content_height = scroll_node.content_size().y;
    let max_scroll = (content_height - viewport_height).max(0.0);

    if max_scroll <= 1.0 || content_height <= viewport_height {
        scroll.y = 0.0;
        *track_visibility = Visibility::Hidden;
        return;
    }

    *track_visibility = Visibility::Visible;
    scroll.y = scroll.y.clamp(0.0, max_scroll);

    let track_height = track_computed.size().y;
    if track_height <= 1.0 {
        *track_visibility = Visibility::Hidden;
        return;
    }
    let min_thumb_height = 32.0_f32.min(track_height);
    let thumb_height =
        (viewport_height / content_height * track_height).clamp(min_thumb_height, track_height);
    let max_thumb_top = (track_height - thumb_height).max(0.0);
    let thumb_top = if max_scroll > 0.0 {
        scroll.y / max_scroll * max_thumb_top
    } else {
        0.0
    };

    thumb_node.height = Val::Px(thumb_height);
    thumb_node.top = Val::Px(thumb_top);
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
            let equipped_key = match slot {
                EquipSlot::Helmet => player.helmet.as_deref(),
                EquipSlot::Accessory => player.accessory.as_deref(),
                EquipSlot::WeaponLH => player.weapon_lh.as_deref().or(player.weapon_2h.as_deref()),
                EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
                EquipSlot::Armor => player.armor.as_deref(),
                EquipSlot::Boots => player.boots.as_deref(),
            };

            if let Some(key) = equipped_key {
                if let Some(weapon) = crate::core::catalog::get_equipment(key) {
                    let name = name_with_level(weapon.name.to_string(), weapon.level);
                    let stat_lines = weapon_stat_lines(&weapon, &player, &localization, lang);

                    spawn_tooltip(&mut commands, &assets, name, stat_lines, &windows);
                }
            }
        }
    }
}

pub fn info_tooltip_system(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    info_q: Query<(&Interaction, &InfoTooltip), Changed<Interaction>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    let lang = settings.language;

    for (interaction, tooltip) in &info_q {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }

        if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
            continue;
        }

        let (title, lines) = match tooltip {
            InfoTooltip::Gold => {
                (localization.get("gold", lang), vec![localization.get("gold_desc", lang)])
            },
            InfoTooltip::ActionPoints => {
                ("AP".to_string(), vec![localization.get("active_points_desc", lang)])
            },
            InfoTooltip::Combat(stat) => {
                let title_key = match stat {
                    PlayingStat::Attack => "attack",
                    PlayingStat::Armor => "armor",
                    PlayingStat::Initiative => "initiative",
                    _ => "",
                };
                (
                    localization.get(title_key, lang),
                    combat_breakdown(*stat, &player, &localization, lang),
                )
            },
            InfoTooltip::Action(act) => {
                let title = localization.get(act, lang);
                let desc_key = format!("{}_desc", act);
                let desc = localization.get_opt(&desc_key, lang)
                    .unwrap_or_else(|| match *act {
                        "hunt" => "Go hunting in the wild to earn gold. Cost: 2 AP. Earns: 10-20 Gold.".to_string(),
                        "shop" => "Buy a random consumable item. Cost: 1 AP, 30 Gold.".to_string(),
                        "quest" => "Embark on an adventure to earn gold and find new equipment. Cost: 3 AP. Earns: 20-40 Gold, 50% chance of equipment.".to_string(),
                        "train" => "Train hard to increase a random attribute. Cost: 2 AP.".to_string(),
                        "craft" => "Craft a piece of equipment suitable for your level. Cost: 2 AP, 15 Gold.".to_string(),
                        "work" => "Do some hard labor to earn a stable gold reward. Cost: 2 AP. Earns: 15-30 Gold.".to_string(),
                        "rest" => "Rest at the tavern to fully recover health and mana. Cost: 1 AP, 10 Gold.".to_string(),
                        _ => "Perform an action.".to_string(),
                    });
                (title, vec![desc])
            }
        };

        spawn_tooltip(&mut commands, &assets, title, lines, &windows);
    }
}

/// Moves the tooltip to follow the mouse cursor.
pub fn tooltip_follow_cursor_system(
    mut tooltip_q: Query<(&mut Node, &TooltipNode)>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        if let Some(cursor) = window.cursor_position() {
            for (mut node, tooltip) in &mut tooltip_q {
                let (left, top) = place_tooltip(
                    cursor,
                    tooltip.width,
                    tooltip.height,
                    window.width(),
                    window.height(),
                );
                node.left = Val::Px(left);
                node.top = Val::Px(top);
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
                width: percent(36.),
                height: percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect {
                    left: Val::Px(8.),
                    right: Val::Px(22.),
                    top: Val::Px(8.),
                    bottom: Val::Px(8.),
                },
                position_type: PositionType::Relative,
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
                        overflow: Overflow::scroll_y(),
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
                            flex_shrink: 0.,
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
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(6.),
                                        ..default()
                                    },
                                    Interaction::default(),
                                    Pickable::default(),
                                    InfoTooltip::Gold,
                                ))
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
                            flex_shrink: 0.,
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
                            flex_shrink: 0.,
                            margin: UiRect::bottom(Val::Px(4.)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 0.,
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
                            flex_shrink: 0.,
                            margin: UiRect::bottom(Val::Px(4.)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 0.,
                            ..default()
                        },
                        PerksList,
                    ));
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(10.),
                        top: Val::Px(8.),
                        bottom: Val::Px(8.),
                        right: Val::Px(3.),
                        border_radius: BorderRadius::all(Val::Px(5.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 170)),
                    Visibility::Hidden,
                    RightScrollbarTrack,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(100.),
                                height: Val::Px(32.),
                                top: Val::Px(0.),
                                border_radius: BorderRadius::all(Val::Px(5.)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba_u8(230, 205, 120, 240)),
                            Button,
                            Interaction::default(),
                            Pickable::default(),
                            RightScrollbarThumb,
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(on_right_scrollbar_thumb_drag);
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

    // Update the equipment image-slots on the portrait.
    for (slot, mut image) in &mut slot_q {
        let equipped_key = match slot {
            EquipSlot::Helmet => player.helmet.as_deref(),
            EquipSlot::Accessory => player.accessory.as_deref(),
            EquipSlot::WeaponLH => player.weapon_lh.as_deref().or(player.weapon_2h.as_deref()),
            EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
            EquipSlot::Armor => player.armor.as_deref(),
            EquipSlot::Boots => player.boots.as_deref(),
        };
        image.image = match equipped_key {
            Some(key) => assets.image(key),
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

            // Equipped items
            let equipped_slots = [
                (&player.helmet, "helmet"),
                (&player.accessory, "accessory"),
                (&player.weapon_lh, "one_hand_weapon"),
                (&player.weapon_rh, "offhand"),
                (&player.weapon_2h, "two_hand_weapon"),
                (&player.armor, "armor"),
                (&player.boots, "boots"),
            ];

            for (slot_val, _kind) in &equipped_slots {
                if let Some(key) = slot_val {
                    if let Some(weapon) = crate::core::catalog::get_equipment(key) {
                        empty = false;
                        spawn_equipment_card(
                            parent,
                            &assets,
                            &weapon.name,
                            format!("{} (Equipped)", name_with_level(weapon.name.to_string(), weapon.level)),
                            weapon_stat_lines(&weapon, &player, &localization, lang),
                            EquipmentCard {
                                key: weapon.name.to_string(),
                                is_equipped: true,
                            },
                        );
                    }
                }
            }

            // Inventory items (unequipped)
            for key in &player.inventory {
                if let Some(weapon) = crate::core::catalog::get_equipment(key) {
                    empty = false;
                    spawn_equipment_card(
                        parent,
                        &assets,
                        &weapon.name,
                        name_with_level(weapon.name.to_string(), weapon.level),
                        weapon_stat_lines(&weapon, &player, &localization, lang),
                        EquipmentCard {
                            key: weapon.name.to_string(),
                            is_equipped: false,
                        },
                    );
                }
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
            for ability_key in &player.abilities {
                if let Some(ability) = crate::core::catalog::get_ability(ability_key) {
                    let desc = localization.get_opt(&format!("{}_desc", ability.name), lang)
                        .unwrap_or_else(|| format!("A powerful {} ability that consumes {} mana.", ability.magic_type.to_lowercase(), ability.mana_cost));
                    spawn_card(
                        parent,
                        &assets,
                        &ability.name,
                        name_with_level(ability.name.to_string(), ability.level),
                        None,
                        vec![
                            ability_detail_line(&ability, &localization, lang),
                            desc,
                        ],
                    );
                }
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
            for perk_key in &player.perks {
                if let Some(perk) = crate::core::catalog::get_perk(perk_key) {
                    let desc = localization.get_opt(&format!("{}_desc", perk.name), lang)
                        .unwrap_or_else(|| format!("An impressive passive perk that empowers your {} capabilities.", perk.theme));
                    spawn_card(
                        parent,
                        &assets,
                        &perk.name,
                        name_with_level(perk.name.to_string(), perk.level),
                        None,
                        vec![desc],
                    );
                }
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
            PlayingStat::CharClass => localized_class_name(&player, &localization, lang),
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
            PlayingStat::ActionPoints => {
                format!("AP: {}", player.ap)
            },
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

pub fn highlight_border<E: std::fmt::Debug + Clone + Reflect>(
    color: Color,
    thickness: Val,
) -> impl Fn(On<Pointer<E>>, Query<(&mut BorderColor, &mut Node)>) {
    move |ev, mut q| {
        if let Ok((mut border_color, mut node)) = q.get_mut(ev.entity) {
            border_color.top = color;
            border_color.right = color;
            border_color.bottom = color;
            border_color.left = color;
            node.border = UiRect::all(thickness);
        }
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
            row_gap: Val::Px(4.),
            margin: UiRect::horizontal(Val::Px(12.)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(88.),
                        height: Val::Px(88.),
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
                    InfoTooltip::Action(action),
                ))
                .observe(handle_playing_action_clicks)
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(highlight_border::<Over>(HOVERED_BUTTON_COLOR, Val::Px(3.)))
                .observe(highlight_border::<Out>(BUTTON_BORDER_COLOR, Val::Px(2.)))
                .observe(highlight_border::<Press>(Color::srgb_u8(240, 190, 60), Val::Px(3.)))
                .observe(highlight_border::<Release>(HOVERED_BUTTON_COLOR, Val::Px(3.)))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default));

            parent.spawn((
                add_text(action_label, "bold", 1.8, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(action.to_string()),
            ));
        });
}

#[derive(Component)]
pub struct EquipmentCard {
    pub key: String,
    pub is_equipped: bool,
}

pub fn handle_playing_action_clicks(
    event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    action_btn_q: Query<&ActionButton>,
) {
    use rand::RngExt;

    if let Ok(action) = action_btn_q.get(event.entity) {
        let cost_gold = match action.0 {
            "craft" => 15,
            "shop" => 30,
            "rest" => 10,
            _ => 0,
        };

        if player.gold < cost_gold {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            return;
        }

        play_audio_msg.write(PlayAudioMsg::new("button"));
        player.gold -= cost_gold;

        // Handle the specific action
        let ap_cost = match action.0 {
            "hunt" => {
                let gold_earned = rand::rng().random_range(10..=20);
                player.gold += gold_earned;
                2
            },
            "work" => {
                let gold_earned = rand::rng().random_range(15..=30);
                player.gold += gold_earned;
                2
            },
            "shop" => {
                let lvl = player.level;
                let items: Vec<&crate::core::catalog::GeneratedEquipment> = crate::core::catalog::GENERATED_EQUIPMENT
                    .iter()
                    .filter(|eq| eq.kind == "consumable" && eq.level <= lvl)
                    .collect();
                use rand::seq::IndexedRandom;
                if let Some(item) = items.choose(&mut rand::rng()) {
                    let name = item.name.to_string();
                    reward_equipment(&mut player, name);
                }
                1
            },
            "quest" => {
                let gold_earned = rand::rng().random_range(20..=40);
                if rand::rng().random_bool(0.5) {
                    let items: Vec<&crate::core::catalog::GeneratedEquipment> = crate::core::catalog::GENERATED_EQUIPMENT
                        .iter()
                        .filter(|eq| eq.level <= player.level)
                        .collect();
                    use rand::seq::IndexedRandom;
                    if let Some(item) = items.choose(&mut rand::rng()) {
                        reward_equipment(&mut player, item.name.to_string());
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
                let items: Vec<&crate::core::catalog::GeneratedEquipment> = crate::core::catalog::GENERATED_EQUIPMENT
                    .iter()
                    .filter(|eq| eq.level == player.level)
                    .collect();
                use rand::seq::IndexedRandom;
                if let Some(item) = items.choose(&mut rand::rng()) {
                    reward_equipment(&mut player, item.name.to_string());
                }
                2
            },
            "rest" => {
                player.health = player.max_health().floor();
                player.mana = player.max_mana().floor();
                1
            },
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

pub fn equip_item(player: &mut Player, key: &str) -> Option<&'static str> {
    if let Some(weapon) = crate::core::catalog::get_equipment(key) {
        // Remove from inventory first
        if let Some(pos) = player.inventory.iter().position(|k| k == key) {
            player.inventory.remove(pos);
        }

        match weapon.kind {
            "consumable" => {
                let name = weapon.name.to_lowercase();
                if name.contains("health") {
                    player.health = player.max_health().floor();
                } else if name.contains("mana") {
                    player.mana = player.max_mana().floor();
                } else if name.contains("strength") {
                    player.strength += 1;
                } else if name.contains("dexterity") {
                    player.dexterity += 1;
                } else if name.contains("constitution") {
                    player.constitution += 1;
                } else if name.contains("intelligence") {
                    player.intelligence += 1;
                } else if name.contains("wisdom") {
                    player.wisdom += 1;
                } else if name.contains("charisma") {
                    player.charisma += 1;
                } else if name.contains("rejuvenation") {
                    player.health = player.max_health().floor();
                    player.mana = player.max_mana().floor();
                } else if name.contains("antidote") {
                    player.health = player.max_health().floor();
                }
                return Some("button");
            },
            "helmet" => {
                if let Some(old) = player.helmet.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "armor" => {
                if let Some(old) = player.armor.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "boots" => {
                if let Some(old) = player.boots.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "accessory" => {
                if let Some(old) = player.accessory.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "one_hand_weapon" => {
                if let Some(old_2h) = player.weapon_2h.take() {
                    player.inventory.push(old_2h);
                }
                if player.weapon_lh.is_none() {
                    player.weapon_lh = Some(key.to_string());
                } else if player.weapon_rh.is_none() {
                    player.weapon_rh = Some(key.to_string());
                } else {
                    if let Some(old) = player.weapon_lh.replace(key.to_string()) {
                        player.inventory.push(old);
                    }
                }
            },
            "two_hand_weapon" => {
                if let Some(old_lh) = player.weapon_lh.take() {
                    player.inventory.push(old_lh);
                }
                if let Some(old_rh) = player.weapon_rh.take() {
                    player.inventory.push(old_rh);
                }
                if let Some(old_2h) = player.weapon_2h.replace(key.to_string()) {
                    player.inventory.push(old_2h);
                }
            },
            "offhand" => {
                if let Some(old_2h) = player.weapon_2h.take() {
                    player.inventory.push(old_2h);
                }
                if let Some(old) = player.weapon_rh.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            _ => {}
        }
    }
    None
}

pub fn unequip_item(player: &mut Player, key: &str) {
    let mut removed = false;
    if player.helmet.as_deref() == Some(key) {
        player.helmet = None;
        removed = true;
    } else if player.armor.as_deref() == Some(key) {
        player.armor = None;
        removed = true;
    } else if player.boots.as_deref() == Some(key) {
        player.boots = None;
        removed = true;
    } else if player.weapon_lh.as_deref() == Some(key) {
        player.weapon_lh = None;
        removed = true;
    } else if player.weapon_rh.as_deref() == Some(key) {
        player.weapon_rh = None;
        removed = true;
    } else if player.weapon_2h.as_deref() == Some(key) {
        player.weapon_2h = None;
        removed = true;
    } else if player.accessory.as_deref() == Some(key) {
        player.accessory = None;
        removed = true;
    }
    if removed {
        player.inventory.push(key.to_string());
    }
}

pub fn unequip_slot(player: &mut Player, slot: EquipSlot) -> bool {
    let key_opt = match slot {
        EquipSlot::Helmet => player.helmet.take(),
        EquipSlot::Accessory => player.accessory.take(),
        EquipSlot::WeaponLH => {
            player.weapon_lh.take().or(player.weapon_2h.take())
        },
        EquipSlot::WeaponRH => player.weapon_rh.take(),
        EquipSlot::Armor => player.armor.take(),
        EquipSlot::Boots => player.boots.take(),
    };
    if let Some(key) = key_opt {
        player.inventory.push(key);
        true
    } else {
        false
    }
}

pub fn reward_equipment(player: &mut Player, key: String) {
    if let Some(weapon) = crate::core::catalog::get_equipment(&key) {
        let is_empty = match weapon.kind {
            "helmet" => player.helmet.is_none(),
            "armor" => player.armor.is_none(),
            "boots" => player.boots.is_none(),
            "accessory" => player.accessory.is_none(),
            "one_hand_weapon" => player.weapon_2h.is_none() && (player.weapon_lh.is_none() || player.weapon_rh.is_none()),
            "two_hand_weapon" => player.weapon_lh.is_none() && player.weapon_rh.is_none() && player.weapon_2h.is_none(),
            "offhand" => player.weapon_2h.is_none() && player.weapon_rh.is_none(),
            _ => false,
        };
        if is_empty {
            equip_item(player, &key);
        } else {
            player.inventory.push(key);
        }
    }
}

pub fn handle_equipment_interactions(
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_interaction_q: Query<(&Interaction, &EquipmentCard), (Changed<Interaction>, With<Button>)>,
    slot_interaction_q: Query<(&Interaction, &EquipSlot), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, card) in &card_interaction_q {
        if *interaction == Interaction::Pressed {
            if card.is_equipped {
                unequip_item(&mut player, &card.key);
                play_audio_msg.write(PlayAudioMsg::new("click"));
            } else {
                let sound = equip_item(&mut player, &card.key).unwrap_or("click");
                play_audio_msg.write(PlayAudioMsg::new(sound));
            }
        }
    }

    for (interaction, slot) in &slot_interaction_q {
        if *interaction == Interaction::Pressed {
            if unequip_slot(&mut player, *slot) {
                play_audio_msg.write(PlayAudioMsg::new("click"));
            }
        }
    }
}

fn spawn_equipment_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    image_key: &str,
    name: String,
    lines: Vec<String>,
    card: EquipmentCard,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                flex_shrink: 0.,
                column_gap: Val::Px(8.),
                padding: UiRect::all(Val::Px(6.)),
                margin: UiRect::bottom(Val::Px(6.)),
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Button,
            Interaction::default(),
            Pickable::default(),
            card,
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(BAR_BG_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .with_children(|parent| {
            spawn_placeholder(parent, assets, image_key, Val::Px(40.));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((add_text(name, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));

                    for line in lines {
                        parent.spawn((
                            add_text(line, "medium", 1.6, assets),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}
