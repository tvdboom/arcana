//! Responsive layout rules shared by desktop, web, tablet, and phone screens.

use std::collections::HashMap;

use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::core::actions::craft::{CraftColumn, CraftColumns};
use crate::core::actions::shop::{
    ShopFiltersPanel, ShopHeader, ShopHeaderGold, ShopItemCard, ShopItemRow, ShopItemsScroll,
    ShopScrollContainer, ShopTabs,
};
use crate::core::combat::ui::{
    CombatColumns, CombatConsumablesRow, CombatSidePanel, COMBAT_CONSUMABLE_CARD_GAP,
};
use crate::core::menu::utils::ResponsiveText;
use crate::core::states::{is_panel_state, GameState};
use crate::core::ui::creation::{CreationLayoutNode, SelectionGestureState};
use crate::core::ui::level_up::{
    LevelUpAbilityChoiceBtn, LevelUpAttrMinusBtn, LevelUpAttrPlusBtn, LevelUpLayoutNode,
    LevelUpPerkChoiceBtn,
};
use crate::core::ui::scrollbar::{HorizontalWheelScroll, ScrollableContainer};
use crate::core::ui::utils::{
    PanelCard, PanelCardCostBadge, PanelCardCostIcon, PanelCardCostRow, PanelCardRow, PanelCmp,
    PanelHeader, PanelIntensitySlider, PanelResources, PanelTitle, PlayScreenColumns2And3,
    PlayScreenColumnsContainer, PlayingActionBar, PlayingContentFrame, PlayingPrimaryColumn,
    ResponsiveOverlayCard, ResponsiveProgressBar, ResponsiveSettingsPanel, ResponsiveSquare,
    ResponsiveWidth, SLIDER_WIDTH,
};

/// Width below which the interface switches from three columns to a vertical flow.
const PORTRAIT_BREAKPOINT: f32 = 720.0;

/// Returns whether the viewport needs the vertically flowing phone layout.
fn uses_portrait_layout(width: f32, height: f32) -> bool {
    width < PORTRAIT_BREAKPOINT || height > width * 1.15
}

/// Resolves a design text size to the pixels used by the responsive typography system.
pub fn responsive_font_size(width: f32, height: f32, design_size: f32) -> f32 {
    if uses_portrait_layout(width, height) {
        let phone_scale = (width / 390.0).clamp(0.9, 1.15);
        (design_size * 5.0 * phone_scale).clamp(11.0, 26.0)
    } else {
        width.min(height) * design_size / 100.0
    }
}

/// Returns a bounded action-card size that leaves surrounding phone UI visible.
fn phone_panel_card_size(width: f32, height: f32) -> Vec2 {
    Vec2::new((width * 0.78).clamp(250.0, 340.0), (height * 0.56).clamp(340.0, 500.0))
}

/// Applies phone-friendly vertical layouts while retaining the desktop layout on wide screens.
pub fn update_responsive_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    game_state: Res<State<GameState>>,
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
            Option<&CombatConsumablesRow>,
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
            With<CombatConsumablesRow>,
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
        combat_consumables,
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
            node.display = if portrait && is_panel_state(*game_state.get()) {
                Display::None
            } else {
                Display::Flex
            };
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
                Val::Vh(100.)
            } else {
                Val::Auto
            };
            node.height = if portrait {
                Val::Vh(100.)
            } else {
                percent(100.)
            };
            node.overflow = if portrait {
                Overflow::scroll_y()
            } else {
                Overflow::visible()
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
        if combat_consumables.is_some() {
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::Center
            };
            node.column_gap = Val::Px(if portrait {
                4.
            } else {
                COMBAT_CONSUMABLE_CARD_GAP
            });
            node.overflow = if portrait {
                Overflow::scroll_x()
            } else {
                Overflow::visible()
            };
        }
    }
}

/// Keeps viewport-relative desktop typography readable on narrow phone screens.
pub fn update_responsive_typography(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut text_q: Query<(&mut TextFont, &ResponsiveText)>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());

    for (mut font, responsive) in &mut text_q {
        font.font_size = if portrait {
            FontSize::Px(responsive_font_size(window.width(), window.height(), responsive.0))
        } else {
            FontSize::VMin(responsive.0)
        };
    }
}

