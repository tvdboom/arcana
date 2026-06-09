use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use strum::IntoEnumIterator;

pub use crate::core::actions::{handle_playing_action_clicks, Action, ActionButton};
pub use crate::core::ui::toast::ToastContainer;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::Class;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::buttons::DisabledButton;
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::player::{Attribute, Player};
use crate::core::settings::{Language, Settings};
use crate::core::ui::creation::SelectionItem;
pub use crate::core::ui::level_up::{manage_level_up_overlay, LevelUpPending};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::window::SystemCursorIcon;

const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const MANA_COLOR: Color = Color::srgb_u8(40, 80, 185);

// Viewport-relative icon sizes (scale with window width)
const ICON_ACTION: Val = Val::Vh(8.5); // action button circles
const ICON_BADGE: Val = Val::Vw(1.9); // equipped badge overlay
const ICON_STAT: Val = Val::Vw(2.4); // gold / AP stat icons

#[derive(Component)]
pub struct PlayingCmp;

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
    PetHealth,
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
pub struct PetHealthBarFill;

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

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum RightTab {
    #[default]
    Equipment,
    Abilities,
    Perks,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct RightTabBtn(pub RightTab);

#[derive(Component)]
pub struct EquipmentListWrapper;
#[derive(Component)]
pub struct AbilitiesListWrapper;
#[derive(Component)]
pub struct PerksListWrapper;

/// The equipment image-slots overlaid on the character portrait.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Helmet,
    Accessory,
    Accessory2,
    WeaponLH,
    WeaponRH,
    Armor,
    Boots,
    Gloves,
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
    Action(Action),
    Pet,
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
    if let Some(ref pet) = player.pet {
        if !pet.name.trim().is_empty() {
            return format!("{} & {}", player.name, pet.name);
        }
    }
    player.name.clone()
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

fn name_with_level(
    name: String,
    level: u8,
    _localization: &Localization,
    _lang: Language,
) -> String {
    format!("{} (Lv. {})", name, level)
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
        parts.push(format!("{}: {}s", localization.get("cooldown", lang), stats.cooldown));
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
            (value != 0).then(|| signed_line(capitalize_words(&weapon.name.to_string()), value))
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
            lines.extend(weapon_bonus_lines(player, |weapon| weapon.attack));
            lines
        },
        PlayingStat::Armor => {
            let mut lines = vec![signed_line(
                localization.get("constitution", lang),
                player.constitution() as i32 / 4,
            )];
            lines.extend(weapon_bonus_lines(player, |weapon| weapon.armor));
            lines
        },
        PlayingStat::Initiative => {
            let mut lines = vec![signed_line(
                localization.get("dexterity", lang),
                player.dexterity() as i32 / 2,
            )];
            lines.extend(weapon_bonus_lines(player, |weapon| weapon.initiative));
            if matches!(player.class, Class::Rogue) {
                lines.push(signed_line(localization.get("rogue", lang), 2));
            }
            lines
        },
        _ => vec![],
    }
}

fn spawn_pet_stat_box(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    label_key: &str,
    image_key: &str,
    value: i32,
) {
    parent
        .spawn((
            Node {
                width: percent(32.),
                aspect_ratio: Some(1.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(1.),
                border: UiRect::all(Val::Px(2.)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(PLACEHOLDER_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
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
                add_text(localization.get(label_key, lang), "medium", 1.6, assets),
                TextColor(BUTTON_TEXT_COLOR),
            ));
            parent
                .spawn((add_text(value.to_string(), "bold", 3.0, assets), TextColor(Color::WHITE)));
        });
}

fn spawn_pet_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    windows: &Query<&Window>,
) {
    let Some(ref pet) = player.pet else {
        return;
    };
    let pet_type_name = capitalize_words(&pet.kind.to_lowername());
    let title = format!("{} ({})", pet.name, pet_type_name);
    let desc = localization
        .get_opt(&format!("{}_desc", pet.kind.to_lowername()), lang)
        .unwrap_or_else(|| format!("A loyal {} companion.", pet_type_name.to_lowercase()));
    let stats = pet;

    let tooltip_width = 320.0_f32;
    let tooltip_height = 180.0_f32;

    let (left, top) = if let Ok(window) = windows.single() {
        if let Some(cursor) = window.cursor_position() {
            place_tooltip(cursor, tooltip_width, tooltip_height, window.width(), window.height())
        } else {
            (20., 20.)
        }
    } else {
        (20., 20.)
    };

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
                row_gap: Val::Px(6.),
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
            parent.spawn((add_text(desc, "medium", 1.6, assets), TextColor(Color::WHITE)));
            // Stat boxes row
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::top(Val::Px(4.)),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_pet_stat_box(
                        parent,
                        assets,
                        localization,
                        lang,
                        "attack",
                        "attack_icon",
                        stats.attack,
                    );
                    spawn_pet_stat_box(
                        parent,
                        assets,
                        localization,
                        lang,
                        "armor",
                        "armor_icon",
                        stats.armor,
                    );
                    spawn_pet_stat_box(
                        parent,
                        assets,
                        localization,
                        lang,
                        "initiative",
                        "initiative_icon",
                        stats.initiative,
                    );
                });
        });
}

