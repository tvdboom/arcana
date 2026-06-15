use crate::core::assets::WorldAssets;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::utils::*;
use bevy::prelude::*;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkSliderState(pub u32); // 0 = Light, 1 = Regular, 2 = Heavy

#[derive(Component)]
pub struct WorkContentWrapper;

#[derive(Component)]
pub struct WorkSliderTrack;
#[derive(Component)]
pub struct WorkSliderHandle;
#[derive(Component)]
pub struct WorkSliderValueNode;
#[derive(Component)]
pub struct WorkSliderValueText;

#[derive(Component)]
pub struct WorkSliderStageButton(pub u32);
impl From<u32> for WorkSliderStageButton {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

#[derive(Component)]
pub struct WorkCardMarker(pub u32); // 0 = Clerical, 1 = Craft, 2 = Manual

pub fn setup_work_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    slider_state: Res<WorkSliderState>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_work");
        let mut track_ent = Entity::PLACEHOLDER;
        let mut stage_ents = Vec::new();
        let mut handle_ent = Entity::PLACEHOLDER;
        let mut card_ents = Vec::new();

        commands.entity(panel_entity).with_children(|parent| {
            let (t, s, h, c) = build_work_content(
                parent,
                &assets,
                &localization,
                settings.language,
                &player,
                slider_state.0,
            );
            track_ent = t;
            stage_ents = s;
            handle_ent = h;
            card_ents = c;
        });

        commands.entity(track_ent).observe(handle_work_slider_clicks_track);
        for stage in stage_ents {
            commands.entity(stage).observe(handle_work_slider_clicks);
        }
        commands
            .entity(handle_ent)
            .observe(handle_work_slider_drag)
            .observe(handle_work_slider_release);

        for card in card_ents {
            commands.entity(card).observe(crate::core::actions::handle_work_card_clicks);
        }
    }
}

pub fn update_work_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    slider_state: Res<WorkSliderState>,
    wrapper_q: Query<Entity, With<WorkContentWrapper>>,
    children_q: Query<&Children>,
) {
    if slider_state.is_changed() || player.is_changed() {
        if let Some(wrapper_entity) = wrapper_q.iter().next() {
            despawn_descendants_manual(&mut commands, wrapper_entity, &children_q);
            let mut track_ent = Entity::PLACEHOLDER;
            let mut stage_ents = Vec::new();
            let mut handle_ent = Entity::PLACEHOLDER;
            let mut card_ents = Vec::new();

            commands.entity(wrapper_entity).with_children(|parent| {
                let (t, s, h, c) = build_work_content_inner(
                    parent,
                    &assets,
                    &localization,
                    settings.language,
                    &player,
                    slider_state.0,
                );
                track_ent = t;
                stage_ents = s;
                handle_ent = h;
                card_ents = c;
            });

            commands.entity(track_ent).observe(handle_work_slider_clicks_track);
            for stage in stage_ents {
                commands.entity(stage).observe(handle_work_slider_clicks);
            }
            commands
                .entity(handle_ent)
                .observe(handle_work_slider_drag)
                .observe(handle_work_slider_release);

            for card in card_ents {
                commands.entity(card).observe(crate::core::actions::handle_work_card_clicks);
            }
        }
    }
}

