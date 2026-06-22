use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::states::GameState;
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::playing::{InfoTooltip, PlayingStat};
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::core::ui::utils::*;
use bevy::prelude::*;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub struct TrainSliderState(pub u32); // 0 = Offense, 1 = Defense, 2 = Tactical

#[derive(Component)]
pub struct TrainContentWrapper;

#[derive(Component)]
pub struct TrainSliderTrack;
#[derive(Component)]
pub struct TrainSliderHandle;
#[derive(Component)]
pub struct TrainSliderValueNode;
#[derive(Component)]
pub struct TrainSliderValueText;

#[derive(Component)]
pub struct TrainSliderStageButton(pub u32);
impl From<u32> for TrainSliderStageButton {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

#[derive(Component)]
pub struct TrainCardMarker(pub u32); // 0 = Melee, 1 = Finesse, 2 = Range

pub fn setup_train_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    slider_state: Res<TrainSliderState>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_train");
        let mut track_ent = Entity::PLACEHOLDER;
        let mut stage_ents = Vec::new();
        let mut handle_ent = Entity::PLACEHOLDER;
        let mut card_ents = Vec::new();

        commands.entity(panel_entity).with_children(|parent| {
            let (t, s, h, c) = build_train_content(
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

        commands.entity(track_ent).observe(handle_train_slider_clicks_track);
        for stage in stage_ents {
            commands.entity(stage).observe(handle_train_slider_clicks);
        }
        commands
            .entity(handle_ent)
            .observe(handle_train_slider_drag)
            .observe(handle_train_slider_release);

        for card in card_ents {
            commands.entity(card).observe(handle_train_card_clicks);
        }
    }
}

pub fn update_train_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    slider_state: Res<TrainSliderState>,
    wrapper_q: Query<Entity, With<TrainContentWrapper>>,
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
                let (t, s, h, c) = build_train_content_inner(
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

            commands.entity(track_ent).observe(handle_train_slider_clicks_track);
            for stage in stage_ents {
                commands.entity(stage).observe(handle_train_slider_clicks);
            }
            commands
                .entity(handle_ent)
                .observe(handle_train_slider_drag)
                .observe(handle_train_slider_release);

            for card in card_ents {
                commands.entity(card).observe(handle_train_card_clicks);
            }
        }
    }
}

pub fn build_train_content(
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
            TrainContentWrapper,
        ))
        .with_children(|parent| {
            let (t, s, h, c) =
                build_train_content_inner(parent, assets, localization, lang, player, slider_val);
            track_ent = t;
            stage_ents = s;
            handle_ent = h;
            card_ents = c;
        });

    (track_ent, stage_ents, handle_ent, card_ents)
}

