use crate::core::actions::gain_xp;
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::all_artifacts;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::states::GameState;
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::playing::InfoTooltip;
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::core::ui::utils::*;
use bevy::prelude::*;
use rand::prelude::IndexedRandom;
use rand::{rng, RngExt};

#[derive(Component)]
pub struct HuntContentWrapper;

#[derive(Component)]
pub struct HuntCardMarker(pub u32); // 0 = Easy, 1 = Wild, 2 = Deadly

#[derive(Resource, Default)]
pub struct PendingHuntXp {
    pub amount: u32,
}

pub fn apply_pending_hunt_xp(
    mut pending_hunt_xp: ResMut<PendingHuntXp>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if pending_hunt_xp.amount == 0 {
        return;
    }

    let amount = pending_hunt_xp.amount;
    pending_hunt_xp.amount = 0;
    gain_xp(
        &mut player,
        amount,
        &mut level_up,
        &mut play_audio_msg,
        &mut next_game_state,
    );
}

pub fn setup_hunt_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }

    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_hunt");
        let mut card_ents = Vec::new();
        commands.entity(panel_entity).with_children(|parent| {
            card_ents =
                build_hunt_content(parent, &assets, &localization, settings.language, &player);
        });
        for card in card_ents {
            commands.entity(card).observe(handle_hunt_card_clicks);
        }
    }
}

pub fn update_hunt_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    wrapper_q: Query<Entity, With<HuntContentWrapper>>,
    children_q: Query<&Children>,
) {
    if !player.is_changed() {
        return;
    }

    if let Some(wrapper_entity) = wrapper_q.iter().next() {
        despawn_descendants_manual(&mut commands, wrapper_entity, &children_q);
        let mut card_ents = Vec::new();
        commands.entity(wrapper_entity).with_children(|parent| {
            card_ents = build_hunt_content_inner(
                parent,
                &assets,
                &localization,
                settings.language,
                &player,
            );
        });
        for card in card_ents {
            commands.entity(card).observe(handle_hunt_card_clicks);
        }
    }
}

fn build_hunt_content(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) -> Vec<Entity> {
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
            HuntContentWrapper,
        ))
        .with_children(|parent| {
            card_ents = build_hunt_content_inner(parent, assets, localization, lang, player);
        });
    card_ents
}

fn build_hunt_content_inner(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) -> Vec<Entity> {
    let mut card_ents = Vec::new();

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
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(30.),
                    ..default()
                },
                add_text(localization.get("hunt", lang), "bold", 3.6, assets),
                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
            ));

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(30.),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(4.),
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
                        ImageNode::new(assets.image("ap")).with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(player.ap.to_string(), "bold", 2.4, assets),
                        TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
                    ));
                });
        });

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
            let title1 = localization.get("easy_hunt_title", lang);
            let desc1 = localization.get("general.easy_hunt_desc", lang);
            card_ents.push(spawn_hunt_card(
                parent,
                assets,
                &title1,
                &desc1,
                "action_easy_hunt",
                1,
                HuntCardMarker(0),
            ));

            let title2 = localization.get("wild_hunt_title", lang);
            let desc2 = localization.get("general.wild_hunt_desc", lang);
            card_ents.push(spawn_hunt_card(
                parent,
                assets,
                &title2,
                &desc2,
                "action_wild_hunt",
                2,
                HuntCardMarker(1),
            ));

            let title3 = localization.get("deadly_hunt_title", lang);
            let desc3 = localization.get("general.deadly_hunt_desc", lang);
            card_ents.push(spawn_hunt_card(
                parent,
                assets,
                &title3,
                &desc3,
                "action_deadly_hunt",
                3,
                HuntCardMarker(2),
            ));
        });

    card_ents
}

fn spawn_hunt_card<M: Component>(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    title: &str,
    description: &str,
    image_key: &str,
    ap_cost: u32,
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
                top: percent(-2.),
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
                });

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
                .observe(crate::core::utils::cursor::<Over>(
                    bevy::window::SystemCursorIcon::Pointer,
                ))
                .observe(crate::core::utils::cursor::<Out>(bevy::window::SystemCursorIcon::Default))
                .id();
        });

    border_entity
}

