

use crate::core::RightColumnTooltip;
use crate::core::assets::WorldAssets;
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_text, recolor};
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::utils::*;
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::core::ui::scrollbar::{ScrollableContainer, ScrollbarTrack, ScrollbarThumb, on_scrollbar_thumb_drag};
use crate::core::ui::level_up::LevelUpPending;
use crate::core::states::GameState;
use crate::core::actions::trigger_level_up;
use crate::core::catalog::catalog::{all_equipment, get_equipment, get_artifact};
use crate::core::catalog::equipment::{Equipment, Kind};
use crate::core::catalog::wearables::WearableSlot;
use crate::core::catalog::weapons::{Category, Hand};
use crate::core::audio::PlayAudioMsg;
use crate::core::utils::cursor;
use crate::core::constants::*;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use std::collections::HashMap;
use crate::core::ui::playing::{equip_item, name_with_level, InfoTooltip};

#[derive(Resource, Default, Clone, Debug)]
pub struct CraftSelection {
    pub selected: Vec<String>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct CraftItemSelection {
    pub items: Vec<String>,
}

#[derive(Component)]
pub struct CraftContentWrapper;

#[derive(Component, Debug, Clone)]
pub struct LeftArtifactBtn {
    pub name: String,
}

#[derive(Component, Debug, Clone)]
pub struct MiddleArtifactBtn {
    pub name: String,
}

#[derive(Component, Debug, Clone)]
pub struct CraftItemBtn {
    pub item_name: String,
}

#[derive(Component, Debug, Clone)]
pub struct CraftAllBtn;

#[derive(Component)]
pub struct LeftScrollMarker;

#[derive(Component)]
pub struct BenchScrollMarker;

#[derive(Component)]
pub struct RecipesScrollMarker;

#[derive(Resource, Default, Clone, Debug)]
pub struct CraftSeed {
    pub artifacts: Vec<String>,
}

pub fn setup_craft_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    mut seed: ResMut<CraftSeed>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    let mut selection = CraftSelection::default();
    for art in seed.artifacts.drain(..) {
        if player.inventory.iter().any(|x| x == &art) {
            selection.selected.push(art);
        }
    }

    commands.insert_resource(selection.clone());
    commands.insert_resource(CraftItemSelection::default());

    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_craft");
        commands.entity(panel_entity).with_children(|parent| {
            build_craft_content(
                parent,
                &assets,
                &localization,
                settings.language,
                &player,
                &selection,
                &CraftItemSelection::default(),
            );
        });
    }
}

pub fn update_craft_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    selection: Res<CraftSelection>,
    item_selection: Res<CraftItemSelection>,
    wrapper_q: Query<Entity, With<CraftContentWrapper>>,
    children_q: Query<&Children>,
    scroll_q: Query<(
        &ScrollPosition,
        Has<LeftScrollMarker>,
        Has<BenchScrollMarker>,
        Has<RecipesScrollMarker>,
    )>,
) {
    if player.is_changed() || selection.is_changed() || item_selection.is_changed() {
        if let Some(wrapper_entity) = wrapper_q.iter().next() {
            // Preserve current scroll positions across the rebuild.
            let (mut left_scroll, mut bench_scroll, mut recipes_scroll) = (0., 0., 0.);
            for (sp, is_left, is_bench, is_recipes) in &scroll_q {
                if is_left {
                    left_scroll = sp.0.y;
                }
                if is_bench {
                    bench_scroll = sp.0.y;
                }
                if is_recipes {
                    recipes_scroll = sp.0.y;
                }
            }

            despawn_descendants_manual(&mut commands, wrapper_entity, &children_q);
            commands.entity(wrapper_entity).with_children(|parent| {
                build_craft_content_inner(
                    parent,
                    &assets,
                    &localization,
                    settings.language,
                    &player,
                    &selection,
                    &item_selection,
                    left_scroll,
                    bench_scroll,
                    recipes_scroll,
                );
            });
        }
    }
}

pub fn build_craft_content(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    selection: &CraftSelection,
    item_selection: &CraftItemSelection,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                padding: UiRect::all(Val::Percent(5.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            CraftContentWrapper,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|parent| {
                    build_craft_content_inner(
                        parent,
                        assets,
                        localization,
                        lang,
                        player,
                        selection,
                        item_selection,
                        0.,
                        0.,
                        0.,
                    );
                });

            parent.spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Px(30.),
                    margin: UiRect::top(Val::Px(5.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    add_text("Click outside the panel to go back", "medium", 1.4, assets),
                    TextColor(Color::srgba_u8(180, 180, 180, 200)),
                ));
            });
        });
}

