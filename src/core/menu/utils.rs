use crate::core::menu::buttons::DisabledButton;
use std::fmt::Debug;

use crate::core::assets::WorldAssets;
use bevy::prelude::*;

/// Change the background color of an entity
pub fn recolor<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(On<Pointer<E>>, Query<(&mut BackgroundColor, Option<&DisabledButton>)>) {
    move |ev, mut bgcolor_q| {
        if let Ok((mut bgcolor, disabled)) = bgcolor_q.get_mut(ev.entity) {
            if disabled.is_none() {
                bgcolor.0 = color;
            }
        };
    }
}

/// Add a root UI node that covers the whole screen
pub fn add_root_node(block: bool) -> (Node, Pickable) {
    (
        Node {
            width: Val::Vw(100.),
            height: Val::Vh(100.),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        if block {
            Pickable {
                should_block_lower: true,
                is_hoverable: false,
            }
        } else {
            Pickable::IGNORE
        },
    )
}

/// Add a standard text component
pub fn add_text(
    text: impl Into<String>,
    font: &str,
    font_size: f32,
    assets: &WorldAssets,
) -> (Text, TextFont) {
    (
        Text::new(text),
        TextFont {
            font: FontSource::Handle(assets.font(font)),
            font_size: FontSize::Vh(font_size),
            ..default()
        },
    )
}

/// Change the image of an entity on event
pub fn reimage<E: Debug + Clone + Reflect>(
    image_handle: Handle<Image>,
) -> impl Fn(On<Pointer<E>>, Query<&mut ImageNode>) {
    move |ev, mut image_q| {
        if let Ok(mut image_node) = image_q.get_mut(ev.entity) {
            image_node.image = image_handle.clone();
            image_node.image_mode = NodeImageMode::Stretch;
        };
    }
}

pub fn spawn_rich_text_row(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    line: impl Into<String>,
    font_size: f32,
    font: &str,
    color: Color,
) {
    let line_str = line.into();
    if !line_str.contains('[') || !line_str.contains(']') {
        parent.spawn((add_text(line_str, font, font_size, assets), TextColor(color)));
        return;
    }

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(2.),
            overflow: Overflow::clip(),
            ..default()
        })
        .with_children(|parent| {
            let mut remaining = &line_str[..];
            while let Some(start_idx) = remaining.find('[') {
                if let Some(end_idx) = remaining[start_idx..].find(']') {
                    let actual_end = start_idx + end_idx;
                    let before = &remaining[..start_idx];
                    if !before.is_empty() {
                        parent.spawn((add_text(before, font, font_size, assets), TextColor(color)));
                    }

                    parent.spawn((
                        Node {
                            width: Val::Vh(font_size * 1.35),
                            height: Val::Vh(font_size * 1.35),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        ImageNode::new(assets.image(&remaining[start_idx + 1..actual_end]))
                            .with_mode(NodeImageMode::Stretch),
                    ));

                    remaining = &remaining[actual_end + 1..];
                } else {
                    break;
                }
            }

            if !remaining.is_empty() {
                parent.spawn((add_text(remaining, font, font_size, assets), TextColor(color)));
            }
        });
}
