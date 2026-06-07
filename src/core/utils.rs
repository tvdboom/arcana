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

pub fn reset_cursor(mut commands: Commands, window_e: Single<Entity, With<Window>>) {
    commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
}

/// Set cursor icon on event
pub fn cursor<T: Debug + Clone + Reflect>(
    icon: SystemCursorIcon,
) -> impl FnMut(On<Pointer<T>>, Commands, Query<&DisabledButton>, Single<Entity, With<Window>>) {
    move |ev: On<Pointer<T>>,
          mut commands: Commands,
          disabled_q: Query<&DisabledButton>,
          window_e: Single<Entity, With<Window>>| {
        if icon == SystemCursorIcon::Pointer && disabled_q.contains(ev.entity) {
            commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
        } else {
            commands.entity(*window_e).insert(CursorIcon::from(icon));
        }
    }
}