pub fn build_craft_content_inner(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    selection: &CraftSelection,
    item_selection: &CraftItemSelection,
    left_scroll: f32,
    bench_scroll: f32,
    recipes_scroll: f32,
) {
    // 1. Top row: stats
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Px(75.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Relative,
            margin: UiRect::bottom(Val::Px(10.)),
            ..default()
        })
        .with_children(|parent| {
            // Left: Title
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(30.),
                    ..default()
                },
                add_text(localization.get("craft", lang), "bold", 3.6, assets),
                TextColor(BUTTON_TEXT_COLOR),
            ));

            // Right: Resources Display (Mana + Gold + AP)
            parent.spawn(Node {
                position_type: PositionType::Absolute,
                right: Val::Px(30.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(15.),
                ..default()
            }).with_children(|parent| {
                // Mana icon + text (Only total mana, no cost/total)
                parent.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.),
                    align_items: AlignItems::Center,
                    ..default()
                }).with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Vw(2.4),
                            height: Val::Vw(2.4),
                            ..default()
                        },
                        ImageNode::new(assets.image("mana"))
                            .with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(player.mana().to_string(), "bold", 2.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

                // Gold icon + text
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.),
                        ..default()
                    },
                    Interaction::default(),
                    Pickable::default(),
                    InfoTooltip::Gold,
                )).with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Vw(2.4),
                            height: Val::Vw(2.4),
                            ..default()
                        },
                        ImageNode::new(assets.image("gold"))
                            .with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(player.gold.to_string(), "bold", 2.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

                // AP icon + text
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.),
                        ..default()
                    },
                    Interaction::default(),
                    Pickable::default(),
                    InfoTooltip::ActionPoints,
                )).with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Vw(2.4),
                            height: Val::Vw(2.4),
                            ..default()
                        },
                        ImageNode::new(assets.image("ap"))
                            .with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(player.ap.to_string(), "bold", 2.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });
            });
        });

    // 2. Main Three-Column Layout
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(82.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(10.),
            overflow: Overflow::clip(),
            ..default()
        })
        .with_children(|parent| {
            // --- LEFT COLUMN: Current Artifacts ---
            let mut col1_cmd = parent.spawn((
                Node {
                    width: Val::Percent(32.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.)),
                    border_radius: BorderRadius::all(Val::Px(4.)),
                    overflow: Overflow::clip(),
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(10, 10, 20, 220)),
                BorderColor::all(BUTTON_BORDER_COLOR),
            ));
            col1_cmd.with_children(|parent| {
                parent.spawn(Node {
                    width: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(8.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .insert(BackgroundColor(Color::srgba_u8(20, 20, 35, 255)))
                .with_children(|parent| {
                    parent.spawn((
                        add_text("OWNED ARTIFACTS", "bold", 1.8, assets),
                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                    ));
                });

                let mut container_cmd = parent.spawn((
                    Node {
                        width: Val::Percent(100.),
                        flex_grow: 1.0,
                        min_height: Val::Px(0.),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(20.),
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                        },
                        row_gap: Val::Px(8.),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollableContainer,
                    ScrollPosition(Vec2::new(0., left_scroll)),
                    LeftScrollMarker,
                    Interaction::default(),
                ));
                let container_entity = container_cmd.id();
                container_cmd.with_children(|parent| {
                    let mut player_arts = Vec::new();
                    for key in &player.inventory {
                        if let Some(eq) = get_equipment(key) {
                            if let Equipment::Artifact(_) = eq {
                                player_arts.push(key.clone());
                            }
                        }
                    }
                    for sel in &selection.selected {
                        if let Some(pos) = player_arts.iter().position(|x| x == sel) {
                            player_arts.remove(pos);
                        }
                    }

                    let mut left_map = HashMap::new();
                    for key in player_arts {
                        *left_map.entry(key).or_insert(0) += 1;
                    }

                    let mut left_keys: Vec<String> = left_map.keys().cloned().collect();
                    left_keys.sort_by(|a, b| {
                        let pa = get_artifact(a).map(|x| x.price).unwrap_or(0);
                        let pb = get_artifact(b).map(|x| x.price).unwrap_or(0);
                        pb.cmp(&pa).then(a.cmp(b))
                    });

                    if left_keys.is_empty() {
                        parent.spawn((
                            add_text("No artifacts available", "medium", 1.5, assets),
                            TextColor(Color::srgba_u8(180, 180, 180, 250)),
                        ));
                    } else {
                        for key in left_keys {
                            let art_eq = get_equipment(&key).unwrap();
                            let count = left_map.get(&key).unwrap();
                            let art = get_artifact(&key).unwrap();
                            let k = art.kind;

                            let label = if *count > 1 {
                                format!("{} (x{})", key, count)
                            } else {
                                key.clone()
                            };

                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(8.),
                                    padding: UiRect::all(Val::Px(6.)),
                                    border: UiRect::all(Val::Px(1.)),
                                    border_radius: BorderRadius::all(Val::Px(4.)),
                                    position_type: PositionType::Relative,
                                    ..default()
                                },
                                BackgroundColor(BAR_BG_COLOR),
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                LeftArtifactBtn { name: key.clone() },
                            ))
                            .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                            .observe(recolor::<Out>(BAR_BG_COLOR))
                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                            .observe(handle_left_artifact_click)
                            .with_children(|parent| {
                                parent.spawn((
                                    Node {
                                        width: Val::Px(35.),
                                        height: Val::Px(35.),
                                        border_radius: BorderRadius::all(Val::Px(3.)),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image(format!("build_{}", art_eq.name()))),
                                ));

                                parent.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(label, "bold", 1.6, assets),
                                        TextColor(Color::WHITE),
                                    ));

                                    parent.spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(6.),
                                        ..default()
                                    }).with_children(|parent| {
                                        // Kind icon
                                        parent.spawn((
                                            Node {
                                                width: Val::Px(16.),
                                                height: Val::Px(16.),
                                                ..default()
                                            },
                                            ImageNode::new(assets.image(k.to_string().to_lowercase()))
                                                .with_mode(NodeImageMode::Stretch),
                                        ));
                                        // Kind name
                                        parent.spawn((
                                            add_text(k.to_string(), "medium", 1.3, assets),
                                            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                                        ));
                                    });
                                });

                                // Gold icon + price in the top-right corner
                                parent.spawn(Node {
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(6.),
                                    top: Val::Px(6.),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(4.),
                                    ..default()
                                }).with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(16.),
                                            height: Val::Px(16.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image("gold")),
                                    ));
                                    parent.spawn((
                                        add_text(art.price.to_string(), "medium", 1.3, assets),
                                        TextColor(Color::srgb(1.0, 0.84, 0.0)),
                                    ));
                                });
                            });
                        }
                    }
                });

                // Spawn scrollbar for Owned Artifacts
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(10.),
                        top: Val::Px(45.),
                        bottom: Val::Px(5.),
                        right: Val::Px(3.),
                        border_radius: BorderRadius::all(Val::Px(5.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 170)),
                    Visibility::Hidden,
                    ScrollbarTrack { container: container_entity },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.),
                            height: Val::Px(32.),
                            top: Val::Px(0.),
                            border_radius: BorderRadius::all(Val::Px(5.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(230, 205, 120, 240)),
                        Button,
                        Interaction::default(),
                        Pickable::default(),
                        ScrollbarThumb { container: container_entity },
                    ))
                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                    .observe(on_scrollbar_thumb_drag);
                });
            });

            // --- MIDDLE COLUMN: Selected Artifacts & Kind Distribution ---
            let mut col2_cmd = parent.spawn((
                Node {
                    width: Val::Percent(34.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.)),
                    border_radius: BorderRadius::all(Val::Px(4.)),
                    overflow: Overflow::clip(),
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(10, 10, 20, 220)),
                BorderColor::all(BUTTON_BORDER_COLOR),
            ));
            col2_cmd.with_children(|parent| {
                let total_selected_price: u32 = selection
                    .selected
                    .iter()
                    .filter_map(|n| get_artifact(n))
                    .map(|a| a.price)
                    .sum();

                parent.spawn(Node {
                    width: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(8.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    ..default()
                })
                .insert(BackgroundColor(Color::srgba_u8(20, 20, 35, 255)))
                .with_children(|parent| {
                    parent.spawn((
                        add_text("CRAFTING BENCH", "bold", 1.8, assets),
                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                    ));
                    // Total gold cost of the bench at the right corner of the title space
                    parent.spawn(Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(10.),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.),
                        ..default()
                    }).with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: Val::Px(20.),
                                height: Val::Px(20.),
                                ..default()
                            },
                            ImageNode::new(assets.image("gold")),
                        ));
                        parent.spawn((
                            add_text(total_selected_price.to_string(), "bold", 1.8, assets),
                            TextColor(Color::srgb(1.0, 0.84, 0.0)),
                        ));
                    });
                });

                parent.spawn((
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Percent(45.),
                        min_height: Val::Px(0.),
                        position_type: PositionType::Relative,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                )).with_children(|parent| {
                let mut container_cmd = parent.spawn((
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(20.),
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                        },
                        row_gap: Val::Px(6.),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollableContainer,
                    ScrollPosition(Vec2::new(0., bench_scroll)),
                    BenchScrollMarker,
                    Interaction::default(),
                ));
                let container_entity = container_cmd.id();
                container_cmd.with_children(|parent| {
                    let mut mid_map = HashMap::new();
                    for key in &selection.selected {
                        *mid_map.entry(key.clone()).or_insert(0) += 1;
                    }

                    let mut mid_keys: Vec<String> = mid_map.keys().cloned().collect();
                    mid_keys.sort_by(|a, b| {
                        let pa = get_artifact(a).map(|x| x.price).unwrap_or(0);
                        let pb = get_artifact(b).map(|x| x.price).unwrap_or(0);
                        pb.cmp(&pa).then(a.cmp(b))
                    });

                    if mid_keys.is_empty() {
                        parent.spawn((
                            add_text("Click owned artifacts to craft", "medium", 1.5, assets),
                            TextColor(Color::srgba_u8(150, 150, 150, 200)),
                        ));
                    } else {
                        for key in mid_keys {
                            let art_eq = get_equipment(&key).unwrap();
                            let count = mid_map.get(&key).unwrap();
                            let art = get_artifact(&key).unwrap();
                            let k = art.kind;

                            let label = if *count > 1 {
                                format!("{} (x{})", key, count)
                            } else {
                                key.clone()
                            };

                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(8.),
                                    padding: UiRect::all(Val::Px(4.)),
                                    border: UiRect::all(Val::Px(1.)),
                                    border_radius: BorderRadius::all(Val::Px(3.)),
                                    position_type: PositionType::Relative,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba_u8(20, 20, 40, 255)),
                                BorderColor::all(Color::srgb(0.35, 0.55, 0.85)),
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                MiddleArtifactBtn { name: key.clone() },
                            ))
                            .observe(recolor::<Over>(Color::srgba_u8(35, 45, 75, 255)))
                            .observe(recolor::<Out>(Color::srgba_u8(20, 20, 40, 255)))
                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                            .observe(handle_middle_artifact_click)
                            .with_children(|parent| {
                                parent.spawn((
                                    Node {
                                        width: Val::Px(28.),
                                        height: Val::Px(28.),
                                        border_radius: BorderRadius::all(Val::Px(3.)),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image(format!("build_{}", art_eq.name()))),
                                ));

                                parent.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                }).with_children(|parent| {
                                    parent.spawn((
                                        add_text(label, "bold", 1.5, assets),
                                        TextColor(Color::WHITE),
                                    ));
                                    parent.spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(6.),
                                        ..default()
                                    }).with_children(|parent| {
                                        // Kind icon
                                        parent.spawn((
                                            Node {
                                                width: Val::Px(16.),
                                                height: Val::Px(16.),
                                                ..default()
                                            },
                                            ImageNode::new(assets.image(k.to_string().to_lowercase()))
                                                .with_mode(NodeImageMode::Stretch),
                                        ));
                                        // Kind name
                                        parent.spawn((
                                            add_text(k.to_string(), "medium", 1.3, assets),
                                            TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                                        ));
                                    });
                                });

                                // Gold icon + price in the top-right corner
                                parent.spawn(Node {
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(6.),
                                    top: Val::Px(4.),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(4.),
                                    ..default()
                                }).with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(16.),
                                            height: Val::Px(16.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image("gold")),
                                    ));
                                    parent.spawn((
                                        add_text(art.price.to_string(), "medium", 1.3, assets),
                                        TextColor(Color::srgb(1.0, 0.84, 0.0)),
                                    ));
                                });
                            });
                        }
                    }
                });

                // Spawn scrollbar for Bench list
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(10.),
                        top: Val::Px(0.),
                        bottom: Val::Px(0.),
                        right: Val::Px(3.),
                        border_radius: BorderRadius::all(Val::Px(5.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 170)),
                    Visibility::Hidden,
                    ScrollbarTrack { container: container_entity },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.),
                            height: Val::Px(32.),
                            top: Val::Px(0.),
                            border_radius: BorderRadius::all(Val::Px(5.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(230, 205, 120, 240)),
                        Button,
                        Interaction::default(),
                        Pickable::default(),
                        ScrollbarThumb { container: container_entity },
                    ))
                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                    .observe(on_scrollbar_thumb_drag);
                });
                });

                parent.spawn(Node {
                    width: Val::Percent(100.),
                    height: Val::Px(1.),
                    ..default()
                })
                .insert(BackgroundColor(BUTTON_BORDER_COLOR));

                parent.spawn(Node {
                    width: Val::Percent(100.),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.)),
                    row_gap: Val::Px(6.),
                    position_type: PositionType::Relative,
                    ..default()
                })
                .with_children(|parent| {
                    let mut total_selected_price = 0;
                    let mut selected_by_kind = HashMap::new();
                    for art_name in &selection.selected {
                        if let Some(art) = get_artifact(art_name) {
                            let k = art.kind;
                            let p = art.price;
                            total_selected_price += p;
                            *selected_by_kind.entry(k).or_insert(0) += p;
                        }
                    }

                    let all_kinds = [
                        Kind::Physical,
                        Kind::Fire,
                        Kind::Ice,
                        Kind::Nature,
                        Kind::Holy,
                        Kind::Shadow,
                    ];

                    let kind_colors = |k: Kind| match k {
                        Kind::Physical => Color::srgb(0.7, 0.7, 0.7),
                        Kind::Fire => Color::srgb(0.9, 0.3, 0.1),
                        Kind::Ice => Color::srgb(0.3, 0.7, 1.0),
                        Kind::Nature => Color::srgb(0.2, 0.8, 0.3),
                        Kind::Holy => Color::srgb(1.0, 0.85, 0.2),
                        Kind::Shadow => Color::srgb(0.5, 0.2, 0.8),
                    };

                    for k in all_kinds {
                        let k_price = selected_by_kind.get(&k).unwrap_or(&0);
                        let pct = if total_selected_price > 0 {
                            (*k_price as f32 / total_selected_price as f32) * 100.0
                        } else {
                            0.0
                        };

                        parent.spawn(Node {
                            width: Val::Percent(100.),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        }).with_children(|parent| {
                            parent.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.),
                                width: Val::Percent(40.),
                                ..default()
                            }).with_children(|parent| {
                                parent.spawn((
                                    Node {
                                        width: Val::Px(24.),
                                        height: Val::Px(24.),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image(k.to_string().to_lowercase()))
                                        .with_mode(NodeImageMode::Stretch),
                                ));
                                parent.spawn((
                                    add_text(k.to_string(), "bold", 1.6, assets),
                                    TextColor(Color::WHITE),
                                ));
                            });

                            parent.spawn(Node {
                                width: Val::Percent(45.),
                                height: Val::Px(16.),
                                border_radius: BorderRadius::all(Val::Px(4.)),
                                overflow: Overflow::clip(),
                                ..default()
                            })
                            .insert(BackgroundColor(Color::srgba_u8(30, 30, 40, 255)))
                            .with_children(|parent| {
                                parent.spawn(Node {
                                    width: Val::Percent(pct),
                                    height: Val::Percent(100.),
                                    ..default()
                                })
                                .insert(BackgroundColor(kind_colors(k)));
                            });

                            parent.spawn(Node {
                                width: Val::Percent(15.),
                                justify_content: JustifyContent::FlexEnd,
                                ..default()
                                }).with_children(|parent| {
                                parent.spawn((
                                    add_text(format!("{:.0}%", pct), "medium", 1.5, assets),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                    }
                });
            });

            // --- RIGHT COLUMN: Possible Craftable Items ---
            let mut col3_cmd = parent.spawn((
                Node {
                    width: Val::Percent(32.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.)),
                    border_radius: BorderRadius::all(Val::Px(4.)),
                    overflow: Overflow::clip(),
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(10, 10, 20, 220)),
                BorderColor::all(BUTTON_BORDER_COLOR),
            ));
            col3_cmd.with_children(|parent| {
                parent.spawn(Node {
                    width: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(8.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .insert(BackgroundColor(Color::srgba_u8(20, 20, 35, 255)))
                .with_children(|parent| {
                    parent.spawn((
                        add_text("POSSIBLE RECIPES", "bold", 1.8, assets),
                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                    ));
                });

                let mut container_cmd = parent.spawn((
                    Node {
                        width: Val::Percent(100.),
                        flex_grow: 1.0,
                        flex_shrink: 1.0,
                        min_height: Val::Px(0.),
                        margin: UiRect::bottom(Val::Px(14.)),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect {
                            left: Val::Px(10.),
                            right: Val::Px(20.),
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                        },
                        row_gap: Val::Px(10.),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollableContainer,
                    ScrollPosition(Vec2::new(0., recipes_scroll)),
                    RecipesScrollMarker,
                    Interaction::default(),
                ));
                let container_entity = container_cmd.id();
                container_cmd.with_children(|parent| {
                    let mut selected_by_kind = HashMap::new();
                    let mut total_selected_price = 0;
                    for art_name in &selection.selected {
                        if let Some(art) = get_artifact(art_name) {
                            let k = art.kind;
                            let p = art.price;
                            total_selected_price += p;
                            *selected_by_kind.entry(k).or_insert(0) += p;
                        }
                    }

                    let mut possible_items = Vec::new();

                    for item in all_equipment() {
                        if matches!(item, Equipment::Artifact(_)) {
                            continue;
                        }
                        if item.level() > player.level as u32 {
                            continue;
                        }

                        let wisdom_mod = player.wisdom_mod();
                        let percentage = (100.0 - wisdom_mod as f32).max(1.0);
                        let required_total_price = (item.price() as f32 * percentage / 100.0) as u32;

                        let item_kind = item.kind();
                        let satisfies_total = total_selected_price >= required_total_price;
                        let kind_selected_price = *selected_by_kind.get(&item_kind).unwrap_or(&0);
                        let required_kind_price = (required_total_price * 30) / 100;
                        let satisfies_kind = kind_selected_price >= required_kind_price;

                        if satisfies_total && satisfies_kind {
                            let mana_cost = (item.price() / 10).max(1);
                            let base_gold_cost = (item.price() as f32 * 0.15) as u32;
                            let gold_cost = (base_gold_cost as f32 * (1.0 - 0.01 * wisdom_mod as f32)).max(1.0) as u32;

                            possible_items.push((item.clone(), required_total_price, gold_cost, mana_cost));
                        }
                    }

                    let selected_set: std::collections::HashSet<String> =
                        item_selection.items.iter().cloned().collect();
                    possible_items.sort_by(|a, b| {
                        let sa = selected_set.contains(a.0.name());
                        let sb = selected_set.contains(b.0.name());
                        sb.cmp(&sa).then(b.0.price().cmp(&a.0.price()))
                    });

                    if possible_items.is_empty() {
                        parent.spawn((
                            add_text("Select more artifacts to unlock recipes", "medium", 1.5, assets),
                            TextColor(Color::srgba_u8(180, 180, 180, 200)),
                        ));
                    } else {
                        for (item, _r, _g, _m) in possible_items {
                            let is_selected = selected_set.contains(item.name());
                            let base_bg = if is_selected {
                                Color::srgba_u8(30, 60, 35, 235)
                            } else {
                                Color::srgba_u8(15, 15, 30, 200)
                            };
                            let border_col = if is_selected {
                                Color::srgb(0.3, 0.9, 0.4)
                            } else {
                                BUTTON_BORDER_COLOR
                            };

                            let mut box_cmd = parent.spawn((
                                Node {
                                    width: Val::Percent(100.),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(8.)),
                                    border: UiRect::all(Val::Px(if is_selected { 2. } else { 1. })),
                                    border_radius: BorderRadius::all(Val::Px(4.)),
                                    position_type: PositionType::Relative,
                                    flex_shrink: 0.,
                                    ..default()
                                },
                                BackgroundColor(base_bg),
                                BorderColor::all(border_col),
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                RightColumnTooltip::Equipment(item.name().to_string()),
                                CraftItemBtn { item_name: item.name().to_string() },
                            ));
                            box_cmd
                                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                                .observe(recolor::<Out>(base_bg))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .observe(handle_craft_item_select);
                            box_cmd.with_children(|parent| {
                                parent.spawn((
                                    Node {
                                        width: Val::Px(35.),
                                        height: Val::Px(35.),
                                        border_radius: BorderRadius::all(Val::Px(3.)),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image(format!("build_{}", item.name()))),
                                ));

                                parent.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    margin: UiRect::left(Val::Px(8.)),
                                    ..default()
                                }).with_children(|parent| {
                                    let display_name = name_with_level(
                                        item.name(),
                                        item.to_lowername().as_str(),
                                        item.level() as u8,
                                        localization,
                                        lang,
                                    );
                                    parent.spawn((
                                        add_text(display_name, "bold", 1.6, assets),
                                        TextColor(Color::WHITE),
                                    ));
                                });

                                // Price in the top right with icon (like the normal tab page)
                                parent.spawn(Node {
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(6.),
                                    top: Val::Px(6.),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(4.),
                                    ..default()
                                }).with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(16.),
                                            height: Val::Px(16.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image("gold")),
                                    ));
                                    parent.spawn((
                                        add_text(item.price().to_string(), "bold", 1.5, assets),
                                        TextColor(Color::srgb(1.0, 0.84, 0.0)),
                                    ));
                                });
                            });
                        }
                    }
                });

                // Spawn scrollbar
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(10.),
                        top: Val::Px(45.),
                        bottom: Val::Px(55.),
                        right: Val::Px(3.),
                        border_radius: BorderRadius::all(Val::Px(5.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 170)),
                    Visibility::Hidden,
                    ScrollbarTrack { container: container_entity },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.),
                            height: Val::Px(32.),
                            top: Val::Px(0.),
                            border_radius: BorderRadius::all(Val::Px(5.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(230, 205, 120, 240)),
                        Button,
                        Interaction::default(),
                        Pickable::default(),
                        ScrollbarThumb { container: container_entity },
                    ))
                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                    .observe(on_scrollbar_thumb_drag);
                });

                // Footer: craft-all button (bottom-right corner)
                let total_mana: u32 = item_selection
                    .items
                    .iter()
                    .filter_map(|n| get_equipment(n))
                    .map(|eq| (eq.price() / 10).max(1))
                    .sum();
                let can_craft_all =
                    !item_selection.items.is_empty() && player.ap >= 1 && player.mana() >= total_mana;

                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Px(50.),
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(10.)),
                        ..default()
                    })
                    .with_children(|parent| {
                        let btn_bg = if can_craft_all {
                            NORMAL_BUTTON_COLOR
                        } else {
                            Color::srgba_u8(40, 40, 50, 150)
                        };
                        let mut btn_cmd = parent.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(20.), Val::Px(10.)),
                                border: UiRect::all(Val::Px(1.)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.),
                                ..default()
                            },
                            BackgroundColor(btn_bg),
                            BorderColor::all(BUTTON_BORDER_COLOR),
                            Button,
                            Interaction::default(),
                            Pickable::default(),
                            CraftAllBtn,
                        ));
                        if can_craft_all {
                            btn_cmd
                                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .observe(handle_craft_all_click);
                        }
                        btn_cmd.with_children(|parent| {
                            // AP icon + cost
                            parent.spawn((
                                Node { width: Val::Px(18.), height: Val::Px(18.), ..default() },
                                ImageNode::new(assets.image("ap")).with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text("1", "bold", 1.4, assets),
                                TextColor(if player.ap >= 1 {
                                    Color::WHITE
                                } else {
                                    Color::srgb(0.85, 0.2, 0.2)
                                }),
                            ));
                            // Mana icon + total cost
                            parent.spawn((
                                Node { width: Val::Px(18.), height: Val::Px(18.), ..default() },
                                ImageNode::new(assets.image("mana")).with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text(total_mana.to_string(), "bold", 1.4, assets),
                                TextColor(if player.mana() >= total_mana {
                                    Color::WHITE
                                } else {
                                    Color::srgb(0.85, 0.2, 0.2)
                                }),
                            ));

                            parent.spawn(Node { width: Val::Px(4.), ..default() });

                            parent.spawn((
                                add_text("CRAFT", "bold", 1.5, assets),
                                TextColor(if can_craft_all {
                                    BUTTON_TEXT_COLOR
                                } else {
                                    Color::srgba_u8(130, 130, 140, 255)
                                }),
                            ));
                        });
                    });
            });
        });
}
fn has_empty_slot_for(player: &Player, equipment: &Equipment) -> bool {
    match equipment {
        Equipment::Wearable(w) => match w.slot {
            WearableSlot::Helmet => player.helmet.is_none(),
            WearableSlot::Chestplate => player.armor.is_none(),
            WearableSlot::Boots => player.boots.is_none(),
            WearableSlot::Gloves => player.gloves.is_none(),
            WearableSlot::Accessory => player.accessory.is_none() || player.accessory2.is_none(),
        },
        Equipment::Weapon(w) => {
            let is_lh_two_hand = player
                .weapon_lh
                .as_deref()
                .and_then(|k| get_equipment(k))
                .map(|eq| match eq {
                    Equipment::Weapon(lh_w) => lh_w.hand == Hand::TwoHand,
                    _ => false,
                })
                .unwrap_or(false);

            if w.hand == Hand::TwoHand {
                player.weapon_lh.is_none() && player.weapon_rh.is_none()
            } else if matches!(w.category, Category::Shield | Category::Book) {
                player.weapon_rh.is_none() && !is_lh_two_hand
            } else {
                !is_lh_two_hand && (player.weapon_lh.is_none() || player.weapon_rh.is_none())
            }
        },
        _ => false,
    }
}

