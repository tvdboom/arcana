use crate::core::assets::WorldAssets;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::utils::*;
use bevy::prelude::*;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub struct StudySliderState(pub u32); // 0 = Light, 1 = Regular, 2 = Heavy

#[derive(Component)]
pub struct StudyContentWrapper;

#[derive(Component)]
pub struct StudySliderTrack;
#[derive(Component)]
pub struct StudySliderHandle;
#[derive(Component)]
pub struct StudySliderValueNode;
#[derive(Component)]
pub struct StudySliderValueText;

#[derive(Component)]
pub struct StudySliderStageButton(pub u32);
impl From<u32> for StudySliderStageButton {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

#[derive(Component)]
pub struct StudyCardMarker(pub u32); // 0 = Apprenticeship, 1 = Mentorship, 2 = Conditioning

pub fn setup_study_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    slider_state: Res<StudySliderState>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_study");
        let mut track_ent = Entity::PLACEHOLDER;
        let mut stage_ents = Vec::new();
        let mut handle_ent = Entity::PLACEHOLDER;
        let mut card_ents = Vec::new();

        commands.entity(panel_entity).with_children(|parent| {
            let (t, s, h, c) = build_study_content(
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

        commands.entity(track_ent).observe(handle_study_slider_clicks_track);
        for stage in stage_ents {
            commands.entity(stage).observe(handle_study_slider_clicks);
        }
        commands
            .entity(handle_ent)
            .observe(handle_study_slider_drag)
            .observe(handle_study_slider_release);

        for card in card_ents {
            commands.entity(card).observe(crate::core::actions::handle_study_card_clicks);
        }
    }
}

pub fn update_study_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    slider_state: Res<StudySliderState>,
    wrapper_q: Query<Entity, With<StudyContentWrapper>>,
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
                let (t, s, h, c) = build_study_content_inner(
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

            commands.entity(track_ent).observe(handle_study_slider_clicks_track);
            for stage in stage_ents {
                commands.entity(stage).observe(handle_study_slider_clicks);
            }
            commands
                .entity(handle_ent)
                .observe(handle_study_slider_drag)
                .observe(handle_study_slider_release);

            for card in card_ents {
                commands.entity(card).observe(crate::core::actions::handle_study_card_clicks);
            }
        }
    }
}

pub fn build_study_content(
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
            StudyContentWrapper,
        ))
        .with_children(|parent| {
            let (t, s, h, c) =
                build_study_content_inner(parent, assets, localization, lang, player, slider_val);
            track_ent = t;
            stage_ents = s;
            handle_ent = h;
            card_ents = c;
        });

    (track_ent, stage_ents, handle_ent, card_ents)
}

