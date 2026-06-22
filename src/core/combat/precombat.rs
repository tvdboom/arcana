use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::get_equipment;
use crate::core::catalog::equipment::Equipment;
use crate::core::catalog::weapons::Hand;
use crate::core::constants::{BUTTON_BORDER_COLOR, NORMAL_BUTTON_COLOR};
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::states::GameState;
use crate::core::ui::creation::SelectionItem;
use crate::core::ui::playing::{equip_item, unequip_slot, EquipSlot};
use crate::core::ui::utils::despawn_descendants_manual;
use crate::core::utils::cursor;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

#[derive(Resource, Default, Clone)]
pub struct PrecombatLoadout {
    pub abilities: Vec<String>,
    pub consumables: Vec<String>,
}

#[derive(Component)]
pub struct PrecombatCmp;

#[derive(Component)]
pub struct PrecombatContentWrapper;

#[derive(Component)]
pub struct PrecombatEquipItem(pub String);

#[derive(Component)]
pub struct PrecombatAbilityItem(pub String);

#[derive(Component)]
pub struct PrecombatConsumableItem(pub String);

#[derive(Component, Clone, Copy)]
pub struct PrecombatSlot(pub EquipSlot);

#[derive(Component)]
pub struct PrecombatStartButton;

pub fn setup_precombat_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    mut loadout: ResMut<PrecombatLoadout>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    play_audio_msg.write(PlayAudioMsg::new("horn"));

    loadout.abilities = player.abilities.iter().take(5).cloned().collect();

    let mut available_consumables = collect_consumables(&player);
    available_consumables.sort_by(|a, b| a.0.cmp(&b.0));
    loadout.consumables =
        available_consumables.iter().take(5).map(|(key, _)| key.clone()).collect();

    let (root_node, pickable) = add_root_node(true);
    commands
        .spawn((
            root_node,
            pickable,
            ImageNode {
                image: assets.image("basebg"),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.2, 0.2, 0.2, 0.95),
                ..default()
            },
            GlobalZIndex(980),
            PrecombatCmp,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(94.),
                        height: percent(92.),
                        border: UiRect::all(Val::Px(2.)),
                        padding: UiRect::all(Val::Px(16.)),
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    BackgroundColor(Color::srgba(0.05, 0.06, 0.08, 0.92)),
                    PrecombatContentWrapper,
                ))
                .with_children(|parent| {
                    build_precombat_content(
                        parent,
                        &assets,
                        &localization,
                        settings.language,
                        &player,
                        &loadout,
                    );
                });
        });
}

pub fn update_precombat_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    loadout: Res<PrecombatLoadout>,
    wrapper_q: Query<Entity, With<PrecombatContentWrapper>>,
    children_q: Query<&Children>,
) {
    if !player.is_changed() && !loadout.is_changed() {
        return;
    }

    if let Some(wrapper) = wrapper_q.iter().next() {
        despawn_descendants_manual(&mut commands, wrapper, &children_q);
        commands.entity(wrapper).with_children(|parent| {
            build_precombat_content(
                parent,
                &assets,
                &localization,
                settings.language,
                &player,
                &loadout,
            );
        });
    }
}

fn build_precombat_content(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    loadout: &PrecombatLoadout,
) {
    parent.spawn((
        add_text(localization.get("general.precombat_title", lang), "bold", 3.6, assets),
        TextColor(Color::WHITE),
    ));

    parent.spawn((
        add_text(localization.get("general.precombat_desc", lang), "medium", 1.8, assets),
        TextColor(Color::srgb(0.85, 0.85, 0.9)),
    ));

    parent
        .spawn(Node {
            width: percent(100.),
            height: percent(84.),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(18.),
            ..default()
        })
        .with_children(|parent| {
            spawn_precombat_portrait(parent, assets, localization, lang, player);
            spawn_precombat_lists(parent, assets, localization, lang, player, loadout);
        });
}