/// Keeps bars, icons, and card cost badges proportional to the phone viewport.
pub fn update_responsive_element_sizes(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<
        (
            &mut Node,
            Option<&ResponsiveProgressBar>,
            Option<&ResponsiveSquare>,
            Option<&PanelCardCostRow>,
            Option<&PanelCardCostBadge>,
            Option<&PanelCardCostIcon>,
            Option<&ResponsiveWidth>,
        ),
        Or<(
            With<ResponsiveProgressBar>,
            With<ResponsiveSquare>,
            With<PanelCardCostRow>,
            With<PanelCardCostBadge>,
            With<PanelCardCostIcon>,
            With<ResponsiveWidth>,
        )>,
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());

    for (mut node, progress, square, cost_row, cost_badge, cost_icon, responsive_width) in
        &mut nodes
    {
        if let Some(progress) = progress {
            node.width = if portrait {
                progress.phone_width
            } else {
                progress.desktop_width
            };
            node.height = if portrait {
                progress.phone_height
            } else {
                progress.desktop_height
            };
        }
        if let Some(square) = square {
            let size = if portrait {
                Val::Px(square.phone_size)
            } else {
                square.desktop_size
            };
            node.width = size;
            node.height = size;
        }
        if let Some(responsive_width) = responsive_width {
            node.width = if portrait {
                responsive_width.phone_width
            } else {
                responsive_width.desktop_width
            };
        }
        if cost_row.is_some() {
            node.top = Val::Px(if portrait {
                8.
            } else {
                10.
            });
            node.right = if portrait {
                Val::Px(8.)
            } else {
                Val::Vw(1.9)
            };
            node.column_gap = Val::Px(if portrait {
                4.
            } else {
                6.
            });
            node.max_width = if portrait {
                Val::Px((window.width() - 32.).max(0.))
            } else {
                Val::Auto
            };
        }
        if cost_badge.is_some() {
            node.column_gap = Val::Px(if portrait {
                3.
            } else {
                4.
            });
            node.padding = if portrait {
                UiRect::axes(Val::Px(6.), Val::Px(3.))
            } else {
                UiRect::axes(Val::Px(8.), Val::Px(4.))
            };
            node.border_radius = BorderRadius::all(Val::Px(if portrait {
                5.
            } else {
                6.
            }));
        }
        if cost_icon.is_some() {
            let size = Val::Px(if portrait {
                16.
            } else {
                20.
            });
            node.width = size;
            node.height = size;
        }
    }
}

/// Opens action panels at their header instead of retaining the gameplay page scroll offset.
pub fn reset_playing_scroll_position(
    mut frame_q: Query<&mut ScrollPosition, With<PlayingContentFrame>>,
) {
    for mut scroll in &mut frame_q {
        scroll.0 = Vec2::ZERO;
    }
}

