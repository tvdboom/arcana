use crate::core::actions::trigger_level_up;
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_text, reimage};
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::states::GameState;
use crate::core::ui::level_up::LevelUpPending;
use crate::core::ui::playing::ToastContainer;
use crate::core::ui::toast::spawn_toast;
use crate::core::ui::utils::*;
use crate::core::utils::cursor;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use rand::{rng, RngExt};

#[derive(Component)]
pub struct RestContentWrapper;

#[derive(Component)]
pub struct RestCardMarker(pub u32); // 0 = Rough Rest, 1 = Common Lodging, 2 = Grand Accommodation

pub fn setup_rest_ui(
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
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_rest");
        let mut card_ents = Vec::new();

        commands.entity(panel_entity).with_children(|parent| {
            card_ents =
                build_rest_content(parent, &assets, &localization, settings.language, &player);
        });

        for card in card_ents {
            commands.entity(card).observe(handle_rest_card_clicks);
        }
    }
}

pub fn update_rest_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    wrapper_q: Query<Entity, With<RestContentWrapper>>,
    children_q: Query<&Children>,
) {
    if player.is_changed() {
        if let Some(wrapper_entity) = wrapper_q.iter().next() {
            despawn_descendants_manual(&mut commands, wrapper_entity, &children_q);
            let mut card_ents = Vec::new();

            commands.entity(wrapper_entity).with_children(|parent| {
                card_ents = build_rest_content_inner(
                    parent,
                    &assets,
                    &localization,
                    settings.language,
                    &player,
                );
            });

            for card in card_ents {
                commands.entity(card).observe(handle_rest_card_clicks);
            }
        }
    }
}

pub fn build_rest_content(
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
            RestContentWrapper,
        ))
        .with_children(|parent| {
            card_ents = build_rest_content_inner(parent, assets, localization, lang, player);
        });

    card_ents
}

pub fn build_rest_content_inner(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) -> Vec<Entity> {
    let mut card_ents = Vec::new();

    let level = player.level as u32;
    let common_gold = 10 * level;
    let grand_gold = 50 * level;

    // Top Row
    parent
        .spawn(Node {
            width: percent(100.),
            height: Val::Px(75.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
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
                add_text(localization.get("rest", lang), "bold", 3.6, assets),
                TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
            ));

            // Right: Resources Display (AP + Gold)
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(30.),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(15.),
                    ..default()
                })
                .with_children(|parent| {
                    // Gold Display
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.),
                            ..default()
                        })
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

                    // AP Display
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
            // Card 1: Rough Rest (Costs 1 AP)
            let title1 = localization.get("rough_rest", lang);
            let desc1 = localization.get("rough_rest_desc", lang);
            let c1 = spawn_rest_card_ui(
                parent,
                assets,
                &title1,
                &desc1,
                "action_simple_rest",
                Some(1),
                None,
                RestCardMarker(0),
            );
            card_ents.push(c1);

            // Card 2: Common Lodging (Costs 2 AP + 10 * level Gold)
            let title2 = localization.get("common_lodging", lang);
            let desc2 = localization.get("common_lodging_desc", lang);
            let c2 = spawn_rest_card_ui(
                parent,
                assets,
                &title2,
                &desc2,
                "action_common_lodging",
                Some(2),
                Some(common_gold),
                RestCardMarker(1),
            );
            card_ents.push(c2);

            // Card 3: Grand Accommodation (Costs 3 AP + 50 * level Gold)
            let title3 = localization.get("grand_accommodation", lang);
            let desc3 = localization.get("grand_accommodation_desc", lang);
            let c3 = spawn_rest_card_ui(
                parent,
                assets,
                &title3,
                &desc3,
                "action_grand_accommodation",
                Some(3),
                Some(grand_gold),
                RestCardMarker(2),
            );
            card_ents.push(c3);
        });

    card_ents
}

pub fn spawn_rest_card_ui<M: Component>(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    title: &str,
    description: &str,
    image_key: &str,
    ap_cost_opt: Option<u32>,
    gold_cost_opt: Option<u32>,
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

            // Costs Display Container in the Top-Right of the Card
            if ap_cost_opt.is_some() || gold_cost_opt.is_some() {
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
                        if let Some(gold_cost) = gold_cost_opt {
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
                                        ImageNode::new(assets.image("gold"))
                                            .with_mode(NodeImageMode::Stretch),
                                    ));
                                    parent.spawn((
                                        add_text(gold_cost.to_string(), "bold", 1.6, assets),
                                        TextColor(crate::core::constants::BUTTON_TEXT_COLOR),
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
                .observe(reimage::<Over>(assets.image("border_hover")))
                .observe(reimage::<Out>(assets.image("border")))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .observe(cursor::<Release>(SystemCursorIcon::Pointer))
                .id();
        });

    border_entity
}

