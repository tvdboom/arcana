use bevy::prelude::*;

#[derive(Component)]
pub struct ScrollableContainer;

#[derive(Component)]
pub struct ScrollbarTrack {
    pub container: Entity,
}

#[derive(Component)]
pub struct ScrollbarThumb {
    pub container: Entity,
}

pub fn scroll_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll, computed) in &mut query {
            // Scroll offset speed factor
            scroll.y -= event.y * 200.0;
            let max_scroll = (computed.content_size().y - computed.size().y).max(0.0);
            scroll.y = scroll.y.clamp(0.0, max_scroll);
        }
    }
}

pub fn on_scrollbar_thumb_drag(
    ev: On<Pointer<Drag>>,
    thumb_q: Query<&ScrollbarThumb>,
    parent_q: Query<&ChildOf>,
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    track_q: Query<&ComputedNode, With<ScrollbarTrack>>,
) {
    let Ok(thumb) = thumb_q.get(ev.entity) else {
        return;
    };
    let Ok((mut scroll, scroll_node)) = scroll_q.get_mut(thumb.container) else {
        return;
    };
    let Ok(parent) = parent_q.get(ev.entity) else {
        return;
    };
    let Ok(track_node) = track_q.get(parent.0) else {
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

pub fn update_scrollbar_system(
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    mut track_q: Query<
        (&ComputedNode, &mut Visibility, &ScrollbarTrack),
        (Without<ScrollbarThumb>, Without<ScrollableContainer>),
    >,
    mut thumb_q: Query<
        (&mut Node, &ScrollbarThumb),
        (Without<ScrollbarTrack>, Without<ScrollableContainer>),
    >,
) {
    for (track_computed, mut track_visibility, track) in &mut track_q {
        let Ok((mut scroll, scroll_node)) = scroll_q.get_mut(track.container) else {
            continue;
        };

        let viewport_height = scroll_node.size().y;
        let content_height = scroll_node.content_size().y;
        let max_scroll = (content_height - viewport_height).max(0.0);

        if max_scroll <= 1.0 || content_height <= viewport_height {
            scroll.y = 0.0;
            if *track_visibility != Visibility::Hidden {
                *track_visibility = Visibility::Hidden;
            }
            continue;
        }

        if *track_visibility != Visibility::Visible {
            *track_visibility = Visibility::Visible;
        }
        scroll.y = scroll.y.clamp(0.0, max_scroll);

        let track_height = track_computed.size().y;
        if track_height <= 1.0 {
            if *track_visibility != Visibility::Hidden {
                *track_visibility = Visibility::Hidden;
            }
            continue;
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

        for (mut thumb_node, thumb) in &mut thumb_q {
            if thumb.container == track.container {
                thumb_node.height = Val::Px(thumb_height);
                thumb_node.top = Val::Px(thumb_top);
            }
        }
    }
}