fn spawn_action_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    action_name: String,
    ap_cost: u32,
    desc: String,
    windows: &Query<&Window>,
) {
    let (window_width, window_height, cursor) = if let Ok(window) = windows.single() {
        (window.width(), window.height(), window.cursor_position())
    } else {
        (1600., 900., None)
    };

    let wrap_limit = ((window_width / 1600.0) * 60.0).clamp(40.0, 90.0) as usize;
    let wrapped: Vec<String> = wrap_tooltip_line(&desc, wrap_limit);
    let desc_max = wrapped.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    // Title width: action name + " (  N)" — approximate as name len + 8 chars
    let title_chars = (action_name.chars().count() + 8) as f32;
    let max_chars = title_chars.max(desc_max as f32);

    let font_size_title = window_height * 0.019;
    let font_size_desc = window_height * 0.016;
    let char_width_desc = font_size_desc * 0.55;
    let line_height_title = font_size_title * 1.35;
    let line_height_desc = font_size_desc * 1.35;

    let tooltip_width =
        (max_chars * char_width_desc + 32.).clamp(200., (window_width - 24.).max(200.));
    let line_count = wrapped.len() as f32;
    let tooltip_height = (line_height_title + line_count * line_height_desc + 36.)
        .clamp(64., (window_height - 24.).max(64.));
    let (left, top) = place_tooltip(
        cursor.unwrap_or(Vec2::new(100., 100.)),
        tooltip_width,
        tooltip_height,
        window_width,
        window_height,
    );

    let icon_size = Val::Px(16.);

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
            // Title row: "ActionName  ([ap icon] N)"
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(action_name, "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                    parent.spawn((
                        add_text("  ".to_string(), "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                    parent.spawn((
                        Node {
                            width: icon_size,
                            height: icon_size,
                            ..default()
                        },
                        ImageNode::new(assets.image("ap")).with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(format!(" {}", ap_cost), "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });
            if !wrapped.is_empty() {
                parent.spawn((
                    add_text(wrapped.join("\n"), "medium", 1.6, assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
}

fn spawn_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    title: String,
    lines: Vec<String>,
    windows: &Query<&Window>,
    price: Option<u32>,
) {
    let (window_width, window_height, cursor) = if let Ok(window) = windows.single() {
        (window.width(), window.height(), window.cursor_position())
    } else {
        (1600., 900., None)
    };

    let wrap_limit = ((window_width / 1600.0) * 60.0).clamp(40.0, 90.0) as usize;
    let wrapped_lines: Vec<String> =
        lines.into_iter().flat_map(|line| wrap_tooltip_line(&line, wrap_limit)).collect();
    let max_chars = std::iter::once(title.as_str())
        .chain(wrapped_lines.iter().map(String::as_str))
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(12) as f32;

    let font_size_title = window_height * 0.019;
    let font_size_desc = window_height * 0.016;
    let char_width_desc = font_size_desc * 0.55;
    let line_height_title = font_size_title * 1.35;
    let line_height_desc = font_size_desc * 1.35;

    let tooltip_width =
        (max_chars * char_width_desc + 32.).clamp(190., (window_width - 24.).max(190.));
    let desc_lines_count = wrapped_lines.len() as f32;
    let tooltip_height = (line_height_title + desc_lines_count * line_height_desc + 32.)
        .clamp(64., (window_height - 24.).max(64.));
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
            // Price display at top-right corner (if provided)
            if let Some(price_value) = price {
                parent
                    .spawn((Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(10.),
                        top: Val::Px(10.),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.),
                        ..default()
                    },))
                    .with_children(|parent| {
                        // Gold icon
                        parent.spawn((
                            Node {
                                width: ICON_BADGE,
                                height: ICON_BADGE,
                                ..default()
                            },
                            ImageNode::new(assets.image("gold")).with_mode(NodeImageMode::Stretch),
                        ));

                        // Price number
                        parent.spawn((
                            add_text(format!("{}", price_value), "bold", 1.9, assets),
                            TextColor(BUTTON_TEXT_COLOR),
                        ));
                    });
            }

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
fn spawn_placeholder(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    image_key: &str,
    size: Val,
) {
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
            spawn_placeholder(parent, assets, image_key, ICON_ITEM);

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
    root_node.padding = UiRect::all(Val::Px(0.));

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode {
                image: assets.image("base"),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.40, 0.40, 0.40, 1.0),
                ..default()
            },
            PlayingCmp,
        ))
        .with_children(|parent| {
            // Toast container: stacks notifications top-right of the whole screen
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(8.),
                    right: Val::Percent(2.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.),
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                PlayingCmp,
                ToastContainer,
                GlobalZIndex(600),
            ));

            // Content column: maintains aspect ratio on wide screens, centered horizontally
            parent
                .spawn(Node {
                    height: percent(100.),
                    max_width: percent(100.),
                    aspect_ratio: Some(16. / 9.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(4.),
                    align_self: AlignSelf::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // Character name, top centered with banner background.
                    parent
                        .spawn(Node {
                            align_self: AlignSelf::Center,
                            width: Val::Auto,
                            min_width: Val::Vh(50.0),
                            height: Val::Vh(7.11),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::horizontal(Val::Px(100.0)),
                            margin: UiRect {
                                top: Val::Vh(2.67),
                                bottom: Val::Vh(1.78),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            // Background banner image stretches to match parent's size
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: percent(100.0),
                                    height: percent(100.0),
                                    ..default()
                                },
                                ImageNode::new(assets.image("banner"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));

                            // Name text sits on top
                            parent.spawn((
                                add_text(playing_title(&player), "bold", 4.2, &assets),
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
                            height: Val::Vh(14.5),
                            flex_shrink: 0.,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.),
                            padding: UiRect {
                                top: Val::Vh(1.5),
                                bottom: Val::Px(4.),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_playing_action_button(
                                parent,
                                Action::Rest,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Study,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Work,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Train,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Craft,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Shop,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Hunt,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Quest,
                                &assets,
                                &localization,
                                lang,
                            );
                        });
                });
        });
}

/// Column 1: Character portrait image with equipment slot overlays and pet.
fn spawn_image_column(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, player: &Player) {
    parent
        .spawn((Node {
            width: percent(33.5),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect::all(Val::Px(6.)),
            ..default()
        },))
        .with_children(|parent| {
            // Portrait (relative container for equipment slot / pet overlays)
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        aspect_ratio: Some(0.88),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(3.)),
                        margin: UiRect::top(Val::Px(2.)),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(portrait_key(player)))
                        .with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    // Left column: two accessory slots
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(2.),
                            top: Val::Percent(2.),
                            width: percent(16.),
                            height: percent(30.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            row_gap: Val::Px(1.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in [EquipSlot::Accessory, EquipSlot::Accessory2] {
                                parent
                                    .spawn((
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
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(handle_equipment_slot_click);
                            }
                        });

                    // Right column: 6 equipment slots
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
                                EquipSlot::Armor,
                                EquipSlot::WeaponLH,
                                EquipSlot::WeaponRH,
                                EquipSlot::Gloves,
                                EquipSlot::Boots,
                            ] {
                                parent
                                    .spawn((
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
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(handle_equipment_slot_click);
                            }
                        });

                    // Pet image, bottom-left overlay — larger
                    if player.pet.is_some() {
                        let pet = player.pet.as_ref().unwrap();
                        parent
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(3.),
                                    bottom: Val::Px(3.),
                                    width: percent(55.),
                                    aspect_ratio: Some(0.92),
                                    border: UiRect::all(Val::Px(2.)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::FlexEnd,
                                    ..default()
                                },
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                ImageNode::new(assets.image(pet.kind.to_lowername()))
                                    .with_mode(NodeImageMode::Stretch),
                                PetImage,
                                Interaction::default(),
                                Pickable::default(),
                                InfoTooltip::Pet,
                            ))
                            .with_children(|parent| {
                                // Thicker health bar container at the bottom of the pet image
                                parent
                                    .spawn((
                                        Node {
                                            width: percent(90.),
                                            height: Val::Px(24.),
                                            border: UiRect::all(Val::Px(1.5)),
                                            align_self: AlignSelf::Center,
                                            margin: UiRect::bottom(Val::Px(4.)),
                                            position_type: PositionType::Relative,
                                            ..default()
                                        },
                                        BackgroundColor(BAR_BG_COLOR),
                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                    ))
                                    .with_children(|parent| {
                                        // Health bar fill
                                        parent.spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: Val::Px(0.),
                                                top: Val::Px(0.),
                                                width: percent(100.),
                                                height: percent(100.),
                                                ..default()
                                            },
                                            BackgroundColor(HEALTH_COLOR),
                                            PetHealthBarFill,
                                        ));

                                        // Text overlay
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
                                                    add_text("", "bold", 1.4, assets),
                                                    TextColor(Color::WHITE),
                                                    StatLabel(PlayingStat::PetHealth),
                                                ));
                                            });
                                    });
                            });
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
        .spawn((Node {
            width: percent(32.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect {
                left: Val::Px(12.),
                right: Val::Px(12.),
                top: Val::Px(8.),
                bottom: Val::Px(8.),
            },
            row_gap: Val::Px(4.),
            overflow: Overflow::clip(),
            ..default()
        },))
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
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.),
                                ..default()
                            },
                            Interaction::default(),
                            Pickable::default(),
                            InfoTooltip::ActionPoints,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    width: ICON_STAT,
                                    height: ICON_STAT,
                                    flex_shrink: 0.,
                                    ..default()
                                },
                                ImageNode::new(assets.image("ap"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text(format!("{}", player.ap), "bold", 2.4, assets),
                                TextColor(Color::WHITE),
                                StatLabel(PlayingStat::ActionPoints),
                            ));
                        });
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
    level_up: Res<LevelUpPending>,
    slot_q: Query<(&Interaction, &EquipSlot), Changed<Interaction>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if level_up.active {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

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
                EquipSlot::Accessory2 => player.accessory2.as_deref(),
                EquipSlot::WeaponLH => player.weapon_lh.as_deref().or(player.weapon_2h.as_deref()),
                EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
                EquipSlot::Armor => player.armor.as_deref(),
                EquipSlot::Boots => player.boots.as_deref(),
                EquipSlot::Gloves => player.gloves.as_deref(),
            };

            if let Some(key) = equipped_key {
                if let Some(weapon) = crate::core::catalog::get_equipment(key) {
                    let name =
                        name_with_level(weapon.name.to_string(), weapon.level, &localization, lang);
                    let stat_lines = weapon_stat_lines(&weapon, &player, &localization, lang);

                    spawn_tooltip(
                        &mut commands,
                        &assets,
                        name,
                        stat_lines,
                        &windows,
                        Some(weapon.price),
                    );
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
    level_up: Res<LevelUpPending>,
    info_q: Query<(&Interaction, &InfoTooltip), Changed<Interaction>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if level_up.active {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    let lang = settings.language;

    for (interaction, tooltip) in &info_q {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }

        if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
            continue;
        }

        if matches!(tooltip, InfoTooltip::Pet) {
            spawn_pet_tooltip(&mut commands, &assets, &localization, lang, &player, &windows);
            continue;
        }

        if let InfoTooltip::Action(act) = tooltip {
            let ap_cost = act.ap_cost();
            let action_name = localization.get(act.to_lowername().as_str(), lang);
            let desc_key = format!("{}_desc", act.to_lowername());
            let desc = match act {
                Action::Work => {
                    let charisma = player.charisma() as i32;
                    let level = player.level as i32;
                    let base = charisma * level;
                    let min_gold = (base * 4 / 5).max(1);
                    let max_gold = base * 6 / 5;
                    let base_text = localization.get_opt("work_desc_base", lang)
                        .unwrap_or_else(|| "Earn gold by working. Scales with Level and Charisma.".to_string());
                    let format_str = localization.get_opt("work_desc_format", lang)
                        .unwrap_or_else(|| "{} Earns: {min}-{max} Gold.".to_string());
                    format_str.replace("{}", &base_text)
                        .replace("{min}", &min_gold.to_string())
                        .replace("{max}", &max_gold.to_string())
                },
                Action::Rest => {
                    let wisdom = player.wisdom() as i32;
                    let level = player.level as i32;
                    let base = wisdom * level;
                    let min_r = (base * 4 / 5).max(1);
                    let max_r = base * 6 / 5;
                    let wisdom_bonus = (player.wisdom() as f32 - 10.).max(0.) * 0.005;
                    let max_pct = ((0.05 + wisdom_bonus).min(0.20) * 100.) as u32;
                    let base_text = localization.get_opt("rest_desc_base", lang)
                        .unwrap_or_else(|| "Rest to recover health and mana. Scales with Level and Wisdom.".to_string());
                    let format_str = localization.get_opt("rest_desc_format", lang)
                        .unwrap_or_else(|| "{} Recovers: {min}-{max} HP/MP. {pct}% chance of +max HP/MP.".to_string());
                    format_str.replace("{}", &base_text)
                        .replace("{min}", &min_r.to_string())
                        .replace("{max}", &max_r.to_string())
                        .replace("{pct}", &max_pct.to_string())
                },
                Action::Study => {
                    let int_bonus = (player.intelligence() as f32 - 10.).max(0.) * 0.025;
                    let perk_pct = ((0.333 + int_bonus).min(0.65) * 100.) as u32;
                    let abil_pct = ((0.200 + int_bonus).min(0.45) * 100.) as u32;
                    let base_text = localization.get_opt("study_desc_base", lang)
                        .unwrap_or_else(|| "Study tomes and scrolls. Scales with Intelligence.".to_string());
                    let format_str = localization.get_opt("study_desc_format", lang)
                        .unwrap_or_else(|| "{} Perk: {perk}%, Ability: {abil}%, +1 Wisdom: 5%.".to_string());
                    format_str.replace("{}", &base_text)
                        .replace("{perk}", &perk_pct.to_string())
                        .replace("{abil}", &abil_pct.to_string())
                },
                _ => localization.get_opt(&desc_key, lang)
                    .unwrap_or_else(|| match act {
                        Action::Hunt => "Go hunting in the wild to earn gold.".to_string(),
                        Action::Shop => "Buy a random consumable item.".to_string(),
                        Action::Quest => "Embark on an adventure to earn gold and find new equipment.".to_string(),
                        Action::Train => "Train hard to improve your combat abilities. Scales with Strength and Dexterity.".to_string(),
                        Action::Craft => "Craft a piece of equipment suitable for your level.".to_string(),
                        _ => "Perform an action.".to_string(),
                    }),
            };
            spawn_action_tooltip(&mut commands, &assets, action_name, ap_cost, desc, &windows);
            continue;
        }

        let (title, lines) = match tooltip {
            InfoTooltip::Gold => {
                (localization.get("gold", lang), vec![localization.get("gold_desc", lang)])
            },
            InfoTooltip::ActionPoints => (
                localization.get("active_points", lang),
                vec![localization.get("active_points_desc", lang)],
            ),
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
            InfoTooltip::Action(_) | InfoTooltip::Pet => unreachable!(),
        };

        spawn_tooltip(&mut commands, &assets, title, lines, &windows, None);
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

fn handle_tab_click(
    ev: On<Pointer<Click>>,
    btn_q: Query<&RightTabBtn>,
    mut right_tab: ResMut<RightTab>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(ev.entity) {
        if *right_tab != btn.0 {
            *right_tab = btn.0;
            play_audio_msg.write(PlayAudioMsg::new("button"));
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
        .spawn((Node {
            width: percent(33.5),
            height: percent(91.5),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect {
                left: Val::Px(8.),
                right: Val::Px(22.),
                top: Val::Px(8.),
                bottom: Val::Px(0.),
            },
            position_type: PositionType::Relative,
            ..default()
        },))
        .with_children(|parent| {
            // Tab row + gold icon at the top
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.,
                    margin: UiRect::bottom(Val::Px(4.)),
                    ..default()
                })
                .with_children(|parent| {
                    // Left: three tab buttons
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for (tab, key, fallback) in [
                                (RightTab::Equipment, "equipment_tab", "Equipment"),
                                (RightTab::Abilities, "abilities_tab", "Abilities"),
                                (RightTab::Perks, "perks_tab", "Perks"),
                            ] {
                                let is_active = tab == RightTab::Equipment;
                                let bg_color = if is_active {
                                    NORMAL_BUTTON_COLOR
                                } else {
                                    Color::srgba_u8(20, 20, 35, 200)
                                };
                                let label = localization
                                    .get_opt(key, lang)
                                    .unwrap_or_else(|| fallback.to_string());
                                parent
                                    .spawn((
                                        Node {
                                            padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                                            border: UiRect::all(Val::Px(1.)),
                                            ..default()
                                        },
                                        BackgroundColor(bg_color),
                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                        Button,
                                        Interaction::default(),
                                        Pickable {
                                            should_block_lower: true,
                                            is_hoverable: true,
                                        },
                                        RightTabBtn(tab),
                                    ))
                                    .observe(handle_tab_click)
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            add_text(label, "bold", 1.8, assets),
                                            TextColor(BUTTON_TEXT_COLOR),
                                            LocalizedText(key.to_string()),
                                        ));
                                    });
                            }
                        });

                    // Right: gold icon + amount
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
                                    width: ICON_STAT,
                                    height: ICON_STAT,
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

            // Separator between tabs and content list
            parent.spawn((
                Node {
                    width: percent(100.),
                    height: Val::Px(1.),
                    margin: UiRect {
                        top: Val::Px(10.),
                        bottom: Val::Px(18.),
                        ..default()
                    },
                    ..default()
                },
                BackgroundColor(BUTTON_BORDER_COLOR),
            ));

            // Scrollable content area
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
                    // Equipment wrapper (visible by default)
                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                ..default()
                            },
                            EquipmentListWrapper,
                            Visibility::Inherited,
                        ))
                        .with_children(|parent| {
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
                        });

                    // Abilities wrapper (hidden by default)
                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                display: Display::None,
                                ..default()
                            },
                            AbilitiesListWrapper,
                            Visibility::Hidden,
                        ))
                        .with_children(|parent| {
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
                        });

                    // Perks wrapper (hidden by default)
                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                display: Display::None,
                                ..default()
                            },
                            PerksListWrapper,
                            Visibility::Hidden,
                        ))
                        .with_children(|parent| {
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
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(10.),
                        top: Val::Px(85.),
                        bottom: Val::Px(0.),
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

#[derive(SystemParam)]
pub struct RebuildPlayingListsQueries<'w, 's> {
    pub equip_q: Query<'w, 's, Entity, With<EquipmentList>>,
    pub abil_q: Query<'w, 's, Entity, With<AbilitiesList>>,
    pub perk_q: Query<'w, 's, Entity, With<PerksList>>,
    pub slot_q: Query<'w, 's, (&'static EquipSlot, &'static mut ImageNode)>,
    pub slot_vis_q: Query<'w, 's, (&'static EquipSlot, &'static mut Visibility)>,
    pub children_q: Query<'w, 's, &'static Children>,
    pub equip_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<EquipmentListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub abil_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<AbilitiesListWrapper>,
            Without<EquipmentListWrapper>,
            Without<PerksListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub perk_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<PerksListWrapper>,
            Without<EquipmentListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub tab_btn_q: Query<
        'w,
        's,
        (
            Entity,
            &'static RightTabBtn,
            &'static mut BackgroundColor,
            &'static mut BorderColor,
            &'static mut Node,
        ),
        (
            With<RightTabBtn>,
            Without<EquipmentListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
            Without<EquipSlot>,
        ),
    >,
    pub text_color_q: Query<'w, 's, &'static mut TextColor>,
    pub scroll_q: Query<'w, 's, &'static mut ScrollPosition, With<ScrollableContainer>>,
}

/// Rebuilds the equipment, abilities and perks lists whenever the player or
/// language changes (these have a variable number of entries).
pub fn rebuild_playing_lists(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    player: Res<Player>,
    right_tab: Res<RightTab>,
    mut queries: RebuildPlayingListsQueries<'_, '_>,
) {
    let lang = settings.language;

    // Update the equipment image-slots on the portrait.
    for (slot, mut image) in &mut queries.slot_q {
        let equipped_key = match slot {
            EquipSlot::Helmet => player.helmet.as_deref(),
            EquipSlot::Accessory => player.accessory.as_deref(),
            EquipSlot::Accessory2 => player.accessory2.as_deref(),
            EquipSlot::WeaponLH => player.weapon_lh.as_deref().or(player.weapon_2h.as_deref()),
            EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
            EquipSlot::Armor => player.armor.as_deref(),
            EquipSlot::Boots => player.boots.as_deref(),
            EquipSlot::Gloves => player.gloves.as_deref(),
        };
        image.image = match equipped_key {
            Some(key) => assets.image(key),
            None => assets.image("stone"),
        };
    }

    // Show only filled slots; hide WeaponRH when a 2H weapon is also equipped
    for (slot, mut vis) in &mut queries.slot_vis_q {
        let visible = match slot {
            EquipSlot::Helmet => player.helmet.is_some(),
            EquipSlot::Accessory => player.accessory.is_some(),
            EquipSlot::Accessory2 => player.accessory2.is_some(),
            EquipSlot::WeaponLH => player.weapon_lh.is_some() || player.weapon_2h.is_some(),
            EquipSlot::WeaponRH => player.weapon_rh.is_some() && player.weapon_2h.is_none(),
            EquipSlot::Armor => player.armor.is_some(),
            EquipSlot::Boots => player.boots.is_some(),
            EquipSlot::Gloves => player.gloves.is_some(),
        };
        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
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
    if let Ok(entity) = queries.equip_q.single() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            let mut empty = true;

            // Equipped items
            let equipped_slots = [
                (&player.helmet, "helmet"),
                (&player.accessory, "accessory"),
                (&player.accessory2, "accessory"),
                (&player.weapon_lh, "one_hand_weapon"),
                (&player.weapon_rh, "offhand"),
                (&player.weapon_2h, "two_hand_weapon"),
                (&player.armor, "armor"),
                (&player.boots, "boots"),
                (&player.gloves, "gloves"),
            ];

            // Collect equipped items and sort by level then name
            let mut equipped_items: Vec<crate::core::catalog::GeneratedEquipment> = equipped_slots
                .iter()
                .filter_map(|(slot_val, _)| {
                    slot_val.as_deref().and_then(crate::core::catalog::get_equipment)
                })
                .collect();
            equipped_items.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for weapon in &equipped_items {
                empty = false;
                spawn_equipment_card(
                    parent,
                    &assets,
                    &weapon.name,
                    name_with_level(weapon.name.to_string(), weapon.level, &localization, lang),
                    weapon_stat_lines(&weapon, &player, &localization, lang),
                    EquipmentCard {
                        key: weapon.name.to_string(),
                        is_equipped: true,
                        price: weapon.price,
                    },
                );
            }

            // Inventory items (unequipped), sorted by level then name
            let mut inventory_items: Vec<crate::core::catalog::GeneratedEquipment> = player
                .inventory
                .iter()
                .filter_map(|key| crate::core::catalog::get_equipment(key))
                .collect();
            inventory_items.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for weapon in &inventory_items {
                empty = false;
                spawn_equipment_card(
                    parent,
                    &assets,
                    &weapon.name,
                    name_with_level(weapon.name.to_string(), weapon.level, &localization, lang),
                    weapon_stat_lines(&weapon, &player, &localization, lang),
                    EquipmentCard {
                        key: weapon.name.to_string(),
                        is_equipped: false,
                        price: weapon.price,
                    },
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
    if let Ok(entity) = queries.abil_q.single() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            if player.abilities.is_empty() {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            let mut sorted_abilities: Vec<_> = player
                .abilities
                .iter()
                .filter_map(|key| crate::core::catalog::get_ability(key))
                .collect();
            sorted_abilities.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for ability in &sorted_abilities {
                let desc = localization
                    .get_opt(&format!("{}_desc", ability.name), lang)
                    .unwrap_or_else(|| {
                        format!(
                            "A powerful {} ability that consumes {} mana.",
                            ability.magic_type.to_lowercase(),
                            ability.mana_cost
                        )
                    });
                spawn_card(
                    parent,
                    &assets,
                    &ability.name,
                    name_with_level(ability.name.to_string(), ability.level, &localization, lang),
                    None,
                    vec![ability_detail_line(&ability, &localization, lang), desc],
                );
            }
        });
    }

    // Perks list.
    if let Ok(entity) = queries.perk_q.single() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            if player.perks.is_empty() {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            let mut sorted_perks: Vec<_> =
                player.perks.iter().filter_map(|key| crate::core::catalog::get_perk(key)).collect();
            sorted_perks.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for perk in &sorted_perks {
                let desc = localization
                    .get_opt(&format!("{}_desc", perk.name), lang)
                    .unwrap_or_else(|| {
                        format!(
                            "An impressive passive perk that empowers your {} capabilities.",
                            perk.theme
                        )
                    });
                spawn_card(
                    parent,
                    &assets,
                    &perk.name,
                    name_with_level(perk.name.to_string(), perk.level, &localization, lang),
                    None,
                    vec![desc],
                );
            }
        });
    }

    // Update wrapper visibility and display based on active tab.
    if let Ok((mut node, mut vis)) = queries.equip_wrap_q.single_mut() {
        if *right_tab == RightTab::Equipment {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }
    if let Ok((mut node, mut vis)) = queries.abil_wrap_q.single_mut() {
        if *right_tab == RightTab::Abilities {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }
    if let Ok((mut node, mut vis)) = queries.perk_wrap_q.single_mut() {
        if *right_tab == RightTab::Perks {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }

    // Reset scroll position to top if the tab was just changed.
    if right_tab.is_changed() {
        if let Ok(mut scroll) = queries.scroll_q.single_mut() {
            scroll.y = 0.0;
        }
    }

    // Update tab button background, border colors, text color of their children, and border thickness.
    for (entity, btn, mut bg, mut border, mut node) in &mut queries.tab_btn_q {
        let active = btn.0 == *right_tab;
        if active {
            *bg = BackgroundColor(NORMAL_BUTTON_COLOR);
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.2)); // Bright gold border
            node.border = UiRect::all(Val::Px(2.)); // Thicker border
        } else {
            *bg = BackgroundColor(Color::srgba_u8(12, 12, 18, 240)); // Much darker background
            *border = BorderColor::all(Color::srgba(0.2, 0.2, 0.2, 0.5)); // Muted border
            node.border = UiRect::all(Val::Px(1.));
        }

        if let Ok(children) = queries.children_q.get(entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = queries.text_color_q.get_mut(child) {
                    txt_col.0 = if active {
                        BUTTON_TEXT_COLOR // Bright gold for active
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.4) // Dimmed gray-white for inactive
                    };
                }
            }
        }
    }
}