pub fn build_study_content_inner(
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

    let ap_cost = slider_val + 1;
    let chance = 40 + player.intelligence_mod() * 5;

    // Calculate learning breakdown by target level
    let offset_probs: &[(i32, f32)] = match slider_val {
        0 => &[(-2, 0.40), (-1, 0.30), (0, 0.20), (1, 0.08), (2, 0.02)],
        1 => &[(-2, 0.15), (-1, 0.20), (0, 0.30), (1, 0.20), (2, 0.15)],
        _ => &[(-2, 0.02), (-1, 0.08), (0, 0.20), (1, 0.30), (2, 0.40)],
    };

    let mut level_probs = std::collections::BTreeMap::new();
    for &(offset, prob) in offset_probs {
        let target_level = (player.level as i32 + offset).clamp(1, 20) as u32;
        *level_probs.entry(target_level).or_insert(0.0) += prob;
    }

    let mut breakdown_rows = Vec::new();
    for (lvl, prob) in level_probs {
        let learn_chance = (chance as f32 * prob).round() as u32;
        breakdown_rows.push(format!(" - Level {}: {}%", lvl, learn_chance));
    }
    let breakdown = breakdown_rows.join("\n");

    let (attr_single, attr_plural) = match lang {
        Language::Dutch => ("attribuut", "attributen"),
        Language::Spanish => ("atributo", "atributos"),
        _ => ("attribute", "attributes"),
    };

    let cond_breakdown = match slider_val {
        0 => format!(" - 1 {}: {}%", attr_single, chance),
        1 => {
            let p = (chance as f32 * 0.5).round() as u32;
            format!(" - 1 {}: {}%\n - 2 {}: {}%", attr_single, p, attr_plural, p)
        },
        _ => {
            let p1 = (chance as f32 * 0.2).round() as u32;
            let p2 = (chance as f32 * 0.4).round() as u32;
            format!(
                " - 1 {}: {}%\n - 2 {}: {}%\n - 3 {}: {}%",
                attr_single, p1, attr_plural, p2, attr_plural, p2
            )
        }
    };

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
                add_text(localization.get("study", lang), "bold", 3.6, assets),
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
                &["basic", "intermediate", "advanced"],
                StudySliderTrack,
                StudySliderHandle,
                StudySliderValueNode,
                StudySliderValueText,
                StudySliderStageButton,
            );
            track_ent = t;
            stage_ents = s;
            handle_ent = h;

            // Right: AP only
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(30.),
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
                        ImageNode::new(assets.image("ap")).with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(player.ap.to_string(), "bold", 2.4, assets),
                        TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                    ));
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
            // Card 1: Apprenticeship
            let title1 = localization.get("apprenticeship_title", lang);
            let desc1 = localization
                .get("apprenticeship_desc", lang)
                .replace("{breakdown}", &breakdown);
            let c1 = spawn_card_ui(
                parent,
                assets,
                &title1,
                &desc1,
                "action_apprenticeship",
                Some(ap_cost),
                None,
                None,
                StudyCardMarker(0),
            );
            card_ents.push(c1);

            // Card 2: Mentorship
            let title2 = localization.get("mentorship_title", lang);
            let desc2 = localization
                .get("mentorship_desc", lang)
                .replace("{breakdown}", &breakdown);
            let c2 = spawn_card_ui(
                parent,
                assets,
                &title2,
                &desc2,
                "action_mentorship",
                Some(ap_cost),
                None,
                None,
                StudyCardMarker(1),
            );
            card_ents.push(c2);

            // Card 3: Conditioning
            let title3 = localization.get("conditioning_title", lang);
            let desc3 = localization
                .get("conditioning_desc", lang)
                .replace("{breakdown}", &cond_breakdown);
            let c3 = spawn_card_ui(
                parent,
                assets,
                &title3,
                &desc3,
                "action_conditioning",
                Some(ap_cost),
                None,
                None,
                StudyCardMarker(2),
            );
            card_ents.push(c3);
        });

    (track_ent, stage_ents, handle_ent, card_ents)
}

// Handle study slider interaction
pub fn handle_study_slider_clicks(
    event: On<Pointer<Click>>,
    stage_q: Query<&StudySliderStageButton>,
    mut slider_state: ResMut<StudySliderState>,
) {
    if let Ok(btn) = stage_q.get(event.entity) {
        slider_state.0 = btn.0;
    }
}

pub fn handle_study_slider_clicks_track(
    _event: On<Pointer<Click>>,
    track_q: Query<&GlobalTransform, With<StudySliderTrack>>,
    windows: Query<&Window>,
    mut slider_state: ResMut<StudySliderState>,
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

pub fn handle_study_slider_drag(
    ev: On<Pointer<Drag>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut handle_q: Query<&mut Node, (With<StudySliderHandle>, Without<StudySliderTrack>)>,
    mut value_node_q: Query<
        &mut Node,
        (With<StudySliderValueNode>, Without<StudySliderHandle>, Without<StudySliderTrack>),
    >,
    mut text_q: Query<&mut Text, With<StudySliderValueText>>,
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
        let stage_names = ["basic", "intermediate", "advanced"];
        text.0 = localization.get(stage_names[stage as usize], settings.language);
    }
}

pub fn handle_study_slider_release(
    _ev: On<Pointer<DragEnd>>,
    handle_q: Query<&Node, (With<StudySliderHandle>, Without<StudySliderTrack>)>,
    mut slider_state: ResMut<StudySliderState>,
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
