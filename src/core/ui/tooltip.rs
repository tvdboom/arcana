use crate::core::assets::WorldAssets;
use crate::core::constants::{BUTTON_BORDER_COLOR, BUTTON_TEXT_COLOR, PLACEHOLDER_COLOR};
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_text, spawn_rich_text_row};
use crate::core::player::Player;
use crate::core::settings::Language;
use crate::utils::{capitalize_words, NameFromEnum};
use bevy::prelude::*;

#[derive(Component)]
pub struct TooltipNode {
    pub width: f32,
    pub height: f32,
}

pub enum TooltipBadge {
    Price(u32),
    ActionPoints(u32),
}

pub struct PetStat {
    pub label: String,
    pub image_key: String,
    pub value: u32,
}

pub struct TooltipContent {
    pub title: String,
    pub lines: Vec<String>,
    pub badge: Option<TooltipBadge>,
    pub pet_stats: Option<Vec<PetStat>>,
    pub image: Option<String>,
}

pub fn spawn_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    content: TooltipContent,
    windows: &Query<&Window>,
) {
    let (window_width, window_height, cursor) = if let Ok(window) = windows.single() {
        (window.width(), window.height(), window.cursor_position())
    } else {
        (1600., 900., None)
    };

    let max_allowed_width = window_width * 0.35;

    let font_size_title = window_height * 0.024;
    let font_size_desc = window_height * 0.018;
    let char_width_desc = font_size_desc * 0.60;
    let char_width_desc_for_width = font_size_desc * 0.68;
    let line_height_title = font_size_title * 1.35;
    let line_height_desc = font_size_desc * 1.35;

    // Total padding + safety margins = 48.0 px
    let padding_width = 48.0_f32;

    let mut text_allowed_width = max_allowed_width;
    if content.image.is_some() {
        text_allowed_width -= 144.0 + 16.0;
    }

    let max_chars_per_line =
        ((text_allowed_width - padding_width) / char_width_desc).floor().max(15.0) as usize;

    // Wrap the description lines
    let mut wrapped_lines = Vec::new();
    for line in &content.lines {
        for sub_line in line.split('\n') {
            if let Some(rest) = sub_line.strip_prefix("• ") {
                let wrapped = wrap_tooltip_line(rest, max_chars_per_line.saturating_sub(3));
                for (i, wl) in wrapped.iter().enumerate() {
                    if i == 0 {
                        wrapped_lines.push(format!("• {}", wl));
                    } else {
                        wrapped_lines.push(format!("  {}", wl));
                    }
                }
            } else {
                wrapped_lines.extend(wrap_tooltip_line(sub_line, max_chars_per_line));
            }
        }
    }

    // Estimate width of content
    let desc_max_chars =
        wrapped_lines.iter().map(|line| visual_chars_count(line)).max().unwrap_or(0) as f32;
    let mut desc_width = desc_max_chars * char_width_desc_for_width;
    if content.image.is_some() {
        desc_width += 144.0 + 16.0;
    }

    let char_width_title_for_width = font_size_title * 0.65;
    let title_chars_width = content.title.chars().count() as f32 * char_width_title_for_width;
    let badge_width = if let Some(ref badge) = content.badge {
        match badge {
            TooltipBadge::Price(val) => {
                32.0 + 12.0
                    + (format!("{}", val).chars().count() as f32) * char_width_desc_for_width
            },
            TooltipBadge::ActionPoints(val) => {
                32.0 + 12.0
                    + (format!("{}", val).chars().count() as f32) * char_width_desc_for_width
            },
        }
    } else {
        0.0
    };
    let title_row_width = if badge_width > 0.0 {
        title_chars_width + badge_width + 24.0
    } else {
        title_chars_width
    };

    let content_width = desc_width.max(title_row_width) * 1.15;
    let tooltip_width =
        (content_width + padding_width).clamp(320.0_f32.min(max_allowed_width), max_allowed_width);

    // Calculate height
    let mut tooltip_height =
        line_height_title + (wrapped_lines.len() as f32) * line_height_desc + 36.0;
    if content.image.is_some() {
        tooltip_height = tooltip_height.max(144.0 + 36.0);
    }
    if content.pet_stats.is_some() {
        let stat_box_height = tooltip_width * 0.32;
        tooltip_height += stat_box_height + 12.0;
    }

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
            GlobalZIndex(1000),
            TooltipNode {
                width: tooltip_width,
                height: tooltip_height,
            },
        ))
        .with_children(|parent| {
            // Badge display at top-right corner (if provided)
            if let Some(badge) = &content.badge {
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
                        match badge {
                            TooltipBadge::Price(price_value) => {
                                // Gold icon
                                parent.spawn((
                                    Node {
                                        width: Val::Px(32.),
                                        height: Val::Px(32.),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image("gold"))
                                        .with_mode(NodeImageMode::Stretch),
                                ));

                                // Price number
                                parent.spawn((
                                    add_text(format!("{}", price_value), "bold", 2.6, assets),
                                    TextColor(BUTTON_TEXT_COLOR),
                                ));
                            },
                            TooltipBadge::ActionPoints(ap_cost) => {
                                // AP icon (larger!)
                                parent.spawn((
                                    Node {
                                        width: Val::Px(32.),
                                        height: Val::Px(32.),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image("ap"))
                                        .with_mode(NodeImageMode::Stretch),
                                ));

                                // AP cost number
                                parent.spawn((
                                    add_text(format!("{}", ap_cost), "bold", 2.6, assets),
                                    TextColor(BUTTON_TEXT_COLOR),
                                ));
                            },
                        }
                    });
            }

            // Content display helper
            let render_content = |parent: &mut ChildSpawnerCommands| {
                // Title
                parent.spawn((
                    add_text(content.title.clone(), "bold", 2.4, assets),
                    TextColor(BUTTON_TEXT_COLOR),
                ));

                // Description
                if !wrapped_lines.is_empty() {
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for line in &wrapped_lines {
                                if line.starts_with("• ") || line.starts_with("  ") {
                                    parent
                                        .spawn(Node {
                                            padding: UiRect::left(Val::Px(32.)),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            spawn_rich_text_row(
                                                parent,
                                                assets,
                                                line,
                                                1.8,
                                                "medium",
                                                Color::WHITE,
                                            );
                                        });
                                } else {
                                    spawn_rich_text_row(
                                        parent,
                                        assets,
                                        line,
                                        1.8,
                                        "medium",
                                        Color::WHITE,
                                    );
                                }
                            }
                        });
                }
            };

            if let Some(ref image_path) = content.image {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(16.),
                        align_items: AlignItems::FlexStart,
                        width: Val::Percent(100.),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Left: Image (50% larger!)
                        parent
                            .spawn((
                                Node {
                                    width: Val::Px(144.),
                                    height: Val::Px(144.),
                                    flex_shrink: 0.,
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                BorderColor::all(BUTTON_BORDER_COLOR),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Node {
                                        width: Val::Percent(100.),
                                        height: Val::Percent(100.),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image(format!("build_{}", image_path)))
                                        .with_mode(NodeImageMode::Stretch),
                                ));
                            });

                        // Right: Column of content
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.),
                                width: Val::Percent(100.),
                                ..default()
                            })
                            .with_children(|parent| {
                                render_content(parent);
                            });
                    });
            } else {
                render_content(parent);
            }

            // Pet stats if provided
            if let Some(stats) = &content.pet_stats {
                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        margin: UiRect::top(Val::Px(4.)),
                        ..default()
                    })
                    .with_children(|parent| {
                        for stat in stats {
                            spawn_pet_stat_box(
                                parent,
                                assets,
                                stat.label.clone(),
                                &stat.image_key,
                                stat.value,
                            );
                        }
                    });
            }
        });
}