/// Reflows action-panel headers and turns their cards into touch-sized carousels.
pub fn update_panel_responsive_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<
        (
            &mut Node,
            Option<&PanelHeader>,
            Option<&PanelTitle>,
            Option<&PanelResources>,
            Option<&PanelIntensitySlider>,
            Option<&PanelCardRow>,
            Option<&PanelCard>,
            Option<&ResponsiveSettingsPanel>,
            Option<&ResponsiveOverlayCard>,
        ),
        Or<(
            With<PanelHeader>,
            With<PanelTitle>,
            With<PanelResources>,
            With<PanelIntensitySlider>,
            With<PanelCardRow>,
            With<PanelCard>,
            With<ResponsiveSettingsPanel>,
            With<ResponsiveOverlayCard>,
        )>,
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());
    let phone_card_size = phone_panel_card_size(window.width(), window.height());

    for (mut node, header, title, resources, slider, card_row, card, settings, overlay) in
        &mut nodes
    {
        if let Some(header) = header {
            node.width = percent(100.);
            node.height = if portrait {
                Val::Auto
            } else {
                Val::Px(75.)
            };
            node.min_height = if portrait {
                Val::Px(if header.has_slider {
                    138.
                } else {
                    82.
                })
            } else {
                Val::Auto
            };
            node.flex_direction = if portrait {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            };
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::SpaceBetween
            };
            node.align_items = AlignItems::Center;
            node.row_gap = if portrait {
                Val::Px(4.)
            } else {
                Val::Px(0.)
            };
        }
        if title.is_some() {
            node.position_type = if portrait {
                PositionType::Relative
            } else {
                PositionType::Absolute
            };
            node.left = if portrait {
                Val::Auto
            } else {
                Val::Px(30.)
            };
        }
        if resources.is_some() {
            node.position_type = if portrait {
                PositionType::Relative
            } else {
                PositionType::Absolute
            };
            node.right = if portrait {
                Val::Auto
            } else {
                Val::Px(30.)
            };
        }
        if slider.is_some() {
            node.width = Val::Px(SLIDER_WIDTH);
            node.height = Val::Px(76.);
            node.flex_shrink = 0.;
        }
        if card_row.is_some() {
            node.width = percent(100.);
            node.height = if portrait {
                Val::Px(phone_card_size.y + 20.)
            } else {
                percent(78.)
            };
            node.flex_direction = FlexDirection::Row;
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::Center
            };
            node.align_items = AlignItems::Center;
            node.column_gap = if portrait {
                Val::Px(10.)
            } else {
                Val::Px(20.)
            };
            node.margin.top = if portrait {
                Val::Px(6.)
            } else {
                Val::Px(15.)
            };
            node.overflow = if portrait {
                Overflow::scroll_x()
            } else {
                Overflow::visible()
            };
        }
        if card.is_some() {
            node.width = if portrait {
                Val::Px(phone_card_size.x)
            } else {
                percent(30.)
            };
            node.height = if portrait {
                Val::Px(phone_card_size.y)
            } else {
                percent(98.)
            };
            node.margin = UiRect::horizontal(if portrait {
                Val::Px(8.)
            } else {
                percent(1.)
            });
            node.flex_shrink = if portrait {
                0.
            } else {
                1.
            };
        }
        if let Some(settings) = settings {
            node.width = if portrait {
                percent(92.)
            } else {
                settings.desktop_width
            };
            node.height = if portrait {
                Val::Auto
            } else {
                settings.desktop_height
            };
            node.max_width = if portrait {
                Val::Auto
            } else {
                settings.desktop_max_width
            };
            node.min_height = if portrait {
                Val::Px(500.)
            } else {
                Val::Auto
            };
        }
        if let Some(overlay) = overlay {
            node.width = if portrait {
                percent(90.)
            } else {
                overlay.desktop_width
            };
            node.height = if portrait {
                Val::Auto
            } else {
                overlay.desktop_height
            };
            node.min_height = if portrait {
                Val::Px(260.)
            } else {
                Val::Auto
            };
        }
    }
}

/// Turns the three crafting workspaces into a readable horizontal carousel on phones.
pub fn update_craft_responsive_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<
        (&mut Node, Option<&CraftColumns>, Option<&CraftColumn>),
        Or<(With<CraftColumns>, With<CraftColumn>)>,
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());

    for (mut node, columns, column) in &mut nodes {
        if columns.is_some() {
            node.width = percent(100.);
            node.height = percent(82.);
            node.flex_direction = FlexDirection::Row;
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::SpaceBetween
            };
            node.align_items = AlignItems::Stretch;
            node.column_gap = Val::Px(10.);
            node.overflow = if portrait {
                Overflow::scroll_x()
            } else {
                Overflow::clip()
            };
        }
        if let Some(column) = column {
            node.width = percent(if portrait {
                82.
            } else {
                column.0
            });
            node.height = percent(100.);
            node.flex_shrink = if portrait {
                0.
            } else {
                1.
            };
            node.margin = if portrait {
                UiRect::horizontal(percent(1.))
            } else {
                UiRect::ZERO
            };
        }
    }
}