fn spawn_precombat_portrait(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) {
    parent
        .spawn(Node {
            width: percent(36.),
            height: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        aspect_ratio: Some(0.88),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(match player.class {
                        crate::core::classes::Class::Mage(ajah) => ajah.get_image_key(player),
                        _ => player.class.get_image_key(player),
                    }))
                    .with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(2.),
                            top: Val::Percent(2.),
                            width: percent(16.),
                            height: percent(30.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in [EquipSlot::Accessory, EquipSlot::Accessory2] {
                                spawn_precombat_slot(parent, assets, player, slot);
                            }
                        });

                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            right: Val::Percent(2.),
                            top: Val::Percent(2.),
                            width: percent(16.),
                            height: percent(96.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in [
                                EquipSlot::Helmet,
                                EquipSlot::Chestplate,
                                EquipSlot::WeaponLH,
                                EquipSlot::WeaponRH,
                                EquipSlot::Gloves,
                                EquipSlot::Boots,
                            ] {
                                spawn_precombat_slot(parent, assets, player, slot);
                            }
                        });
                });

            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.),
                    ..default()
                })
                .with_children(|parent| {
                    for (key, value) in [
                        ("general.attack", player.attack()),
                        ("general.defense", player.defense()),
                        ("general.initiative", player.initiative()),
                        ("general.health", player.health()),
                        ("general.mana", player.mana()),
                    ] {
                        parent.spawn((
                            add_text(
                                format!("{}: {}", localization.get(key, lang), value),
                                "medium",
                                1.8,
                                assets,
                            ),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}

fn spawn_precombat_slot(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    player: &Player,
    slot: EquipSlot,
) {
    let slot_key = equipped_key_for_slot(player, slot);
    let image = if let Some(key) = slot_key {
        get_equipment(key)
            .map(|eq| assets.image(format!("build_{}", eq.name())))
            .unwrap_or_else(|| assets.image("stone"))
    } else {
        assets.image("stone")
    };

    parent
        .spawn((
            Node {
                width: percent(100.),
                aspect_ratio: Some(1.),
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BorderColor::all(BUTTON_BORDER_COLOR),
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.85)),
            ImageNode::new(image).with_mode(NodeImageMode::Stretch),
            Button,
            Interaction::default(),
            Pickable::default(),
            PrecombatSlot(slot),
        ))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(handle_precombat_slot_click);
}

fn spawn_precombat_lists(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    loadout: &PrecombatLoadout,
) {
    parent
        .spawn(Node {
            width: percent(64.),
            height: percent(100.),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get("general.precombat_equipment", lang),
                            "bold",
                            2.2,
                            assets,
                        ),
                        TextColor(Color::WHITE),
                    ));

                    let mut equipables: Vec<_> = player
                        .inventory
                        .iter()
                        .filter_map(|key| {
                            get_equipment(key).and_then(|eq| match eq {
                                Equipment::Weapon(_) | Equipment::Wearable(_) => {
                                    Some((key.clone(), eq))
                                },
                                _ => None,
                            })
                        })
                        .collect();
                    equipables.sort_by(|a, b| {
                        a.1.level().cmp(&b.1.level()).then(a.1.name().cmp(b.1.name()))
                    });

                    for (key, eq) in equipables.into_iter().take(10) {
                        parent
                            .spawn((
                                Node {
                                    width: percent(100.),
                                    padding: UiRect::axes(Val::Px(8.), Val::Px(6.)),
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON_COLOR),
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                PrecombatEquipItem(key),
                            ))
                            .observe(recolor::<Over>(Color::srgb(0.35, 0.35, 0.35)))
                            .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                            .observe(handle_precombat_equip_click)
                            .with_children(|parent| {
                                parent.spawn((
                                    add_text(
                                        format!("{} (Lv{})", eq.name(), eq.level()),
                                        "medium",
                                        1.6,
                                        assets,
                                    ),
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });

            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: percent(50.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    format!(
                                        "{} ({}/5)",
                                        localization.get("general.precombat_abilities", lang),
                                        loadout.abilities.len()
                                    ),
                                    "bold",
                                    2.0,
                                    assets,
                                ),
                                TextColor(Color::WHITE),
                            ));

                            for key in &player.abilities {
                                let selected = loadout.abilities.contains(key);
                                parent
                                    .spawn((
                                        Node {
                                            width: percent(100.),
                                            padding: UiRect::axes(Val::Px(8.), Val::Px(6.)),
                                            border: UiRect::all(Val::Px(1.)),
                                            ..default()
                                        },
                                        BackgroundColor(if selected {
                                            Color::srgb(0.23, 0.38, 0.23)
                                        } else {
                                            NORMAL_BUTTON_COLOR
                                        }),
                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                        Button,
                                        Interaction::default(),
                                        Pickable::default(),
                                        PrecombatAbilityItem(key.clone()),
                                    ))
                                    .observe(recolor::<Over>(Color::srgb(0.35, 0.35, 0.35)))
                                    .observe(recolor::<Out>(if selected {
                                        Color::srgb(0.23, 0.38, 0.23)
                                    } else {
                                        NORMAL_BUTTON_COLOR
                                    }))
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(handle_precombat_ability_click)
                                    .with_children(|parent| {
                                        let prefix = if selected {
                                            "[x]"
                                        } else {
                                            "[ ]"
                                        };
                                        parent.spawn((
                                            add_text(
                                                format!("{prefix} {key}"),
                                                "medium",
                                                1.5,
                                                assets,
                                            ),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            }
                        });

                    parent
                        .spawn(Node {
                            width: percent(50.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    format!(
                                        "{} ({}/5)",
                                        localization.get("general.precombat_consumables", lang),
                                        loadout.consumables.len()
                                    ),
                                    "bold",
                                    2.0,
                                    assets,
                                ),
                                TextColor(Color::WHITE),
                            ));

                            for (key, count) in collect_consumables(player) {
                                let selected = loadout.consumables.contains(&key);
                                parent
                                    .spawn((
                                        Node {
                                            width: percent(100.),
                                            padding: UiRect::axes(Val::Px(8.), Val::Px(6.)),
                                            border: UiRect::all(Val::Px(1.)),
                                            ..default()
                                        },
                                        BackgroundColor(if selected {
                                            Color::srgb(0.23, 0.38, 0.23)
                                        } else {
                                            NORMAL_BUTTON_COLOR
                                        }),
                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                        Button,
                                        Interaction::default(),
                                        Pickable::default(),
                                        PrecombatConsumableItem(key.clone()),
                                    ))
                                    .observe(recolor::<Over>(Color::srgb(0.35, 0.35, 0.35)))
                                    .observe(recolor::<Out>(if selected {
                                        Color::srgb(0.23, 0.38, 0.23)
                                    } else {
                                        NORMAL_BUTTON_COLOR
                                    }))
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(handle_precombat_consumable_click)
                                    .with_children(|parent| {
                                        let prefix = if selected {
                                            "[x]"
                                        } else {
                                            "[ ]"
                                        };
                                        parent.spawn((
                                            add_text(
                                                format!("{prefix} {key} x{count}"),
                                                "medium",
                                                1.5,
                                                assets,
                                            ),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            }
                        });
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(8.),
                        bottom: Val::Px(8.),
                        padding: UiRect::axes(Val::Px(18.), Val::Px(10.)),
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.20, 0.45, 0.20)),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    PrecombatStartButton,
                ))
                .observe(recolor::<Over>(Color::srgb(0.28, 0.58, 0.28)))
                .observe(recolor::<Out>(Color::srgb(0.20, 0.45, 0.20)))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .observe(handle_precombat_start_click)
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            localization.get("general.start_combat", lang),
                            "bold",
                            2.0,
                            assets,
                        ),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn collect_consumables(player: &Player) -> Vec<(String, u32)> {
    let mut map = std::collections::BTreeMap::new();
    for key in &player.inventory {
        if matches!(get_equipment(key), Some(Equipment::Consumable(_))) {
            *map.entry(key.clone()).or_insert(0u32) += 1;
        }
    }
    map.into_iter().collect()
}

fn equipped_key_for_slot(player: &Player, slot: EquipSlot) -> Option<&str> {
    let is_lh_two_hand = player
        .weapon_lh
        .as_deref()
        .and_then(get_equipment)
        .map(|eq| matches!(eq, Equipment::Weapon(w) if w.hand == Hand::TwoHand))
        .unwrap_or(false);

    match slot {
        EquipSlot::Helmet => player.helmet.as_deref(),
        EquipSlot::Accessory => player.accessory.as_deref(),
        EquipSlot::Accessory2 => player.accessory2.as_deref(),
        EquipSlot::WeaponLH => player.weapon_lh.as_deref(),
        EquipSlot::WeaponRH => {
            if is_lh_two_hand {
                None
            } else {
                player.weapon_rh.as_deref()
            }
        },
        EquipSlot::Chestplate => player.armor.as_deref(),
        EquipSlot::Boots => player.boots.as_deref(),
        EquipSlot::Gloves => player.gloves.as_deref(),
    }
}

pub fn handle_precombat_equip_click(
    event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    item_q: Query<&PrecombatEquipItem>,
) {
    if let Ok(item) = item_q.get(event.entity) {
        let sound = equip_item(&mut player, &item.0).unwrap_or("click");
        play_audio_msg.write(PlayAudioMsg::new(sound));
    }
}

pub fn handle_precombat_slot_click(
    event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    slot_q: Query<&PrecombatSlot>,
) {
    if let Ok(slot) = slot_q.get(event.entity) {
        if unequip_slot(&mut player, slot.0) {
            play_audio_msg.write(PlayAudioMsg::new("click"));
        } else {
            play_audio_msg.write(PlayAudioMsg::new("error"));
        }
    }
}

pub fn handle_precombat_ability_click(
    event: On<Pointer<Click>>,
    mut loadout: ResMut<PrecombatLoadout>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    item_q: Query<&PrecombatAbilityItem>,
) {
    if let Ok(item) = item_q.get(event.entity) {
        if let Some(pos) = loadout.abilities.iter().position(|x| x == &item.0) {
            loadout.abilities.remove(pos);
            play_audio_msg.write(PlayAudioMsg::new("button"));
        } else if loadout.abilities.len() < 5 {
            loadout.abilities.push(item.0.clone());
            play_audio_msg.write(PlayAudioMsg::new("button"));
        } else {
            play_audio_msg.write(PlayAudioMsg::new("error"));
        }
    }
}

pub fn handle_precombat_consumable_click(
    event: On<Pointer<Click>>,
    mut loadout: ResMut<PrecombatLoadout>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    item_q: Query<&PrecombatConsumableItem>,
) {
    if let Ok(item) = item_q.get(event.entity) {
        if let Some(pos) = loadout.consumables.iter().position(|x| x == &item.0) {
            loadout.consumables.remove(pos);
            play_audio_msg.write(PlayAudioMsg::new("button"));
        } else if loadout.consumables.len() < 5 {
            loadout.consumables.push(item.0.clone());
            play_audio_msg.write(PlayAudioMsg::new("button"));
        } else {
            play_audio_msg.write(PlayAudioMsg::new("error"));
        }
    }
}

pub fn handle_precombat_start_click(
    _event: On<Pointer<Click>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}