pub fn tab_button_hover_system(
    right_tab: Res<RightTab>,
    mut tab_btn_q: Query<(Entity, &RightTabBtn, &Interaction, &mut BackgroundColor)>,
    children_q: Query<&Children>,
    mut text_color_q: Query<&mut TextColor>,
) {
    for (entity, btn, interaction, mut bg) in &mut tab_btn_q {
        let active = btn.0 == *right_tab;
        *bg = match (active, interaction) {
            (_, Interaction::Pressed) => BackgroundColor(Color::srgba_u8(30, 30, 50, 240)),
            (true, Interaction::Hovered) => BackgroundColor(NORMAL_BUTTON_COLOR),
            (false, Interaction::Hovered) => BackgroundColor(BUTTON_TEXT_COLOR),
            (true, Interaction::None) => BackgroundColor(NORMAL_BUTTON_COLOR),
            (false, Interaction::None) => BackgroundColor(Color::srgba_u8(12, 12, 18, 240)),
        };

        if let Ok(children) = children_q.get(entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = text_color_q.get_mut(child) {
                    txt_col.0 = match (active, interaction) {
                        (_, Interaction::Pressed) => Color::srgba(1.0, 1.0, 1.0, 0.4),
                        (true, _) => BUTTON_TEXT_COLOR,
                        (false, Interaction::Hovered) => Color::BLACK,
                        (false, Interaction::None) => Color::srgba(1.0, 1.0, 1.0, 0.4),
                    };
                }
            }
        }
    }
}