/// Stacks level-up content vertically and keeps every control reachable on phones.
pub fn update_level_up_responsive_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut layout_nodes: Query<(&mut Node, &LevelUpLayoutNode)>,
    mut attr_buttons: Query<
        &mut Node,
        (Or<(With<LevelUpAttrPlusBtn>, With<LevelUpAttrMinusBtn>)>, Without<LevelUpLayoutNode>),
    >,
    mut choice_buttons: Query<
        &mut Node,
        (
            Or<(With<LevelUpAbilityChoiceBtn>, With<LevelUpPerkChoiceBtn>)>,
            Without<LevelUpLayoutNode>,
            Without<LevelUpAttrPlusBtn>,
            Without<LevelUpAttrMinusBtn>,
        ),
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());

    for (mut node, layout) in &mut layout_nodes {
        match layout {
            LevelUpLayoutNode::Overlay => {
                node.width = Val::Vw(100.);
                node.height = Val::Vh(100.);
            },
            LevelUpLayoutNode::Panel => {
                node.width = if portrait {
                    percent(96.)
                } else {
                    Val::Vw(88.)
                };
                node.height = if portrait {
                    percent(96.)
                } else {
                    Val::VMin(100.)
                };
                node.padding = if portrait {
                    UiRect {
                        left: Val::Px(24.),
                        right: Val::Px(24.),
                        top: Val::Px(24.),
                        bottom: Val::Px(28.),
                    }
                } else {
                    UiRect {
                        left: Val::Px(84.),
                        right: Val::Px(84.),
                        top: Val::Px(64.),
                        bottom: Val::Px(76.),
                    }
                };
                node.overflow = if portrait {
                    Overflow::scroll_y()
                } else {
                    Overflow::visible()
                };
            },
            LevelUpLayoutNode::Content => {
                node.flex_grow = if portrait {
                    0.
                } else {
                    1.
                };
            },
            LevelUpLayoutNode::Header => {
                node.margin = if portrait {
                    UiRect {
                        top: Val::Px(16.),
                        bottom: Val::Px(12.),
                        ..default()
                    }
                } else {
                    UiRect {
                        top: Val::Px(64.),
                        bottom: Val::Px(16.),
                        ..default()
                    }
                };
            },
            LevelUpLayoutNode::Columns => {
                node.flex_direction = if portrait {
                    FlexDirection::Column
                } else {
                    FlexDirection::Row
                };
                node.justify_content = if portrait {
                    JustifyContent::FlexStart
                } else {
                    JustifyContent::Center
                };
                node.flex_grow = if portrait {
                    0.
                } else {
                    1.
                };
                node.column_gap = if portrait {
                    Val::Px(0.)
                } else {
                    Val::Px(32.)
                };
                node.row_gap = if portrait {
                    Val::Px(16.)
                } else {
                    Val::Px(0.)
                };
            },
            LevelUpLayoutNode::Attributes => {
                node.width = if portrait {
                    percent(100.)
                } else {
                    percent(32.)
                };
            },
            LevelUpLayoutNode::Choices => {
                node.width = if portrait {
                    percent(100.)
                } else {
                    percent(42.)
                };
            },
            LevelUpLayoutNode::Footer => {
                node.height = if portrait {
                    Val::Px(80.)
                } else {
                    Val::Px(70.)
                };
                node.margin = UiRect::bottom(if portrait {
                    Val::Px(12.)
                } else {
                    Val::Px(48.)
                });
                node.flex_shrink = 0.;
            },
        }
    }

    for mut node in &mut attr_buttons {
        node.width = Val::Px(if portrait {
            40.
        } else {
            20.
        });
        node.height = Val::Px(if portrait {
            40.
        } else {
            20.
        });
    }
    for mut node in &mut choice_buttons {
        node.min_height = if portrait {
            Val::Px(72.)
        } else {
            Val::Auto
        };
    }
}

