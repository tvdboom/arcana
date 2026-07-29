//! Responsive layout rules shared by desktop, web, tablet, and phone screens.

use std::collections::HashMap;

use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::core::combat::ui::{CombatColumns, CombatSidePanel};
use crate::core::ui::scrollbar::{HorizontalWheelScroll, ScrollableContainer};
use crate::core::ui::utils::{
    PanelCmp, PlayScreenColumns2And3, PlayScreenColumnsContainer, PlayingActionBar,
    PlayingContentFrame, PlayingPrimaryColumn,
};

/// Width below which the interface switches from three columns to a vertical flow.
const PORTRAIT_BREAKPOINT: f32 = 720.0;

/// Returns whether the viewport needs the vertically flowing phone layout.
fn uses_portrait_layout(width: f32, height: f32) -> bool {
    width < PORTRAIT_BREAKPOINT || height > width * 1.15
}

/// Applies phone-friendly vertical layouts while retaining the desktop layout on wide screens.
pub fn update_responsive_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<
        (
            &mut Node,
            Option<&PlayingContentFrame>,
            Option<&PlayScreenColumnsContainer>,
            Option<&PlayingPrimaryColumn>,
            Option<&PlayScreenColumns2And3>,
            Option<&PlayingActionBar>,
            Option<&PanelCmp>,
            Option<&CombatColumns>,
            Option<&CombatSidePanel>,
        ),
        Or<(
            With<PlayingContentFrame>,
            With<PlayScreenColumnsContainer>,
            With<PlayingPrimaryColumn>,
            With<PlayScreenColumns2And3>,
            With<PlayingActionBar>,
            With<PanelCmp>,
            With<CombatColumns>,
            With<CombatSidePanel>,
        )>,
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());

    for (
        mut node,
        frame,
        columns,
        primary,
        secondary,
        action,
        panel,
        combat_columns,
        combat_panel,
    ) in &mut nodes
    {
        if frame.is_some() {
            node.width = if portrait {
                percent(100.)
            } else {
                Val::Auto
            };
            node.height = percent(100.);
            node.aspect_ratio = if portrait {
                None
            } else {
                Some(16. / 9.)
            };
            node.overflow = if portrait {
                Overflow::scroll_y()
            } else {
                Overflow::visible()
            };
        }
        if columns.is_some() {
            node.height = if portrait {
                Val::Auto
            } else {
                percent(66.)
            };
            node.flex_direction = if portrait {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            };
            node.padding = UiRect::horizontal(if portrait {
                Val::Px(8.)
            } else {
                Val::Px(26.)
            });
        }
        if primary.is_some() {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(33.5)
            };
            node.min_height = if portrait {
                Val::VMin(100.)
            } else {
                Val::Auto
            };
        }
        if secondary.is_some() {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(32.)
            };
            node.min_height = if portrait {
                Val::VMin(90.)
            } else {
                Val::Auto
            };
        }
        if action.is_some() {
            node.height = if portrait {
                Val::Auto
            } else {
                Val::VMin(14.5)
            };
            node.min_height = if portrait {
                Val::Px(64.)
            } else {
                Val::Auto
            };
            node.flex_wrap = if portrait {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            };
            node.padding.top = if portrait {
                Val::Px(12.)
            } else {
                Val::VMin(6.5)
            };
            node.padding.bottom = if portrait {
                Val::Px(24.)
            } else {
                Val::Px(0.)
            };
        }
        if panel.is_some() {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(66.5)
            };
            node.min_height = if portrait {
                Val::VMin(100.)
            } else {
                Val::Auto
            };
        }
        if combat_columns.is_some() {
            node.height = if portrait {
                Val::Auto
            } else {
                percent(100.)
            };
            node.flex_direction = if portrait {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            };
            node.overflow = if portrait {
                Overflow::scroll_y()
            } else {
                Overflow::visible()
            };
        }
        if let Some(combat_panel) = combat_panel {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(combat_panel.desktop_width)
            };
            node.height = if portrait {
                Val::VMin(150.)
            } else {
                percent(100.)
            };
        }
    }
}

/// Converts one-finger drags over a scrollable area into vertical or horizontal scrolling.
pub fn touch_scroll_system(
    mut touch_events: MessageReader<TouchInput>,
    mut last_positions: Local<HashMap<u64, Vec2>>,
    mut scroll_q: Query<
        (&mut ScrollPosition, &ComputedNode, &Interaction, Option<&HorizontalWheelScroll>),
        With<ScrollableContainer>,
    >,
) {
    for touch in touch_events.read() {
        match touch.phase {
            TouchPhase::Started => {
                last_positions.insert(touch.id, touch.position);
            },
            TouchPhase::Moved => {
                let Some(previous) = last_positions.insert(touch.id, touch.position) else {
                    continue;
                };
                let delta = touch.position - previous;
                for (mut scroll, computed, interaction, horizontal) in &mut scroll_q {
                    if *interaction == Interaction::None {
                        continue;
                    }
                    let max_x = (computed.content_size().x - computed.size().x).max(0.0);
                    let max_y = (computed.content_size().y - computed.size().y).max(0.0);
                    if horizontal.is_some() || (max_x > 0.0 && max_y <= 0.0) {
                        scroll.x = (scroll.x - delta.x).clamp(0.0, max_x);
                    } else {
                        scroll.y = (scroll.y - delta.y).clamp(0.0, max_y);
                    }
                }
            },
            TouchPhase::Ended | TouchPhase::Canceled => {
                last_positions.remove(&touch.id);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Selects compact flow for phones while retaining the desktop layout.
    fn viewport_classification_matches_phone_and_desktop() {
        assert!(uses_portrait_layout(390.0, 844.0));
        assert!(!uses_portrait_layout(1280.0, 720.0));
    }
}