pub fn build_train_content_inner(
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

    let ap_cost = 1;
    let hp_cost = 10 * player.level();
    let mp_cost = 10 * player.level();

    let skill_key = match slider_val {
        0 => "general.attack",
        1 => "general.defense",
        2 => "general.initiative",
        _ => "",
    };
    let skill_name = localization.get(skill_key, lang);

    let melee_bonus = match slider_val {
        0 => player.training.melee.attack,
        1 => player.training.melee.defense,
        2 => player.training.melee.initiative,
        _ => 0,
    };
    let finesse_bonus = match slider_val {
        0 => player.training.finesse.attack,
        1 => player.training.finesse.defense,
        2 => player.training.finesse.initiative,
        _ => 0,
    };
    let range_bonus = match slider_val {
        0 => player.training.range.attack,
        1 => player.training.range.defense,
        2 => player.training.range.initiative,
        _ => 0,
    };

    let melee_gold_cost = 10 + 10 * melee_bonus;
    let finesse_gold_cost = 10 + 10 * finesse_bonus;
    let range_gold_cost = 10 + 10 * range_bonus;

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
                add_text(localization.get("general.train", lang), "bold", 3.6, assets),
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
                &["offense_training", "defense_training", "tactical_training"],
                TrainSliderTrack,
                TrainSliderHandle,
                TrainSliderValueNode,
                TrainSliderValueText,
                TrainSliderStageButton,
            );
            track_ent = t;
            stage_ents = s;
            handle_ent = h;

            // Right: Active Stat, Gold, AP
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(30.),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.),
                    ..default()
                })
                .with_children(|parent| {
                    // Item 1: Active Stat
                    let (stat_icon, stat_val, stat_tooltip) = match slider_val {
                        0 => ("attack", player.attack(), InfoTooltip::Combat(PlayingStat::Attack)),
                        1 => {
                            ("defense", player.defense(), InfoTooltip::Combat(PlayingStat::Defense))
                        },
                        _ => (
                            "initiative",
                            player.initiative(),
                            InfoTooltip::Combat(PlayingStat::Initiative),
                        ),
                    };
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
                            stat_tooltip,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    width: Val::Vw(2.4),
                                    height: Val::Vw(2.4),
                                    ..default()
                                },
                                ImageNode::new(assets.image(stat_icon))
                                    .with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text(stat_val.to_string(), "bold", 2.4, assets),
                                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                            ));
                        });

                    // Item 2: Gold
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
                            InfoTooltip::Gold,
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

                    // Item 3: AP
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
                            InfoTooltip::ActionPoints,
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
            // Card 1: Melee / Combat Training
            let title1 = localization.get("general.offense_training_title", lang);
            let desc1 = localization
                .get("general.melee_training_desc", lang)
                .replace("{skill}", &skill_name);
            let c1 = spawn_card_ui(
                parent,
                assets,
                &title1,
                &desc1,
                "action_melee",
                Some(ap_cost),
                Some((hp_cost, "health", Color::srgb(0.85, 0.20, 0.20))),
                Some(melee_gold_cost),
                TrainCardMarker(0),
            );
            card_ents.push(c1);

            // Card 2: Finesse / Martial Training
            let title2 = localization.get("general.martial_training_title", lang);
            let desc2 = localization
                .get("general.finesse_training_desc", lang)
                .replace("{skill}", &skill_name);
            let c2 = spawn_card_ui(
                parent,
                assets,
                &title2,
                &desc2,
                "action_finesse",
                Some(ap_cost),
                Some((hp_cost, "health", Color::srgb(0.85, 0.20, 0.20))),
                Some(finesse_gold_cost),
                TrainCardMarker(1),
            );
            card_ents.push(c2);

            // Card 3: Ranged / Precision Training
            let title3 = localization.get("general.precision_training_title", lang);
            let desc3 = localization
                .get("general.range_training_desc", lang)
                .replace("{skill}", &skill_name);
            let c3 = spawn_card_ui(
                parent,
                assets,
                &title3,
                &desc3,
                "action_range",
                Some(ap_cost),
                Some((mp_cost, "mana", Color::srgb(0.20, 0.55, 0.90))),
                Some(range_gold_cost),
                TrainCardMarker(2),
            );
            card_ents.push(c3);
        });

    (track_ent, stage_ents, handle_ent, card_ents)
}

pub fn handle_train_slider_clicks(
    event: On<Pointer<Click>>,
    stage_q: Query<&TrainSliderStageButton>,
    mut slider_state: ResMut<TrainSliderState>,
) {
    if let Ok(btn) = stage_q.get(event.entity) {
        slider_state.0 = btn.0;
    }
}

pub fn handle_train_slider_clicks_track(
    _event: On<Pointer<Click>>,
    track_q: Query<&GlobalTransform, With<TrainSliderTrack>>,
    windows: Query<&Window>,
    mut slider_state: ResMut<TrainSliderState>,
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

pub fn handle_train_slider_drag(
    ev: On<Pointer<Drag>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut handle_q: Query<&mut Node, (With<TrainSliderHandle>, Without<TrainSliderTrack>)>,
    mut value_node_q: Query<
        &mut Node,
        (With<TrainSliderValueNode>, Without<TrainSliderHandle>, Without<TrainSliderTrack>),
    >,
    mut text_q: Query<&mut Text, With<TrainSliderValueText>>,
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
        let stage_names = ["offense_training", "defense_training", "tactical_training"];
        text.0 = localization.get(stage_names[stage as usize], settings.language);
    }
}