pub fn update_playing_screen(
    player: Res<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut text_q: Query<(&mut Text, &StatLabel)>,
    mut attr_q: Query<(&mut Text, &AttrValue), Without<StatLabel>>,
    mut hbar_q: Query<
        &mut Node,
        (With<HealthBarFill>, Without<ManaBarFill>, Without<PetHealthBarFill>),
    >,
    mut mbar_q: Query<
        &mut Node,
        (With<ManaBarFill>, Without<HealthBarFill>, Without<PetHealthBarFill>),
    >,
    mut pet_hbar_q: Query<
        &mut Node,
        (With<PetHealthBarFill>, Without<HealthBarFill>, Without<ManaBarFill>),
    >,
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
            PlayingStat::CharAge => {
                format!("{} {}", player.age, localization.get("years", lang))
            },
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
            PlayingStat::ActionPoints => format!("{}", player.ap),
            PlayingStat::PetHealth => {
                if let Some(ref pet) = player.pet {
                    let pet_max = pet.max_health as f32;
                    let pet_current = pet.health as f32;
                    format!(
                        "{} / {} {}",
                        pet_current.max(0.) as i32,
                        pet_max as i32,
                        localization.get("health", lang)
                    )
                } else {
                    "".to_string()
                }
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
    if let Some(ref pet) = player.pet {
        if let Ok(mut node) = pet_hbar_q.single_mut() {
            let pet_max = pet.max_health as f32;
            let pet_current = pet.health as f32;
            let ratio = (pet_current / pet_max).clamp(0., 1.) * 100.;
            node.width = percent(ratio);
        }
    }
}

pub fn highlight_border<E: std::fmt::Debug + Clone + Reflect>(
    color: Color,
    thickness: Val,
) -> impl Fn(On<Pointer<E>>, Query<(&mut BorderColor, &mut Node, Option<&DisabledButton>)>) {
    move |ev, mut q| {
        if let Ok((mut border_color, mut node, disabled)) = q.get_mut(ev.entity) {
            if disabled.is_none() {
                border_color.top = color;
                border_color.right = color;
                border_color.bottom = color;
                border_color.left = color;
                node.border = UiRect::all(thickness);
            }
        }
    }
}

pub fn spawn_playing_action_button(
    parent: &mut ChildSpawnerCommands,
    action: Action,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    let action_label = localization.get(action.to_lowername().as_str(), lang);
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(4.),
            margin: UiRect::horizontal(Val::Vw(0.9)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: ICON_ACTION,
                        height: ICON_ACTION,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.)),
                        border_radius: BorderRadius::all(percent(50.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(format!("action_{}", action.to_lowername())))
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

            // Plain action name label below icon
            parent.spawn((
                add_text(action_label, "bold", 1.8, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(action.to_lowername().to_string()),
            ));
        });
}

