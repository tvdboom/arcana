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

#[derive(Component)]
pub struct ScrollbarTrackX {
    pub container: Entity,
}

#[derive(Component)]
pub struct ScrollbarThumbX {
    pub container: Entity,
}

pub fn scroll_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut ScrollPosition, &ComputedNode, &Interaction), With<ScrollableContainer>>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll, computed, interaction) in &mut query {
            if *interaction != Interaction::None {
                // Scroll offset speed factor
                let max_scroll_y = (computed.content_size().y - computed.size().y).max(0.0);
                let max_scroll_x = (computed.content_size().x - computed.size().x).max(0.0);

                if max_scroll_x > 0.0 && max_scroll_y <= 0.0 {
                    scroll.x -= event.y * 200.0;
                    scroll.x = scroll.x.clamp(0.0, max_scroll_x);
                } else {
                    scroll.y -= event.y * 200.0;
                    scroll.y = scroll.y.clamp(0.0, max_scroll_y);
                    if max_scroll_x > 0.0 {
                        scroll.x += event.x * 200.0;
                        scroll.x = scroll.x.clamp(0.0, max_scroll_x);
                    }
                }
            }
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

        if viewport_height <= 0.0 {
            continue;
        }

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

pub fn on_scrollbar_thumb_drag_x(
    ev: On<Pointer<Drag>>,
    thumb_q: Query<&ScrollbarThumbX>,
    parent_q: Query<&ChildOf>,
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    track_q: Query<&ComputedNode, With<ScrollbarTrackX>>,
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

    let viewport_width = scroll_node.size().x;
    let content_width = scroll_node.content_size().x;
    let max_scroll = (content_width - viewport_width).max(0.0);
    if max_scroll <= 0.0 || content_width <= 0.0 {
        scroll.x = 0.0;
        return;
    }

    let track_width = track_node.size().x;
    if track_width <= 1.0 {
        return;
    }
    let min_thumb_width = 32.0_f32.min(track_width);
    let thumb_width =
        (viewport_width / content_width * track_width).clamp(min_thumb_width, track_width);
    let max_thumb_left = (track_width - thumb_width).max(1.0);
    scroll.x = (scroll.x + ev.delta.x * max_scroll / max_thumb_left).clamp(0.0, max_scroll);
}

pub fn update_scrollbar_x_system(
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    mut track_q: Query<
        (&ComputedNode, &mut Visibility, &ScrollbarTrackX),
        (Without<ScrollbarThumbX>, Without<ScrollableContainer>),
    >,
    mut thumb_q: Query<
        (&mut Node, &ScrollbarThumbX),
        (Without<ScrollbarTrackX>, Without<ScrollableContainer>),
    >,
) {
    for (track_computed, mut track_visibility, track) in &mut track_q {
        let Ok((mut scroll, scroll_node)) = scroll_q.get_mut(track.container) else {
            continue;
        };

        let viewport_width = scroll_node.size().x;
        let content_width = scroll_node.content_size().x;
        let max_scroll = (content_width - viewport_width).max(0.0);

        if viewport_width <= 0.0 {
            continue;
        }

        if max_scroll <= 1.0 || content_width <= viewport_width {
            scroll.x = 0.0;
            if *track_visibility != Visibility::Hidden {
                *track_visibility = Visibility::Hidden;
            }
            continue;
        }

        if *track_visibility != Visibility::Visible {
            *track_visibility = Visibility::Visible;
        }
        scroll.x = scroll.x.clamp(0.0, max_scroll);

        let track_width = track_computed.size().x;
        if track_width <= 1.0 {
            if *track_visibility != Visibility::Hidden {
                *track_visibility = Visibility::Hidden;
            }
            continue;
        }

        let min_thumb_width = 32.0_f32.min(track_width);
        let thumb_width =
            (viewport_width / content_width * track_width).clamp(min_thumb_width, track_width);
        let max_thumb_left = (track_width - thumb_width).max(0.0);
        let thumb_left = if max_scroll > 0.0 {
            scroll.x / max_scroll * max_thumb_left
        } else {
            0.0
        };

        for (mut thumb_node, thumb) in &mut thumb_q {
            if thumb.container == track.container {
                thumb_node.width = Val::Px(thumb_width);
                thumb_node.left = Val::Px(thumb_left);
            }
        }
    }
}
