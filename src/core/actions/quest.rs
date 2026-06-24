use crate::core::actions::gain_xp;
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::{all_artifacts, all_consumables, all_equipment, get_equipment};
use crate::core::catalog::equipment::Equipment;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::states::GameState;
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::playing::equip_item;
use crate::core::ui::playing::InfoTooltip;
use crate::core::ui::toast::{spawn_toast, ToastContainer};
use crate::core::ui::utils::*;
use crate::utils::capitalize_words;
use bevy::prelude::*;
use rand::{rng, RngExt};

#[derive(Component)]
pub struct QuestContentWrapper;

#[derive(Component)]
pub struct QuestCardMarker(pub u32); // 0 = Errand, 1 = Expedition, 2 = Odyssey

#[derive(Resource, Default)]
pub struct PendingQuestXp {
    pub amount: u32,
}

#[derive(Clone, Copy)]
struct QuestCardProfile {
    combat_chance: f64,
    xp_min: u32,
    xp_max: u32,
    gold_min: u32,
    gold_max: u32,
    equipment_min: u32,
    equipment_max: u32,
    consumable_min: u32,
    consumable_max: u32,
    artifact_min: u32,
    artifact_max: u32,
}

pub fn apply_pending_quest_xp(
    mut pending_quest_xp: ResMut<PendingQuestXp>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if pending_quest_xp.amount == 0 {
        return;
    }

    let amount = pending_quest_xp.amount;
    pending_quest_xp.amount = 0;
    gain_xp(
        &mut player,
        amount,
        &mut level_up,
        &mut play_audio_msg,
        &mut next_game_state,
    );
}

pub fn setup_quest_ui(
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
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_quest");
        let mut card_ents = Vec::new();
        commands.entity(panel_entity).with_children(|parent| {
            card_ents = build_quest_content(parent, &assets, &localization, settings.language, &player);
        });
        for card in card_ents {
            commands.entity(card).observe(handle_quest_card_clicks);
        }
    }
}

pub fn update_quest_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    wrapper_q: Query<Entity, With<QuestContentWrapper>>,
    children_q: Query<&Children>,
) {
    if !player.is_changed() {
        return;
    }

    if let Some(wrapper_entity) = wrapper_q.iter().next() {
        despawn_descendants_manual(&mut commands, wrapper_entity, &children_q);
        let mut card_ents = Vec::new();
        commands.entity(wrapper_entity).with_children(|parent| {
            card_ents =
                build_quest_content_inner(parent, &assets, &localization, settings.language, &player);
        });
        for card in card_ents {
            commands.entity(card).observe(handle_quest_card_clicks);
        }
    }
}

fn quest_profile(tier: u32) -> QuestCardProfile {
    match tier {
        0 => QuestCardProfile {
            combat_chance: 0.20,
            xp_min: 0,
            xp_max: 3,
            gold_min: 4,
            gold_max: 14,
            equipment_min: 0,
            equipment_max: 1,
            consumable_min: 0,
            consumable_max: 1,
            artifact_min: 0,
            artifact_max: 1,
        },
        1 => QuestCardProfile {
            combat_chance: 0.40,
            xp_min: 2,
            xp_max: 5,
            gold_min: 10,
            gold_max: 28,
            equipment_min: 0,
            equipment_max: 2,
            consumable_min: 0,
            consumable_max: 2,
            artifact_min: 0,
            artifact_max: 1,
        },
        _ => QuestCardProfile {
            combat_chance: 0.80,
            xp_min: 3,
            xp_max: 9,
            gold_min: 20,
            gold_max: 60,
            equipment_min: 1,
            equipment_max: 3,
            consumable_min: 1,
            consumable_max: 3,
            artifact_min: 1,
            artifact_max: 2,
        },
    }
}