#[derive(Component)]
pub struct EquipmentCard {
    pub key: String,
    pub is_equipped: bool,
    pub price: u32,
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
                if player.accessory.is_none() {
                    player.accessory = Some(key.to_string());
                } else if player.accessory2.is_none() {
                    player.accessory2 = Some(key.to_string());
                } else {
                    if let Some(old) = player.accessory.replace(key.to_string()) {
                        player.inventory.push(old);
                    }
                }
            },
            "gloves" => {
                if let Some(old) = player.gloves.replace(key.to_string()) {
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
            _ => {},
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
    } else if player.accessory2.as_deref() == Some(key) {
        player.accessory2 = None;
        removed = true;
    } else if player.gloves.as_deref() == Some(key) {
        player.gloves = None;
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
        EquipSlot::Accessory2 => player.accessory2.take(),
        EquipSlot::WeaponLH => player.weapon_lh.take().or(player.weapon_2h.take()),
        EquipSlot::WeaponRH => player.weapon_rh.take(),
        EquipSlot::Armor => player.armor.take(),
        EquipSlot::Boots => player.boots.take(),
        EquipSlot::Gloves => player.gloves.take(),
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
            "gloves" => player.gloves.is_none(),
            "accessory" => player.accessory.is_none() || player.accessory2.is_none(),
            "one_hand_weapon" => {
                player.weapon_2h.is_none()
                    && (player.weapon_lh.is_none() || player.weapon_rh.is_none())
            },
            "two_hand_weapon" => {
                player.weapon_lh.is_none()
                    && player.weapon_rh.is_none()
                    && player.weapon_2h.is_none()
            },
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

pub fn handle_equipment_card_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    window_e: Single<Entity, With<Window>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_q: Query<&EquipmentCard>,
) {
    if let Ok(card) = card_q.get(event.entity) {
        // Right-click: sell item for full price
        if event.button == PointerButton::Secondary {
            let sell_price = card.price;

            // If equipped, unequip first
            if card.is_equipped {
                unequip_item(&mut player, &card.key);
            }

            // Remove from inventory
            if let Some(pos) = player.inventory.iter().position(|k| k == &card.key) {
                player.inventory.remove(pos);
                player.gold += sell_price;
                play_audio_msg.write(PlayAudioMsg::new("coins"));
                commands.entity(*window_e).insert(bevy::window::CursorIcon::from(
                    bevy::window::SystemCursorIcon::Default,
                ));
            }
            return;
        }

        // Left-click: equip/unequip
        if card.is_equipped {
            unequip_item(&mut player, &card.key);
            play_audio_msg.write(PlayAudioMsg::new("click"));
        } else {
            // Check consumable restrictions
            if let Some(eq) = crate::core::catalog::get_equipment(&card.key) {
                if eq.kind == "consumable" {
                    let name = eq.name.to_lowercase();
                    let blocked = if name.contains("rejuvenation") {
                        player.health >= player.max_health().floor()
                            && player.mana >= player.max_mana().floor()
                    } else if name.contains("health") || name.contains("antidote") {
                        player.health >= player.max_health().floor()
                    } else if name.contains("mana") {
                        player.mana >= player.max_mana().floor()
                    } else {
                        false
                    };
                    if blocked {
                        play_audio_msg.write(PlayAudioMsg::new("error"));
                        return;
                    }
                }
            }
            let sound = equip_item(&mut player, &card.key).unwrap_or("click");
            play_audio_msg.write(PlayAudioMsg::new(sound));
        }
    }
}

/*
pub fn tick_gold_toasts(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut GoldToast)>,
) {
    for (entity, mut toast) in &mut q {
        toast.timer -= time.delta_secs();
        if toast.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}
*/

pub fn handle_equipment_slot_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    window_e: Single<Entity, With<Window>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    slot_q: Query<&EquipSlot>,
) {
    if let Ok(slot) = slot_q.get(event.entity) {
        // Get the equipped item key for this slot
        let equipped_key = match slot {
            EquipSlot::Helmet => player.helmet.as_ref(),
            EquipSlot::Accessory => player.accessory.as_ref(),
            EquipSlot::Accessory2 => player.accessory2.as_ref(),
            EquipSlot::WeaponLH => player.weapon_lh.as_ref().or(player.weapon_2h.as_ref()),
            EquipSlot::WeaponRH => player.weapon_rh.as_ref(),
            EquipSlot::Armor => player.armor.as_ref(),
            EquipSlot::Boots => player.boots.as_ref(),
            EquipSlot::Gloves => player.gloves.as_ref(),
        };

        if let Some(key) = equipped_key {
            let key_str = key.to_string();

            // Right-click: sell equipped item for full price
            if event.button == PointerButton::Secondary {
                if let Some(eq) = crate::core::catalog::get_equipment(&key_str) {
                    let sell_price = eq.price;

                    // Directly unequip from the slot without adding to inventory
                    match slot {
                        EquipSlot::Helmet => player.helmet = None,
                        EquipSlot::Accessory => player.accessory = None,
                        EquipSlot::Accessory2 => player.accessory2 = None,
                        EquipSlot::WeaponLH => {
                            if player.weapon_lh.is_some() {
                                player.weapon_lh = None;
                            } else {
                                player.weapon_2h = None;
                            }
                        },
                        EquipSlot::WeaponRH => player.weapon_rh = None,
                        EquipSlot::Armor => player.armor = None,
                        EquipSlot::Boots => player.boots = None,
                        EquipSlot::Gloves => player.gloves = None,
                    }

                    player.gold += sell_price;
                    play_audio_msg.write(PlayAudioMsg::new("coins"));
                    commands.entity(*window_e).insert(bevy::window::CursorIcon::from(
                        bevy::window::SystemCursorIcon::Default,
                    ));
                }
            } else {
                // Left-click: unequip item
                if unequip_slot(&mut player, *slot) {
                    play_audio_msg.write(PlayAudioMsg::new("click"));
                }
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
    let is_equipped = card.is_equipped;
    let price = card.price;
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
                position_type: PositionType::Relative,
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
        .observe(handle_equipment_card_click)
        .with_children(|parent| {
            // Top-right corner: equipped badge (if equipped) + price display
            parent
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(4.),
                    top: Val::Px(4.),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.),
                    ..default()
                },))
                .with_children(|parent| {
                    // Equipped badge (left of price)
                    if is_equipped {
                        parent.spawn((
                            Node {
                                width: ICON_BADGE,
                                height: ICON_BADGE,
                                ..default()
                            },
                            ImageNode::new(assets.image("equipped"))
                                .with_mode(NodeImageMode::Stretch),
                        ));
                    }

                    // Gold icon (same size as equipped badge)
                    parent.spawn((
                        Node {
                            width: ICON_BADGE,
                            height: ICON_BADGE,
                            ..default()
                        },
                        ImageNode::new(assets.image("gold")).with_mode(NodeImageMode::Stretch),
                    ));

                    // Price number (same color as weapon name)
                    parent.spawn((
                        add_text(format!("{}", price), "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            spawn_placeholder(parent, assets, image_key, ICON_ITEM);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((add_text(name, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));

                    for line in lines {
                        parent.spawn((
                            add_text(line, "medium", 1.6, assets),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}

pub fn update_action_buttons(
    player: Res<Player>,
    mut commands: Commands,
    mut btn_q: Query<(
        Entity,
        &ActionButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ImageNode,
        Option<&DisabledButton>,
    )>,
) {
    for (entity, action_btn, mut bg, mut border, mut img, disabled) in &mut btn_q {
        let cost_ap = action_btn.0.ap_cost();
        let cost_gold = action_btn.0.gold_cost();
        let is_valid = player.ap >= cost_ap && player.gold >= cost_gold;

        if is_valid {
            if disabled.is_some() {
                commands.entity(entity).remove::<DisabledButton>();
                bg.0 = NORMAL_BUTTON_COLOR;
                *border = BorderColor::all(BUTTON_BORDER_COLOR);
                img.color = Color::WHITE;
            }
        } else {
            if disabled.is_none() {
                commands.entity(entity).insert(DisabledButton);
                bg.0 = DISABLED_BUTTON_COLOR;
                *border = BorderColor::all(DISABLED_BORDER_COLOR);
                img.color = Color::srgb(0.3, 0.3, 0.3);
            }
        }
    }
}
