use std::fmt::Debug;

use crate::core::assets::WorldAssets;
use bevy::prelude::*;


/// Change the background color of an entity
pub fn recolor<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(On<Pointer<E>>, Query<&mut BackgroundColor>) {
    move |ev, mut bgcolor_q| {
        if let Ok(mut bgcolor) = bgcolor_q.get_mut(ev.entity) {
            bgcolor.0 = color;
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