fn quest_level_range(tier: u32, player_level: u32) -> (u32, u32) {
    match tier {
        0 => (player_level.saturating_sub(2).max(1), player_level.saturating_add(1).max(1)),
        1 => (player_level.saturating_sub(1).max(1), player_level.saturating_add(3).max(1)),
        _ => (player_level.max(1), player_level.saturating_add(5).max(1)),
    }
}

fn roll_count(rng: &mut impl rand::Rng, min: u32, max: u32) -> u32 {
    if min >= max {
        min
    } else {
        rng.random_range(min..=max)
    }
}

fn sample_names(pool: Vec<String>, count: u32, rng: &mut impl rand::Rng) -> Vec<String> {
    let mut rewards = Vec::new();
    let mut remaining = pool;

    for _ in 0..count {
        if remaining.is_empty() {
            break;
        }
        let idx = rng.random_range(0..remaining.len());
        rewards.push(remaining.remove(idx));
    }

    rewards
}

fn quest_equipment_rewards(tier: u32, player_level: u32, count: u32, rng: &mut impl rand::Rng) -> Vec<String> {
    let (min_level, max_level) = quest_level_range(tier, player_level);
    let mut pool: Vec<String> = all_equipment()
        .iter()
        .filter(|item| matches!(item, Equipment::Weapon(_) | Equipment::Wearable(_)))
        .filter(|item| item.level() >= min_level && item.level() <= max_level)
        .map(|item| item.name().to_string())
        .collect();

    if pool.is_empty() {
        pool = all_equipment()
            .iter()
            .filter(|item| matches!(item, Equipment::Weapon(_) | Equipment::Wearable(_)))
            .map(|item| item.name().to_string())
            .collect();
    }

    sample_names(pool, count, rng)
}

fn quest_consumable_rewards(
    tier: u32,
    player_level: u32,
    count: u32,
    rng: &mut impl rand::Rng,
) -> Vec<String> {
    let (min_level, max_level) = quest_level_range(tier, player_level);
    let mut pool: Vec<String> = all_consumables()
        .iter()
        .filter(|item| item.level >= min_level && item.level <= max_level)
        .map(|item| item.name.to_string())
        .collect();

    if pool.is_empty() {
        pool = all_consumables().iter().map(|item| item.name.to_string()).collect();
    }

    sample_names(pool, count, rng)
}

fn quest_artifact_rewards(
    tier: u32,
    player_level: u32,
    count: u32,
    rng: &mut impl rand::Rng,
) -> Vec<String> {
    let (min_level, max_level) = quest_level_range(tier, player_level);
    let mut pool: Vec<String> = all_artifacts()
        .iter()
        .filter(|item| item.level >= min_level && item.level <= max_level)
        .map(|item| item.name.to_string())
        .collect();

    if pool.is_empty() {
        pool = all_artifacts().iter().map(|item| item.name.to_string()).collect();
    }

    sample_names(pool, count, rng)
}

fn can_auto_equip(player: &Player, equipment: &Equipment) -> bool {
    match equipment {
        Equipment::Wearable(w) => match w.slot {
            crate::core::catalog::wearables::WearableSlot::Helmet => player.helmet.is_none(),
            crate::core::catalog::wearables::WearableSlot::Chestplate => player.armor.is_none(),
            crate::core::catalog::wearables::WearableSlot::Gloves => player.gloves.is_none(),
            crate::core::catalog::wearables::WearableSlot::Boots => player.boots.is_none(),
            crate::core::catalog::wearables::WearableSlot::Accessory => {
                player.accessory.is_none() || player.accessory2.is_none()
            },
        },
        Equipment::Weapon(w) => {
            if w.hand == crate::core::catalog::weapons::Hand::TwoHand {
                player.weapon_lh.is_none() && player.weapon_rh.is_none()
            } else if matches!(
                w.category,
                crate::core::catalog::weapons::Category::Shield
                    | crate::core::catalog::weapons::Category::Book
            ) {
                player.weapon_rh.is_none()
            } else {
                player.weapon_lh.is_none() || player.weapon_rh.is_none()
            }
        },
        _ => false,
    }
}