pub fn handle_left_artifact_click(
    event: On<Pointer<Click>>,
    mut selection: ResMut<CraftSelection>,
    btn_q: Query<&LeftArtifactBtn>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        selection.selected.push(btn.name.clone());
        play_audio_msg.write(PlayAudioMsg::new("click"));
    }
}

pub fn handle_middle_artifact_click(
    event: On<Pointer<Click>>,
    mut selection: ResMut<CraftSelection>,
    btn_q: Query<&MiddleArtifactBtn>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        if let Some(pos) = selection.selected.iter().position(|x| x == &btn.name) {
            selection.selected.remove(pos);
            play_audio_msg.write(PlayAudioMsg::new("click"));
        }
    }
}

pub fn handle_craft_item_select(
    event: On<Pointer<Click>>,
    mut item_selection: ResMut<CraftItemSelection>,
    btn_q: Query<&CraftItemBtn>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        if let Some(pos) = item_selection.items.iter().position(|x| x == &btn.item_name) {
            item_selection.items.remove(pos);
        } else {
            item_selection.items.push(btn.item_name.clone());
        }
        play_audio_msg.write(PlayAudioMsg::new("click"));
    }
}

pub fn handle_craft_all_click(
    _event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut selection: ResMut<CraftSelection>,
    mut item_selection: ResMut<CraftItemSelection>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut level_up: ResMut<LevelUpPending>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if item_selection.items.is_empty() {
        play_audio_msg.write(PlayAudioMsg::new("error"));
        return;
    }

    let total_mana: u32 = item_selection
        .items
        .iter()
        .filter_map(|n| get_equipment(n))
        .map(|eq| (eq.price() / 10).max(1))
        .sum();

    if player.ap < 1 || player.mana() < total_mana {
        play_audio_msg.write(PlayAudioMsg::new("error"));
        return;
    }

    let new_mana = player.mana().saturating_sub(total_mana);
    player.set_mana(new_mana);

    // Destroy selected artifacts
    for req in &selection.selected {
        if let Some(pos) = player.inventory.iter().position(|x| x == req) {
            player.inventory.remove(pos);
        }
    }
    selection.selected.clear();
    
    let crafted: Vec<String> = item_selection.items.clone();
    for item_name in &crafted {
        if let Some(item_eq) = get_equipment(item_name) {
            let has_empty = has_empty_slot_for(&player, &item_eq);
            player.inventory.push(item_name.clone());
            if has_empty {
                equip_item(&mut player, item_name);
            }
        }
    }
    item_selection.items.clear();

    play_audio_msg.write(PlayAudioMsg::new("work"));

    if let Some(toast) = toast_container_q.iter().next() {
        let msg = if crafted.len() == 1 {
            format!("Successfully crafted {}!", crafted[0])
        } else {
            format!("Successfully crafted {} items!", crafted.len())
        };
        spawn_toast(
            &mut commands,
            &assets,
            msg,
            Color::srgba(0.05, 0.20, 0.05, 0.93),
            Color::srgb(0.20, 0.85, 0.20),
            Color::srgb(0.80, 1.0, 0.80),
            toast,
        );
    }

    if player.ap <= 1 {
        trigger_level_up(&mut player, &mut level_up, &mut play_audio_msg, &mut next_game_state);
    } else {
        player.ap -= 1;
    }

    player.as_mut();
}