fn choose_hunting_artifact(tier: u32) -> Option<String> {
    let mut hunting_artifacts: Vec<_> = all_artifacts()
        .iter()
        .filter(|art| art.image.contains("/skinning_") || art.image.contains("\\skinning_"))
        .collect();
    if hunting_artifacts.is_empty() {
        return None;
    }

    hunting_artifacts.sort_by_key(|a| a.price);
    if hunting_artifacts.len() < 3 {
        let mut rng = rng();
        return hunting_artifacts.choose(&mut rng).map(|art| art.name.clone());
    }

    let len = hunting_artifacts.len();
    let low_end = ((len as f32) * 0.33).ceil() as usize;
    let high_start = ((len as f32) * 0.66).floor() as usize;
    let (start, end) = match tier {
        0 => (0, low_end.max(1).min(len)),
        1 => {
            let s = low_end.min(len.saturating_sub(1));
            let e = high_start.max(s + 1).min(len);
            (s, e)
        },
        _ => (high_start.min(len.saturating_sub(1)), len),
    };

    let mut rng = rng();
    hunting_artifacts[start..end].choose(&mut rng).map(|art| art.name.clone())
}

pub fn handle_hunt_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut pending_hunt_xp: ResMut<PendingHuntXp>,
    card_q: Query<&HuntCardMarker>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
) {
    let Ok(marker) = card_q.get(event.entity) else {
        return;
    };

    let lang = settings.language;
    let toast = toast_container_q.single().unwrap();
    let mut rng = rng();

    let (ap_gain, xp_gain, combat_chance, loot_chance, tier) = match marker.0 {
        0 => (1, 1, 0.10, 0.50, 0),
        1 => (2, 3, 0.30, 0.50, 1),
        _ => (3, 7, 0.70, 0.50, 2),
    };

    player.ap += ap_gain;
    let combat_triggered = rng.random_bool(combat_chance);

    let mut loot_found = None;
    if rng.random_bool(loot_chance) {
        if let Some(artifact_name) = choose_hunting_artifact(tier) {
            player.inventory.push(artifact_name.clone());
            loot_found = Some(artifact_name);
        }
    }

    if combat_triggered {
        let p_level = player.level();
        let (min_lvl, max_lvl) = match tier {
            0 => (p_level.saturating_sub(2).max(1), p_level),
            1 => (p_level.saturating_sub(1).max(1), p_level.saturating_add(1)),
            _ => (p_level, p_level.saturating_add(2)),
        };

        let possible: Vec<crate::core::monsters::Monster> =
            crate::core::catalog::catalog::all_monsters()
                .iter()
                .filter(|m| {
                    (m.is_from_image_dir("pets") || m.is_from_image_dir("dragons"))
                        && m.level >= min_lvl
                        && m.level <= max_lvl
                })
                .cloned()
                .collect();

        if !possible.is_empty() {
            pending_hunt_xp.amount = pending_hunt_xp.amount.saturating_add(xp_gain);
            let idx = rng.random_range(0..possible.len());
            let selected = possible[idx].clone();
            commands.insert_resource(crate::core::monsters::ActiveMonster { monster: selected });
            next_game_state.set(GameState::Combat);
            return;
        }
    }

    gain_xp(&mut player, xp_gain, &mut level_up, &mut play_audio_msg, &mut next_game_state);
    play_audio_msg.write(PlayAudioMsg::new("hunt"));

    if let Some(artifact_name) = loot_found {
        spawn_toast(
            &mut commands,
            &assets,
            localization.get("general.hunt_loot_found", lang).replace("{item}", &artifact_name),
            Color::srgba(0.08, 0.16, 0.12, 0.93),
            Color::srgb(0.25, 0.75, 0.50),
            Color::srgb(0.60, 1.0, 0.75),
            toast,
        );
    } else {
        spawn_toast(
            &mut commands,
            &assets,
            localization.get("general.hunt_no_loot", lang),
            Color::srgba(0.08, 0.10, 0.20, 0.93),
            Color::srgb(0.35, 0.55, 0.90),
            Color::srgb(0.75, 0.90, 1.0),
            toast,
        );
    }

    spawn_toast(
        &mut commands,
        &assets,
        localization.get("general.hunt_xp_gained", lang).replace("{xp}", &xp_gain.to_string()),
        Color::srgba(0.08, 0.10, 0.20, 0.93),
        Color::srgb(0.35, 0.55, 0.90),
        Color::srgb(0.75, 0.90, 1.0),
        toast,
    );
}