/// Reflows shop categories, filters, and item cards for narrow touch screens.
pub fn update_shop_responsive_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<
        (
            &mut Node,
            Option<&ShopHeader>,
            Option<&ShopTabs>,
            Option<&ShopHeaderGold>,
            Option<&ShopScrollContainer>,
            Option<&ShopItemsScroll>,
            Option<&ShopFiltersPanel>,
            Option<&ShopItemRow>,
            Option<&ShopItemCard>,
        ),
        Or<(
            With<ShopHeader>,
            With<ShopTabs>,
            With<ShopHeaderGold>,
            With<ShopScrollContainer>,
            With<ShopItemsScroll>,
            With<ShopFiltersPanel>,
            With<ShopItemRow>,
            With<ShopItemCard>,
        )>,
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());

    for (mut node, header, tabs, gold, container, items, filters, row, card) in &mut nodes {
        if header.is_some() {
            node.flex_direction = if portrait {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            };
            node.align_items = if portrait {
                AlignItems::Stretch
            } else {
                AlignItems::Center
            };
            node.min_height = if portrait {
                Val::Px(88.)
            } else {
                Val::Auto
            };
        }
        if tabs.is_some() {
            node.width = if portrait {
                percent(100.)
            } else {
                Val::Auto
            };
            node.height = if portrait {
                Val::Px(44.)
            } else {
                Val::Auto
            };
            node.margin = if portrait {
                UiRect::top(Val::Px(8.))
            } else {
                UiRect {
                    left: Val::Px(15.),
                    top: Val::Px(15.),
                    ..default()
                }
            };
            node.overflow = if portrait {
                Overflow::scroll_x()
            } else {
                Overflow::clip()
            };
        }
        if gold.is_some() {
            node.margin = if portrait {
                UiRect {
                    right: Val::Px(8.),
                    top: Val::Px(4.),
                    ..default()
                }
            } else {
                UiRect {
                    right: Val::Px(45.),
                    top: Val::Px(15.),
                    ..default()
                }
            };
            node.align_self = if portrait {
                AlignSelf::FlexEnd
            } else {
                AlignSelf::Auto
            };
        }
        if container.is_some() {
            node.height = if portrait {
                percent(76.)
            } else {
                percent(70.)
            };
        }
        if items.is_some() {
            node.width = if portrait {
                percent(74.)
            } else {
                percent(85.)
            };
            node.padding = UiRect::all(if portrait {
                Val::Px(8.)
            } else {
                Val::Px(15.)
            });
        }
        if filters.is_some() {
            node.width = if portrait {
                percent(24.)
            } else {
                percent(11.)
            };
            node.margin = if portrait {
                UiRect::horizontal(Val::Px(4.))
            } else {
                UiRect {
                    left: Val::Px(5.),
                    right: Val::Px(10.),
                    ..default()
                }
            };
            node.row_gap = if portrait {
                Val::Px(8.)
            } else {
                Val::Px(12.)
            };
        }
        if row.is_some() {
            node.flex_wrap = if portrait {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            };
            node.column_gap = if portrait {
                percent(2.)
            } else {
                percent(1.5)
            };
            node.row_gap = if portrait {
                Val::Px(8.)
            } else {
                Val::Px(0.)
            };
            node.overflow = if portrait {
                Overflow::visible()
            } else {
                Overflow::clip()
            };
        }
        if card.is_some() {
            node.width = if portrait {
                percent(48.)
            } else {
                percent(23.)
            };
        }
    }
}

/// Applies phone layouts to character creation and horizontal selection screens.
pub fn update_creation_responsive_layout(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<(&mut Node, &CreationLayoutNode)>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let portrait = uses_portrait_layout(window.width(), window.height());

    for (mut node, layout) in &mut nodes {
        apply_creation_layout(&mut node, *layout, portrait);
    }
}