fn grant_equipment_reward(player: &mut Player, key: String) -> bool {
    if let Some(equipment) = get_equipment(&key) {
        if can_auto_equip(player, &equipment) {
            equip_item(player, &key);
            return true;
        }
    }

    player.inventory.push(key);
    false
}

fn build_quest_content(
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
            QuestContentWrapper,
        ))
        .with_children(|parent| {
            card_ents = build_quest_content_inner(parent, assets, localization, lang, player);
        });
    card_ents
}

fn build_quest_content_inner(
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
                add_text(localization.get("quest", lang), "bold", 3.6, assets),
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
            let title1 = localization.get("general.errand_title", lang);
            let desc1 = localization.get("general.errand_desc", lang);
            card_ents.push(spawn_quest_card(
                parent,
                assets,
                &title1,
                &desc1,
                "action_errand",
                1,
                QuestCardMarker(0),
            ));

            let title2 = localization.get("general.expedition_title", lang);
            let desc2 = localization.get("general.expedition_desc", lang);
            card_ents.push(spawn_quest_card(
                parent,
                assets,
                &title2,
                &desc2,
                "action_expedition",
                2,
                QuestCardMarker(1),
            ));

            let title3 = localization.get("general.odyssey_title", lang);
            let desc3 = localization.get("general.odyssey_desc", lang);
            card_ents.push(spawn_quest_card(
                parent,
                assets,
                &title3,
                &desc3,
                "action_odyssey",
                3,
                QuestCardMarker(2),
            ));
        });

    card_ents
}

fn spawn_quest_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    title: &str,
    description: &str,
    image_key: &str,
    ap_cost: u32,
    marker: QuestCardMarker,
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
                                ImageNode::new(assets.image("ap")).with_mode(NodeImageMode::Stretch),
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
                .observe(crate::core::utils::cursor::<Out>(
                    bevy::window::SystemCursorIcon::Default,
                ))
                .id();
        });

    border_entity
}