// System to handle click on rest cards
pub fn handle_rest_card_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    card_q: Query<&RestCardMarker>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(marker) = card_q.get(event.entity) {
        let lang = settings.language;
        let toast = toast_container_q.single().unwrap();

        let level = player.level as u32;
        let ap_cost = match marker.0 {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => 1,
        };
        let gold_cost = match marker.0 {
            0 => 0,
            1 => 10 * level,
            2 => 50 * level,
            _ => 0,
        };

        if player.ap < ap_cost {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            spawn_toast(
                &mut commands,
                &assets,
                localization.get("not_enough_ap", lang),
                Color::srgba(0.20, 0.05, 0.05, 0.93),
                Color::srgb(0.85, 0.20, 0.20),
                Color::srgb(1.0, 0.80, 0.80),
                toast,
            );
            return;
        }

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

        // Spend resources
        player.gold -= gold_cost;
        let triggers_level_up = player.ap <= ap_cost;
        if triggers_level_up {
            trigger_level_up(&mut player, &mut level_up, &mut play_audio_msg, &mut next_game_state);
        } else {
            player.ap -= ap_cost;
        }

        let mut rng = rng();
        let max_hp = player.max_health();
        let max_mp = player.max_mana();

        match marker.0 {
            0 => {
                // Rough Rest: recovers between 40 and 60 + constitution modifier percent of total health and mana back (also to pet)
                let pct_roll = rng.random_range(40..=60) + player.constitution_mod();
                let pct = (pct_roll as f32 / 100.0).max(0.0).min(1.0);

                let health_recovered = (max_hp as f32 * pct) as u32;
                let mana_recovered = (max_mp as f32 * pct) as u32;

                let next_hp = (player.health() + health_recovered).min(max_hp);
                let next_mp = (player.mana() + mana_recovered).min(max_mp);
                player.set_health(next_hp);
                player.set_mana(next_mp);

                if let Some(ref mut pet) = player.pet {
                    let pet_hp_recovered = (pet.max_health as f32 * pct) as u32;
                    pet.health = (pet.health + pet_hp_recovered).min(pet.max_health);
                }

                play_audio_msg.write(PlayAudioMsg::new("rest"));
                spawn_toast(
                    &mut commands,
                    &assets,
                    localization
                        .get("toast_rest_recovered", lang)
                        .replace("{hp}", &health_recovered.to_string())
                        .replace("{mp}", &mana_recovered.to_string()),
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    toast,
                );
            },
            1 => {
                // Common Lodging: returns full health and mana back (also to pet)
                player.set_health(max_hp);
                player.set_mana(max_mp);

                if let Some(ref mut pet) = player.pet {
                    pet.health = pet.max_health;
                }

                play_audio_msg.write(PlayAudioMsg::new("rest"));
                spawn_toast(
                    &mut commands,
                    &assets,
                    localization.get("toast_rest_full_recovered", lang),
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    toast,
                );
            },
            2 => {
                // Grand Accommodation: returns full health and mana back and also between 0-10 * 10 * player level * constitution modifier extra max health and max mana
                let con_mod = player.constitution_mod().max(0) as u32;
                let factor = rng.random_range(0..=10) as u32;
                let bonus = factor * 10 * level * con_mod;

                player.bonus_max_health += bonus;
                player.bonus_max_mana += bonus;

                // Recover full health and mana (incorporating new bonus max)
                let new_max_hp = player.max_health();
                let new_max_mp = player.max_mana();
                player.set_health(new_max_hp);
                player.set_mana(new_max_mp);

                if let Some(ref mut pet) = player.pet {
                    pet.health = pet.max_health;
                }

                play_audio_msg.write(PlayAudioMsg::new("rest"));

                let mut msg = localization.get("toast_rest_full_recovered", lang);
                if bonus > 0 {
                    msg = format!(
                        "{} (+{})",
                        msg,
                        localization
                            .get("toast_rest_grand_bonus", lang)
                            .replace("{bonus}", &bonus.to_string())
                    );
                }
                spawn_toast(
                    &mut commands,
                    &assets,
                    msg,
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    toast,
                );
            },
            _ => {},
        }
    }
}