/// Applies one responsive creation rule to a UI node.
fn apply_creation_layout(node: &mut Node, layout: CreationLayoutNode, portrait: bool) {
    let zero_rect = UiRect::all(Val::Px(0.));

    match layout {
        CreationLayoutNode::CharacterScreen => {
            node.overflow = if portrait {
                Overflow::scroll_y()
            } else {
                Overflow::visible()
            };
            node.padding = if portrait {
                UiRect::horizontal(Val::Px(6.))
            } else {
                zero_rect
            };
            node.row_gap = if portrait {
                Val::Px(8.)
            } else {
                Val::Px(0.)
            };
        },
        CreationLayoutNode::CharacterTitle => {
            node.margin = if portrait {
                UiRect {
                    top: Val::Px(12.),
                    bottom: Val::Px(6.),
                    ..default()
                }
            } else {
                UiRect {
                    top: percent(5.),
                    bottom: percent(3.),
                    ..default()
                }
            };
        },
        CreationLayoutNode::CharacterContent => {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(55.)
            };
            node.height = if portrait {
                Val::Auto
            } else {
                percent(65.)
            };
            node.flex_direction = if portrait {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            };
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::SpaceBetween
            };
            node.row_gap = if portrait {
                Val::Px(12.)
            } else {
                Val::Px(0.)
            };
            node.flex_shrink = if portrait {
                0.
            } else {
                1.
            };
        },
        CreationLayoutNode::IdentityColumn => {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(45.)
            };
            node.height = if portrait {
                Val::Auto
            } else {
                percent(100.)
            };
            node.min_height = if portrait {
                Val::Px(285.)
            } else {
                Val::Auto
            };
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::Center
            };
            node.flex_shrink = if portrait {
                0.
            } else {
                1.
            };
        },
        CreationLayoutNode::AttributesColumn => {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(45.)
            };
            node.height = if portrait {
                Val::Auto
            } else {
                percent(100.)
            };
            node.min_height = if portrait {
                Val::Px(385.)
            } else {
                Val::Auto
            };
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::Center
            };
            node.flex_shrink = if portrait {
                0.
            } else {
                1.
            };
        },
        CreationLayoutNode::CharacterFooter => {
            node.position_type = if portrait {
                PositionType::Relative
            } else {
                PositionType::Absolute
            };
            node.bottom = if portrait {
                Val::Auto
            } else {
                percent(4.)
            };
            node.min_height = if portrait {
                Val::Px(64.)
            } else {
                Val::Auto
            };
            node.margin = if portrait {
                UiRect::bottom(Val::Px(12.))
            } else {
                zero_rect
            };
            node.align_items = AlignItems::Center;
            node.flex_shrink = if portrait {
                0.
            } else {
                1.
            };
        },
        CreationLayoutNode::SelectionTitle => {
            node.margin = if portrait {
                UiRect {
                    top: Val::Px(12.),
                    bottom: Val::Px(6.),
                    ..default()
                }
            } else {
                UiRect {
                    top: percent(3.),
                    bottom: percent(3.),
                    ..default()
                }
            };
        },
        CreationLayoutNode::SelectionWrapper => {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(96.)
            };
            node.height = if portrait {
                percent(80.)
            } else {
                percent(72.)
            };
        },
        CreationLayoutNode::SelectionViewport {
            center_cards,
        } => {
            node.height = percent(96.);
            node.justify_content = if portrait || !center_cards {
                JustifyContent::FlexStart
            } else {
                JustifyContent::Center
            };
        },
        CreationLayoutNode::SelectionCard => {
            node.width = if portrait {
                percent(78.)
            } else {
                percent(22.)
            };
            node.height = if portrait {
                percent(90.)
            } else {
                percent(94.)
            };
            node.margin = UiRect::horizontal(if portrait {
                percent(4.)
            } else {
                percent(1.5)
            });
        },
        CreationLayoutNode::SelectionFooter => {
            node.bottom = if portrait {
                percent(1.)
            } else {
                percent(3.)
            };
        },
        CreationLayoutNode::DeityScreen => {
            node.overflow = if portrait {
                Overflow::clip()
            } else {
                Overflow::visible()
            };
        },
        CreationLayoutNode::DeityTitle => {
            node.margin = if portrait {
                UiRect {
                    top: Val::Px(12.),
                    bottom: Val::Px(6.),
                    ..default()
                }
            } else {
                UiRect::vertical(percent(2.5))
            };
        },
        CreationLayoutNode::DeityCards => {
            node.width = if portrait {
                percent(100.)
            } else {
                percent(90.)
            };
            node.height = if portrait {
                percent(78.)
            } else {
                percent(76.)
            };
            node.margin.top = if portrait {
                Val::Px(20.)
            } else {
                Val::Px(0.)
            };
            node.justify_content = if portrait {
                JustifyContent::FlexStart
            } else {
                JustifyContent::Center
            };
            node.column_gap = if portrait {
                Val::Px(10.)
            } else {
                percent(2.)
            };
            node.overflow = if portrait {
                Overflow::scroll_x()
            } else {
                Overflow::visible()
            };
        },
        CreationLayoutNode::DeityCard => {
            node.width = if portrait {
                percent(78.)
            } else {
                percent(26.)
            };
            node.height = if portrait {
                percent(90.)
            } else {
                percent(94.)
            };
            node.margin = UiRect::horizontal(if portrait {
                percent(4.)
            } else {
                percent(1.5)
            });
            node.flex_shrink = if portrait {
                0.
            } else {
                1.
            };
        },
        CreationLayoutNode::DeityFooter => {
            node.position_type = if portrait {
                PositionType::Relative
            } else {
                PositionType::Absolute
            };
            node.bottom = if portrait {
                Val::Auto
            } else {
                percent(2.)
            };
            node.min_height = if portrait {
                Val::Px(56.)
            } else {
                Val::Auto
            };
            node.margin.top = if portrait {
                Val::Px(4.)
            } else {
                Val::Px(0.)
            };
            node.flex_shrink = 0.;
        },
    }
}