pub fn handle_train_slider_release(
    _ev: On<Pointer<DragEnd>>,
    handle_q: Query<&Node, (With<TrainSliderHandle>, Without<TrainSliderTrack>)>,
    mut slider_state: ResMut<TrainSliderState>,
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

pub fn handle_train_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    _level_up: ResMut<LevelUpPending>,
    card_q: Query<&TrainCardMarker>,
    slider_state: Res<TrainSliderState>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    _next_game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(marker) = card_q.get(event.entity) {
        let slider_val = slider_state.0;
        let ap_cost = 1;
        let lang = settings.language;
        let toast = toast_container_q.single().unwrap();

        // Get current bonus
        let current_bonus = match marker.0 {
            0 => match slider_val {
                0 => player.training.melee.attack,
                1 => player.training.melee.defense,
                2 => player.training.melee.initiative,
                _ => 0,
            },
            1 => match slider_val {
                0 => player.training.finesse.attack,
                1 => player.training.finesse.defense,
                2 => player.training.finesse.initiative,
                _ => 0,
            },
            2 => match slider_val {
                0 => player.training.range.attack,
                1 => player.training.range.defense,
                2 => player.training.range.initiative,
                _ => 0,
            },
            _ => 0,
        };

        // 2. Check Gold
        let gold_cost = 10 + 10 * current_bonus;
        if player.gold < gold_cost {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            spawn_toast(
                &mut commands,
                &assets,
                localization.get("not_enough_gold", lang),
                Color::srgba(0.20, 0.05, 0.05, 0.93),
                Color::srgb(0.85, 0.20, 0.20),
                Color::srgb(1.0, 0.80, 0.80),
                toast,
            );
            return;
        }

        // 3. Check health or mana cost
        let hp_cost = 10 * player.level();
        let mp_cost = 10 * player.level();

        match marker.0 {
            0 | 1 => {
                // Melee or Finesse: Health cost
                if player.health() <= hp_cost {
                    play_audio_msg.write(PlayAudioMsg::new("error"));
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization.get("not_enough_health", lang),
                        Color::srgba(0.20, 0.05, 0.05, 0.93),
                        Color::srgb(0.85, 0.20, 0.20),
                        Color::srgb(1.0, 0.80, 0.80),
                        toast,
                    );
                    return;
                }
                let next_hp = player.health() - hp_cost;
                player.set_health(next_hp);
            },
            2 => {
                // Ranged: Mana cost
                if player.mana() < mp_cost {
                    play_audio_msg.write(PlayAudioMsg::new("error"));
                    spawn_toast(
                        &mut commands,
                        &assets,
                        localization.get("not_enough_mana", lang),
                        Color::srgba(0.20, 0.05, 0.05, 0.93),
                        Color::srgb(0.85, 0.20, 0.20),
                        Color::srgb(1.0, 0.80, 0.80),
                        toast,
                    );
                    return;
                }
                let next_mp = player.mana() - mp_cost;
                player.set_mana(next_mp);
            },
            _ => {},
        }

        // 4. Pay gold cost
        player.gold -= gold_cost;

        // 5. Apply training bonus
        match marker.0 {
            0 => match slider_val {
                0 => player.training.melee.attack += 1,
                1 => player.training.melee.defense += 1,
                2 => player.training.melee.initiative += 1,
                _ => {},
            },
            1 => match slider_val {
                0 => player.training.finesse.attack += 1,
                1 => player.training.finesse.defense += 1,
                2 => player.training.finesse.initiative += 1,
                _ => {},
            },
            2 => match slider_val {
                0 => player.training.range.attack += 1,
                1 => player.training.range.defense += 1,
                2 => player.training.range.initiative += 1,
                _ => {},
            },
            _ => {},
        }

        // 6. Play sound
        play_audio_msg.write(PlayAudioMsg::new("train"));

        // 7. Success Toast
        let skill_key = match slider_val {
            0 => "general.attack",
            1 => "general.defense",
            2 => "general.initiative",
            _ => "",
        };
        let weapon_key = match marker.0 {
            0 => "general.melee",
            1 => "general.finesse",
            2 => "general.range",
            _ => "",
        };
        let skill_name = localization.get(skill_key, lang);
        let weapon_name = localization.get(weapon_key, lang);
        let success_text = localization
            .get("general.toast_training_succeeded", lang)
            .replace("{skill}", &skill_name)
            .replace("{weapon}", &weapon_name);

        spawn_toast(
            &mut commands,
            &assets,
            success_text,
            Color::srgba(0.08, 0.10, 0.20, 0.93),
            Color::srgb(0.35, 0.55, 0.90),
            Color::srgb(0.75, 0.90, 1.0),
            toast,
        );

        // 8. AP Cost
        player.ap += ap_cost;
    }
}
