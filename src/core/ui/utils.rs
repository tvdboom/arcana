use crate::core::assets::WorldAssets;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::settings::Language;
use crate::core::states::GameState;
use crate::core::utils::cursor;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

#[derive(Component)]
pub struct PanelCmp;

#[derive(Component)]
pub struct PlayScreenColumns2And3;

#[derive(Component)]
pub struct PlayScreenColumnsContainer;

pub const SLIDER_WIDTH: f32 = 280.0;
pub const SLIDER_VALUE_WIDTH: f32 = 120.0;

// Generic helper to despawn an entity and all its descendants manually
pub fn despawn_recursive_manual(
    commands: &mut Commands,
    entity: Entity,
    children_q: &Query<&Children>,
) {
    if let Ok(children) = children_q.get(entity) {
        for child in children.iter() {
            despawn_recursive_manual(commands, child, children_q);
        }
    }
    commands.entity(entity).try_despawn();
}

pub fn despawn_descendants_manual(
    commands: &mut Commands,
    entity: Entity,
    children_q: &Query<&Children>,
) {
    if let Ok(children) = children_q.get(entity) {
        for child in children.iter() {
            despawn_recursive_manual(commands, child, children_q);
        }
    }
}

// System to cleanup any opened panel UI (used for OnExit of panel states)
pub fn cleanup_panel_ui(
    mut commands: Commands,
    panel_q: Query<Entity, With<PanelCmp>>,
    mut columns_2_3_q: Query<&mut Node, With<PlayScreenColumns2And3>>,
    children_q: Query<&Children>,
) {
    for entity in &panel_q {
        despawn_recursive_manual(&mut commands, entity, &children_q);
    }
    for mut node in &mut columns_2_3_q {
        node.display = Display::Flex;
    }
}

pub fn global_click_listener(
    trigger: On<Pointer<Click>>,
    state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut play_audio_msg: MessageWriter<crate::core::audio::PlayAudioMsg>,
    parent_q: Query<&ChildOf>,
    panel_q: Query<&Interaction, With<PanelCmp>>,
    action_btn_q: Query<&crate::core::actions::ActionButton>,
) {
    let current_state = state.get();
    if !matches!(
        current_state,
        GameState::Shop | GameState::Work | GameState::Study | GameState::Rest | GameState::Train
    ) {
        return;
    }

    // 1. If clicked entity or its ancestors is an ActionButton, let its own handler transition immediately
    let clicked_entity = trigger.original_event_target();
    let mut current = clicked_entity;
    loop {
        if action_btn_q.get(current).is_ok() {
            return;
        }
        if let Ok(parent) = parent_q.get(current) {
            current = parent.0;
        } else {
            break;
        }
    }

    // 2. If the panel itself is hovered or pressed, the click is inside the panel
    for interaction in &panel_q {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            return;
        }
    }

    // 3. Otherwise, click is outside! Close panel.
    play_audio_msg.write(crate::core::audio::PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}

// Base panel spawning helper
pub fn spawn_panel_base(
    commands: &mut Commands,
    assets: &WorldAssets,
    container_entity: Entity,
    bg_image_key: &str,
) -> Entity {
    // Spawn Panel inside columns container
    let mut panel_entity = Entity::PLACEHOLDER;
    commands.entity(container_entity).with_children(|parent| {
        panel_entity = parent
            .spawn((
                Node {
                    width: percent(66.5),
                    height: percent(100.),
                    align_items: AlignItems::Stretch,
                    justify_content: JustifyContent::Stretch,
                    ..default()
                },
                ImageNode {
                    image: assets.image(bg_image_key),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(0.55, 0.55, 0.55, 1.0),
                    ..default()
                },
                Button,
                Interaction::default(),
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
                GlobalZIndex(910),
                PanelCmp,
            ))
            .id();
    });
    panel_entity
}

// Slider calculations
pub fn slider_relative_x_from_cursor(
    track_transform: &GlobalTransform,
    window: &Window,
    cursor_x: f32,
) -> f32 {
    let track_center_x = track_transform.translation().x + window.width() / 2.0;
    let track_left = track_center_x - SLIDER_WIDTH / 2.0;
    (cursor_x - track_left).clamp(0., SLIDER_WIDTH)
}