pub fn handle_quest_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut pending_quest_xp: ResMut<PendingQuestXp>,
    card_q: Query<&QuestCardMarker>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
) {
    let Ok(marker) = card_q.get(event.entity) else {
        return;
    };

    let lang = settings.language;
    let toast = toast_container_q.single().unwrap();
    let mut rng = rng();
    let profile = quest_profile(marker.0);

    player.ap += 1 + marker.0;

    let combat_triggered = rng.random_bool(profile.combat_chance);
    let xp_gain = rng.random_range(profile.xp_min..=profile.xp_max);
    let gold_gain = rng.random_range(profile.gold_min..=profile.gold_max);

    let equipment_count = roll_count(&mut rng, profile.equipment_min, profile.equipment_max);
    let consumable_count = roll_count(&mut rng, profile.consumable_min, profile.consumable_max);
    let artifact_count = roll_count(&mut rng, profile.artifact_min, profile.artifact_max);

    let mut loot_found = false;

    if gold_gain > 0 {
        player.gold += gold_gain;
        loot_found = true;
        spawn_toast(
            &mut commands,
            &assets,
            localization
                .get("general.toast_gold_earned", lang)
                .replace("{gold}", &gold_gain.to_string()),
            Color::srgba(0.08, 0.16, 0.12, 0.93),
            Color::srgb(0.25, 0.75, 0.50),
            Color::srgb(0.60, 1.0, 0.75),
            toast,
        );
    }

    for reward in quest_equipment_rewards(marker.0, player.level(), equipment_count, &mut rng) {
        loot_found = true;
        let auto_equipped = grant_equipment_reward(&mut player, reward.clone());
        spawn_toast(
            &mut commands,
            &assets,
            if auto_equipped {
                format!("Quest equipment found and equipped: {}", capitalize_words(&reward))
            } else {
                format!("Quest equipment found: {}", capitalize_words(&reward))
            },
            Color::srgba(0.08, 0.16, 0.12, 0.93),
            Color::srgb(0.25, 0.75, 0.50),
            Color::srgb(0.60, 1.0, 0.75),
            toast,
        );
    }

    for reward in quest_consumable_rewards(marker.0, player.level(), consumable_count, &mut rng) {
        loot_found = true;
        player.inventory.push(reward.clone());
        spawn_toast(
            &mut commands,
            &assets,
            format!("Quest consumable found: {}", capitalize_words(&reward)),
            Color::srgba(0.08, 0.16, 0.12, 0.93),
            Color::srgb(0.25, 0.75, 0.50),
            Color::srgb(0.60, 1.0, 0.75),
            toast,
        );
    }

    for reward in quest_artifact_rewards(marker.0, player.level(), artifact_count, &mut rng) {
        loot_found = true;
        player.inventory.push(reward.clone());
        spawn_toast(
            &mut commands,
            &assets,
            format!("Quest artifact found: {}", capitalize_words(&reward)),
            Color::srgba(0.08, 0.16, 0.12, 0.93),
            Color::srgb(0.25, 0.75, 0.50),
            Color::srgb(0.60, 1.0, 0.75),
            toast,
        );
    }

    if xp_gain > 0 && !combat_triggered {
        gain_xp(
            &mut player,
            xp_gain,
            &mut level_up,
            &mut play_audio_msg,
            &mut next_game_state,
        );
        spawn_toast(
            &mut commands,
            &assets,
            format!("Quest complete. +{} XP", xp_gain),
            Color::srgba(0.08, 0.10, 0.20, 0.93),
            Color::srgb(0.35, 0.55, 0.90),
            Color::srgb(0.75, 0.90, 1.0),
            toast,
        );
    }

    if loot_found {
        play_audio_msg.write(PlayAudioMsg::new("quest"));
    } else {
        spawn_toast(
            &mut commands,
            &assets,
            "Quest yielded no loot this time.".to_string(),
            Color::srgba(0.08, 0.10, 0.20, 0.93),
            Color::srgb(0.35, 0.55, 0.90),
            Color::srgb(0.75, 0.90, 1.0),
            toast,
        );
    }

    let mut combat_encounter_selected = false;
    if combat_triggered {
        let p_level = player.level();
        let tier = marker.0;
        let (min_lvl, max_lvl) = match tier {
            0 => (p_level.saturating_sub(2).max(1), p_level),
            1 => (p_level.saturating_sub(1).max(1), p_level.saturating_add(1)),
            _ => (p_level, p_level.saturating_add(2)),
        };

        let possible: Vec<crate::core::monsters::Monster> =
            crate::core::catalog::catalog::all_monsters()
                .iter()
                .filter(|m| {
                    (m.is_from_image_dir("creatures") || m.is_from_image_dir("dragons"))
                        && m.level >= min_lvl
                        && m.level <= max_lvl
                })
                .cloned()
                .collect();

        if !possible.is_empty() {
            pending_quest_xp.amount = pending_quest_xp.amount.saturating_add(xp_gain);
            let idx = rng.random_range(0..possible.len());
            let selected = possible[idx].clone();
            commands.insert_resource(crate::core::monsters::ActiveMonster { monster: selected });
            next_game_state.set(GameState::Combat);
            combat_encounter_selected = true;
        }
    }

    if combat_triggered && !combat_encounter_selected && xp_gain > 0 {
        gain_xp(
            &mut player,
            xp_gain,
            &mut level_up,
            &mut play_audio_msg,
            &mut next_game_state,
        );
        spawn_toast(
            &mut commands,
            &assets,
            format!("Quest complete. +{} XP", xp_gain),
            Color::srgba(0.08, 0.10, 0.20, 0.93),
            Color::srgb(0.35, 0.55, 0.90),
            Color::srgb(0.75, 0.90, 1.0),
            toast,
        );
    }
}
