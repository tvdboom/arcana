use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::menu::utils::{add_text, recolor};
use crate::core::utils::cursor;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

/// Spawns a styled action button (same visual style as the level-up confirm button):
/// a bordered pill with the standard button colors, gold border and gold bold label.
///
/// Hover recoloring and the pointer cursor are wired up automatically. The returned
/// [`EntityCommands`] let the caller attach a marker component and a click observer.
pub fn spawn_action_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    assets: &WorldAssets,
    label: impl Into<String>,
) -> EntityCommands<'a> {
    let mut cmd = parent.spawn((
        Node {
            align_self: AlignSelf::Center,
            padding: UiRect::axes(Val::Px(32.), Val::Px(10.)),
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON_COLOR),
        BorderColor::all(BUTTON_BORDER_COLOR),
        Button,
        Interaction::default(),
        Pickable::default(),
    ));

    cmd.observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .with_children(|parent| {
            parent.spawn((add_text(label, "bold", 1.8, assets), TextColor(BUTTON_TEXT_COLOR)));
        });

    cmd
}