/// Converts one-finger drags over a scrollable area into vertical or horizontal scrolling.
pub fn touch_scroll_system(
    mut touch_events: MessageReader<TouchInput>,
    mut last_positions: Local<HashMap<u64, Vec2>>,
    mut start_positions: Local<HashMap<u64, Vec2>>,
    time: Res<Time>,
    mut gesture: ResMut<SelectionGestureState>,
    mut scroll_q: Query<
        (&mut ScrollPosition, &ComputedNode, &Interaction, Option<&HorizontalWheelScroll>),
        With<ScrollableContainer>,
    >,
) {
    for touch in touch_events.read() {
        match touch.phase {
            TouchPhase::Started => {
                last_positions.insert(touch.id, touch.position);
                start_positions.insert(touch.id, touch.position);
            },
            TouchPhase::Moved => {
                let Some(previous) = last_positions.insert(touch.id, touch.position) else {
                    continue;
                };
                let delta = touch.position - previous;
                if start_positions
                    .get(&touch.id)
                    .is_some_and(|start| touch.position.distance_squared(*start) >= 25.0)
                {
                    gesture.suppress_after_drag(time.elapsed_secs_f64());
                }
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
                start_positions.remove(&touch.id);
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

    #[test]
    /// Keeps phone typography readable without allowing small labels to crowd their controls.
    fn responsive_font_size_is_bounded_on_phones() {
        assert_eq!(responsive_font_size(390.0, 844.0, 1.4), 11.0);
        assert_eq!(responsive_font_size(390.0, 844.0, 5.0), 25.0);
        assert_eq!(responsive_font_size(1280.0, 720.0, 2.0), 14.4);
    }

    #[test]
    /// Keeps phone action cards smaller than the viewport in both dimensions.
    fn phone_action_card_size_leaves_context_visible() {
        let card = phone_panel_card_size(390.0, 844.0);
        assert!((card.x - 304.2).abs() < 0.01);
        assert!((card.y - 472.64).abs() < 0.01);
        assert!(card.x < 390.0);
        assert!(card.y < 844.0);
    }

    #[test]
    /// Verifies character creation stacks vertically and selection cards remain touch-sized.
    fn phone_creation_layout_uses_vertical_full_width_content() {
        let mut content = Node::default();
        apply_creation_layout(&mut content, CreationLayoutNode::CharacterContent, true);
        assert_eq!(content.width, percent(100.));
        assert_eq!(content.height, Val::Auto);
        assert_eq!(content.flex_direction, FlexDirection::Column);

        let mut card = Node::default();
        apply_creation_layout(&mut card, CreationLayoutNode::SelectionCard, true);
        assert_eq!(card.width, percent(78.));
        assert_eq!(card.height, percent(90.));
    }
}