pub fn slider_stage_from_relative_x(relative_x: f32, stages_count: u32) -> u32 {
    let frac = relative_x / SLIDER_WIDTH;
    ((frac * (stages_count - 1) as f32).round() as u32).clamp(0, stages_count - 1)
}

pub fn update_slider_visuals(relative_x: f32, handle_node: &mut Node, value_node: &mut Node) {
    let relative_x = relative_x.clamp(0., SLIDER_WIDTH);
    handle_node.left = Val::Px(relative_x - 12.);
    let stage = slider_stage_from_relative_x(relative_x, 3);
    let notch_x = (stage as f32 / 2.0) * SLIDER_WIDTH;
    value_node.left = Val::Px(notch_x - SLIDER_VALUE_WIDTH / 2.);
}

// Spawns a generic 3-stage slider
pub fn spawn_intensity_slider<
    T: Component,
    H: Component,
    VNode: Component,
    VText: Component,
    B: Component + From<u32>,
>(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    current_stage: u32,
    stages_count: u32,
    stage_names: &[&str],
    track_marker: T,
    handle_marker: H,
    vnode_marker: VNode,
    vtext_marker: VText,
    btn_marker_fn: impl Fn(u32) -> B,
) -> (Entity, Vec<Entity>, Entity) {
    let mut track_id = Entity::PLACEHOLDER;
    let mut btn_ids = Vec::new();
    let mut handle_id = Entity::PLACEHOLDER;

    parent
        .spawn(Node {
            width: Val::Px(SLIDER_WIDTH),
            height: Val::Px(68.),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            track_id = parent
                .spawn((
                    Node {
                        width: Val::Px(SLIDER_WIDTH),
                        height: Val::Px(30.),
                        position_type: PositionType::Relative,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    BackgroundColor(Color::srgba(0., 0., 0., 0.01)),
                    track_marker,
                ))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.),
                            top: Val::Px(12.),
                            width: percent(100.),
                            height: Val::Px(6.),
                            border_radius: BorderRadius::all(Val::Px(3.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(60, 60, 80, 200)),
                        Pickable::IGNORE,
                    ));

                    for i in 0..stages_count {
                        let notch_x = (i as f32 / (stages_count - 1) as f32) * SLIDER_WIDTH;
                        parent.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(notch_x - 2.),
                                top: Val::Px(5.),
                                width: Val::Px(4.),
                                height: Val::Px(20.),
                                border_radius: BorderRadius::all(Val::Px(2.)),
                                ..default()
                            },
                            BackgroundColor(crate::core::constants::BUTTON_BORDER_COLOR),
                            Pickable::IGNORE,
                        ));
                    }

                    for i in 0..stages_count {
                        let (left, width) = if i == 0 {
                            (0., SLIDER_WIDTH / (2. * (stages_count - 1) as f32))
                        } else if i == stages_count - 1 {
                            (
                                SLIDER_WIDTH * (2. * (stages_count - 1) as f32 - 1.)
                                    / (2. * (stages_count - 1) as f32),
                                SLIDER_WIDTH / (2. * (stages_count - 1) as f32),
                            )
                        } else {
                            (
                                (i as f32 - 0.5) * SLIDER_WIDTH / (stages_count - 1) as f32,
                                SLIDER_WIDTH / (stages_count - 1) as f32,
                            )
                        };

                        let btn = parent
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(left),
                                    top: Val::Px(0.),
                                    width: Val::Px(width),
                                    height: Val::Px(30.),
                                    ..default()
                                },
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                BackgroundColor(Color::srgba(0., 0., 0., 0.01)),
                                btn_marker_fn(i),
                            ))
                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                            .id();
                        btn_ids.push(btn);
                    }

                    let initial_frac = current_stage as f32 / (stages_count - 1) as f32;
                    handle_id = parent
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(24.),
                                height: Val::Px(24.),
                                top: Val::Px(3.),
                                left: Val::Px(initial_frac * SLIDER_WIDTH - 12.),
                                border: UiRect::all(Val::Px(2.)),
                                border_radius: BorderRadius::all(Val::Px(12.)),
                                ..default()
                            },
                            BackgroundColor(crate::core::constants::BUTTON_TEXT_COLOR),
                            BorderColor::all(crate::core::constants::BUTTON_BORDER_COLOR),
                            Button,
                            Interaction::default(),
                            Pickable::default(),
                            handle_marker,
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .id();
                })
                .id();

            let initial_frac = current_stage as f32 / (stages_count - 1) as f32;
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(34.),
                        left: Val::Px(initial_frac * SLIDER_WIDTH - SLIDER_VALUE_WIDTH / 2.),
                        width: Val::Px(SLIDER_VALUE_WIDTH),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    vnode_marker,
                ))
                .with_children(|parent| {
                    let text_str = localization.get(stage_names[current_stage as usize], lang);
                    parent.spawn((
                        add_text(
                            text_str,
                            "bold",
                            crate::core::constants::BUTTON_TEXT_SIZE,
                            assets,
                        ),
                        TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                        TextLayout::justify(Justify::Center),
                        vtext_marker,
                    ));
                });
        });

    (track_id, btn_ids, handle_id)
}