pub fn spawn_pet_tooltip(
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
        .get_opt(&format!("pet.{}_desc", pet.kind.to_lowername()), lang)
        .unwrap_or_else(|| format!("A loyal {} companion.", pet_type_name.to_lowercase()));

    let content = TooltipContent {
        title,
        lines: vec![desc],
        badge: None,
        pet_stats: Some(vec![
            PetStat {
                label: localization.get("attack", lang),
                image_key: "attack".to_string(),
                value: pet.attack,
            },
            PetStat {
                label: localization.get("defense", lang),
                image_key: "defense".to_string(),
                value: pet.defense,
            },
            PetStat {
                label: localization.get("initiative", lang),
                image_key: "initiative".to_string(),
                value: pet.initiative,
            },
        ]),
        image: None,
    };

    spawn_tooltip(commands, assets, content, windows);
}

pub fn spawn_action_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    action_name: String,
    ap_cost: u32,
    desc: String,
    windows: &Query<&Window>,
) {
    let content = TooltipContent {
        title: action_name,
        lines: vec![desc],
        badge: Some(TooltipBadge::ActionPoints(ap_cost)),
        pet_stats: None,
        image: None,
    };

    spawn_tooltip(commands, assets, content, windows);
}

pub fn spawn_item_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    title: String,
    lines: Vec<String>,
    windows: &Query<&Window>,
    price: Option<u32>,
    image: Option<String>,
) {
    let content = TooltipContent {
        title,
        lines,
        badge: price.map(TooltipBadge::Price),
        pet_stats: None,
        image,
    };

    spawn_tooltip(commands, assets, content, windows);
}

pub fn visual_chars_count(s: &str) -> usize {
    let mut count = 0;
    let mut in_brackets = false;
    for c in s.chars() {
        if c == '[' {
            in_brackets = true;
            count += 1;
        } else if c == ']' {
            in_brackets = false;
        } else if !in_brackets {
            count += 1;
        }
    }
    count
}

pub fn wrap_tooltip_line(line: &str, max_chars: usize) -> Vec<String> {
    if visual_chars_count(line) <= max_chars {
        return vec![line.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        let current_visual = visual_chars_count(&current);
        let word_visual = visual_chars_count(word);
        let next_len = current_visual
            + if current.is_empty() {
                0
            } else {
                1
            }
            + word_visual;
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

pub fn place_tooltip(
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

fn spawn_pet_stat_box(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    label: String,
    image_key: &str,
    value: u32,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(32.),
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
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ImageNode {
                    image: assets.image(image_key),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(1., 1., 1., 0.3),
                    ..default()
                },
            ));
            parent.spawn((add_text(label, "medium", 1.6, assets), TextColor(BUTTON_TEXT_COLOR)));
            parent
                .spawn((add_text(value.to_string(), "bold", 3.0, assets), TextColor(Color::WHITE)));
        });
}