pub fn build_work_content(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    slider_val: u32,
) -> (Entity, Vec<Entity>, Entity, Vec<Entity>) {
    let mut track_ent = Entity::PLACEHOLDER;
    let mut stage_ents = Vec::new();
    let mut handle_ent = Entity::PLACEHOLDER;
    let mut card_ents = Vec::new();

    parent
        .spawn((
            Node {
                width: percent(100.),
                height: percent(100.),
                padding: UiRect::all(percent(5.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            WorkContentWrapper,
        ))
        .with_children(|parent| {
            let (t, s, h, c) =
                build_work_content_inner(parent, assets, localization, lang, player, slider_val);
            track_ent = t;
            stage_ents = s;
            handle_ent = h;
            card_ents = c;
        });

    (track_ent, stage_ents, handle_ent, card_ents)
}

pub fn build_work_content_inner(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    slider_val: u32,
) -> (Entity, Vec<Entity>, Entity, Vec<Entity>) {
    let mut track_ent = Entity::PLACEHOLDER;
    let mut stage_ents = Vec::new();
    let mut handle_ent = Entity::PLACEHOLDER;
    let mut card_ents = Vec::new();

    // Calculate gold ranges
    let slider_mult = [1.0, 2.5, 4.0][slider_val as usize];

    // Card 1 Ranges
    let base_clerical =
        (1.0 + player.charisma_mod() as f32) * (player.level as f32).powf(1.2) * 2.0 * slider_mult;
    let min_clerical = (base_clerical * 0.8).max(1.0) as u32;
    let max_clerical = (base_clerical * 1.2).max(2.0) as u32;

    // Card 2 Ranges
    let base_craft =
        (1.0 + player.charisma_mod() as f32) * (player.level as f32).powf(1.2) * 2.5 * slider_mult;
    let min_craft = (base_craft * 0.8).max(1.0) as u32;
    let max_craft = (base_craft * 1.2).max(2.0) as u32;

    // Card 3 Ranges
    let base_manual =
        (1.0 + player.charisma_mod() as f32) * (player.level as f32).powf(1.2) * 3.5 * slider_mult;
    let min_manual = (base_manual * 0.8).max(1.0) as u32;
    let max_manual = (base_manual * 1.2).max(2.0) as u32;

    let ap_cost = slider_val + 1;

    // Fixed costs calculations:
    let craft_percentage =
        (5.0 + player.level as f32 * 0.5 - player.charisma_mod() as f32).max(1.0);
    let craft_cost =
        ((craft_percentage / 100.0) * player.max_mana() as f32 * slider_mult).max(1.0) as u32;

    let manual_percentage =
        (7.0 + player.level as f32 * 0.5 - player.charisma_mod() as f32).max(1.0);
    let manual_cost =
        ((manual_percentage / 100.0) * player.max_health() as f32 * slider_mult).max(1.0) as u32;

    // Top Row
    parent
        .spawn(Node {
            width: percent(100.),
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
                add_text(localization.get("work", lang), "bold", 3.6, assets),
                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
            ));

            // Center: Slider
            let (t, s, h) = spawn_intensity_slider(
                parent,
                assets,
                localization,
                lang,
                slider_val,
                3,
                &["light", "regular", "heavy"],
                WorkSliderTrack,
                WorkSliderHandle,
                WorkSliderValueNode,
                WorkSliderValueText,
                WorkSliderStageButton,
            );
            track_ent = t;
            stage_ents = s;
            handle_ent = h;

            // Right: AP + Gold
            parent
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(30.),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.),
                    ..default()
                },))
                .with_children(|parent| {
                    // AP icon + text
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.),
                                ..default()
                            },
                            Interaction::default(),
                            Pickable::default(),
                            crate::core::ui::playing::InfoTooltip::ActionPoints,
                        ))
                        .with_children(|parent| {
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
                                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                            ));
                        });

                    // Gold icon + text
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.),
                                ..default()
                            },
                            Interaction::default(),
                            Pickable::default(),
                            crate::core::ui::playing::InfoTooltip::Gold,
                        ))
                        .with_children(|parent| {
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
                                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                            ));
                        });
                });
        });

    // Center Cards Row
    parent
        .spawn(Node {
            width: percent(100.),
            height: percent(78.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(20.),
            margin: UiRect::top(Val::Px(15.)),
            ..default()
        })
        .with_children(|parent| {
            // Card 1: Clerical Labor
            let title1 = localization.get("clerical_labor_title", lang);
            let desc1_raw = localization
                .get("clerical_labor_desc", lang)
                .replace("{min}", &min_clerical.to_string())
                .replace("{max}", &max_clerical.to_string());
            let desc1 = desc1_raw.split('\n').next().unwrap().to_string();
            let c1 = spawn_card_ui(
                parent,
                assets,
                &title1,
                &desc1,
                "action_clerical_labor",
                Some(ap_cost),
                None,
                WorkCardMarker(0),
            );
            card_ents.push(c1);

            // Card 2: Craft Labor
            let title2 = localization.get("craft_labor_title", lang);
            let desc2_raw = localization
                .get("craft_labor_desc", lang)
                .replace("{min}", &min_craft.to_string())
                .replace("{max}", &max_craft.to_string());
            let desc2 = desc2_raw.split('\n').next().unwrap().to_string();
            let c2 = spawn_card_ui(
                parent,
                assets,
                &title2,
                &desc2,
                "action_craft_labor",
                Some(ap_cost),
                Some((craft_cost, "mana", Color::srgb(40. / 255., 80. / 255., 185. / 255.))),
                WorkCardMarker(1),
            );
            card_ents.push(c2);

            // Card 3: Manual Labor
            let title3 = localization.get("manual_labor_title", lang);
            let desc3_raw = localization
                .get("manual_labor_desc", lang)
                .replace("{min}", &min_manual.to_string())
                .replace("{max}", &max_manual.to_string());
            let desc3 = desc3_raw.split('\n').next().unwrap().to_string();
            let c3 = spawn_card_ui(
                parent,
                assets,
                &title3,
                &desc3,
                "action_manual_labor",
                Some(ap_cost),
                Some((manual_cost, "health", Color::srgb(170. / 255., 35. / 255., 35. / 255.))),
                WorkCardMarker(2),
            );
            card_ents.push(c3);
        });

    (track_ent, stage_ents, handle_ent, card_ents)
}

