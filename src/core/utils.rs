use crate::core::menu::buttons::DisabledButton;
use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};
use std::fmt::Debug;

/// Generic system that despawns all entities with a specific component
pub fn despawn<T: Component>(mut commands: Commands, query_c: Query<Entity, With<T>>) {
    for entity in &query_c {
        commands.entity(entity).try_despawn();
    }
}

pub fn reset_cursor(
    mut commands: Commands,
    window_e: Single<Entity, With<Window>>,
    hovered_q: Query<&Interaction>,
) {
    let mut any_hovered = false;
    for interaction in &hovered_q {
        if *interaction == Interaction::Hovered {
            any_hovered = true;
            break;
        }
    }
    if !any_hovered {
        commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
    }
}

/// Set cursor icon on event
pub fn cursor<T: Debug + Clone + Reflect>(
    icon: SystemCursorIcon,
) -> impl FnMut(
    On<Pointer<T>>,
    Commands,
    Query<&DisabledButton>,
    Single<Entity, With<Window>>,
    Query<Entity, With<crate::core::ui::playing::PrecombatDragGhost>>,
) {
    move |ev: On<Pointer<T>>,
          mut commands: Commands,
          disabled_q: Query<&DisabledButton>,
          window_e: Single<Entity, With<Window>>,
          ghost_q: Query<Entity, With<crate::core::ui::playing::PrecombatDragGhost>>| {
        if !ghost_q.is_empty() {
            return;
        }
        if icon == SystemCursorIcon::Pointer && disabled_q.contains(ev.entity) {
            commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
        } else {
            commands.entity(*window_e).insert(CursorIcon::from(icon));
        }
    }
}