// Spawns a generic race/class style selection card and returns the border overlay entity for click observation
pub fn spawn_card_ui<M: Component>(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    title: &str,
    description: &str,
    image_key: &str,
    ap_cost_opt: Option<u32>,
    secondary_cost_opt: Option<(u32, &'static str, Color)>,
    marker: M,
) -> Entity {
    let mut border_entity = Entity::PLACEHOLDER;
    parent
        .spawn((
            Node {
                width: percent(30.),
                height: percent(98.),
                position_type: PositionType::Relative,
                margin: UiRect::horizontal(percent(1.)),
                top: percent(-2.0),
                ..default()
            },
            BackgroundColor(crate::core::constants::NORMAL_BUTTON_COLOR),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(percent(1.5)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: percent(100.),
                            height: percent(50.),
                            ..default()
                        },
                        ImageNode::new(assets.image(image_key)).with_mode(NodeImageMode::Stretch),
                    ));

                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: percent(50.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            },
                            ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    margin: UiRect::vertical(percent(4.5)),
                                    ..default()
                                },
                                add_text(title, "bold", 2.2, assets),
                                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                            ));

                            parent.spawn((
                                Node {
                                    width: percent(85.),
                                    margin: UiRect::horizontal(percent(7.5)),
                                    ..default()
                                },
                                add_text(description, "medium", 1.8, assets),
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            if ap_cost_opt.is_some() || secondary_cost_opt.is_some() {
                parent
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.),
                        right: Val::Vw(1.9),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.),
                        ..default()
                    })
                    .with_children(|parent| {
                        if let Some((val, icon_key, color)) = secondary_cost_opt {
                            parent
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(4.),
                                        padding: UiRect::axes(Val::Px(8.), Val::Px(4.)),
                                        border_radius: BorderRadius::all(Val::Px(6.)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0., 0., 0., 0.85)),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(20.),
                                            height: Val::Px(20.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image(icon_key))
                                            .with_mode(NodeImageMode::Stretch),
                                    ));
                                    parent.spawn((
                                        add_text(val.to_string(), "bold", 1.6, assets),
                                        TextColor(color),
                                    ));
                                });
                        }

                        if let Some(ap_cost) = ap_cost_opt {
                            parent
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(4.),
                                        padding: UiRect::axes(Val::Px(8.), Val::Px(4.)),
                                        border_radius: BorderRadius::all(Val::Px(6.)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0., 0., 0., 0.85)),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(20.),
                                            height: Val::Px(20.),
                                            ..default()
                                        },
                                        ImageNode::new(assets.image("ap"))
                                            .with_mode(NodeImageMode::Stretch),
                                    ));
                                    parent.spawn((
                                        add_text(ap_cost.to_string(), "bold", 1.6, assets),
                                        TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                                    ));
                                });
                        }
                    });
            }

            border_entity = parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(110.),
                        height: percent(110.),
                        left: percent(-5.),
                        top: percent(-5.),
                        ..default()
                    },
                    ImageNode::new(assets.image("border")).with_mode(NodeImageMode::Stretch),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    marker,
                ))
                .observe(crate::core::menu::utils::reimage::<Over>(assets.image("border_hover")))
                .observe(crate::core::menu::utils::reimage::<Out>(assets.image("border")))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .id();
        });
    border_entity
}