// Handle work slider interaction
pub fn handle_work_slider_clicks(
    event: On<Pointer<Click>>,
    stage_q: Query<&WorkSliderStageButton>,
    mut slider_state: ResMut<WorkSliderState>,
) {
    if let Ok(btn) = stage_q.get(event.entity) {
        slider_state.0 = btn.0;
    }
}

pub fn handle_work_slider_clicks_track(
    _event: On<Pointer<Click>>,
    track_q: Query<&GlobalTransform, With<WorkSliderTrack>>,
    windows: Query<&Window>,
    mut slider_state: ResMut<WorkSliderState>,
) {
    let Ok(track_transform) = track_q.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let relative_x = slider_relative_x_from_cursor(track_transform, window, cursor_pos.x);
    let stage = slider_stage_from_relative_x(relative_x, 3);
    slider_state.0 = stage;
}

pub fn handle_work_slider_drag(
    ev: On<Pointer<Drag>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut handle_q: Query<&mut Node, (With<WorkSliderHandle>, Without<WorkSliderTrack>)>,
    mut value_node_q: Query<
        &mut Node,
        (With<WorkSliderValueNode>, Without<WorkSliderHandle>, Without<WorkSliderTrack>),
    >,
    mut text_q: Query<&mut Text, With<WorkSliderValueText>>,
) {
    let current_left = {
        let Ok(handle_node) = handle_q.single_mut() else {
            return;
        };
        match handle_node.left {
            Val::Px(px) => px,
            _ => -12.,
        }
    };
    let relative_x = (current_left + 12. + ev.delta.x).clamp(0., SLIDER_WIDTH);
    if let Ok(mut h) = handle_q.single_mut() {
        if let Ok(mut v) = value_node_q.single_mut() {
            update_slider_visuals(relative_x, &mut h, &mut v);
        }
    }
    let stage = slider_stage_from_relative_x(relative_x, 3);
    if let Ok(mut text) = text_q.single_mut() {
        let stage_names = ["light", "regular", "heavy"];
        text.0 = localization.get(stage_names[stage as usize], settings.language);
    }
}

pub fn handle_work_slider_release(
    _ev: On<Pointer<DragEnd>>,
    handle_q: Query<&Node, (With<WorkSliderHandle>, Without<WorkSliderTrack>)>,
    mut slider_state: ResMut<WorkSliderState>,
) {
    let Ok(handle_node) = handle_q.single() else {
        return;
    };
    let relative_x = match handle_node.left {
        Val::Px(px) => px + 12.,
        _ => 0.0,
    };
    let stage = slider_stage_from_relative_x(relative_x, 3);
    slider_state.0 = stage;
}
