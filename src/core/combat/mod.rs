use bevy::prelude::*;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::get_equipment;
use crate::core::catalog::equipment::Equipment;
use crate::core::constants::{
    BAR_BG_COLOR, BUTTON_BORDER_COLOR, BUTTON_TEXT_COLOR, NORMAL_BUTTON_COLOR,
    PLACEHOLDER_COLOR, PRESSED_BUTTON_COLOR, SELECTED_COLOR, HOVERED_BUTTON_COLOR,
};
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::player::Player;
use crate::core::PlayingStat;
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::ui::creation::SelectionItem;
use crate::core::ui::playing::{spawn_bar, EquipSlot, InfoTooltip, RightColumnTooltip};
use crate::utils::capitalize_words;
use bevy::window::SystemCursorIcon;
use crate::core::classes::Class;
use crate::core::monsters::{ActiveMonster, Monster, MonsterKind};

const ACTIVE_HOTKEYS: [&str; 5] = ["Q", "W", "E", "R", "T"];
const LEFT_PANEL_WIDTH: f32 = 46.0;
const RIGHT_PANEL_WIDTH: f32 = 46.0;
const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const BUTTON_TEXT_SIZE: f32 = 2.2;
const COMBAT_PORTRAIT_ASPECT: f32 = 0.88;
const COMBAT_IMAGE_COLUMN_WIDTH: f32 = 80.0;
const COMBAT_STATS_COLUMN_WIDTH: f32 = 20.0;
const COMBAT_CONSUMABLE_CARD_SIZE: f32 = 4.2;
const COMBAT_ABILITY_CARD_SIZE: f32 = 5.6;
const CONSUMABLE_HOTKEYS: [&str; 6] = ["A", "S", "D", "F", "G", "H"];

#[derive(Component)]
pub struct CombatCmp;

#[derive(Component)]
struct CombatMonsterHealthFill;

pub fn setup_combat_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    active_monster: Option<Res<ActiveMonster>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    play_audio_msg.write(PlayAudioMsg::new("horn"));

    let active_monster =
        active_monster.expect("ActiveMonster resource missing when entering combat");
    let monster = &active_monster.monster;
    let lang = settings.language;

    let (mut root_node, pickable) = add_root_node(true);
    root_node.padding = UiRect::all(Val::Px(0.));

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode {
                image: assets.image("bg_combat"),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.40, 0.40, 0.40, 1.0),
                ..default()
            },
            GlobalZIndex(980),
            CombatCmp,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    padding: UiRect::all(Val::Px(18.)),
                    column_gap: Val::Px(16.),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_player_panel(parent, &assets, &localization, lang, &player);
                    spawn_monster_panel(parent, &assets, &localization, lang, monster);
                });

            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(24.),
                    bottom: Val::Px(24.),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                width: Val::Vh(18.0),
                                height: Val::Vh(5.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Vh(0.22)),
                                border_radius: BorderRadius::all(Val::Vh(0.44)),
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON_COLOR),
                            BorderColor::all(BUTTON_BORDER_COLOR),
                            Button,
                            Interaction::default(),
                            Pickable::default(),
                        ))
                        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                        .observe(crate::core::utils::cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(crate::core::utils::cursor::<Out>(SystemCursorIcon::Default))
                        .observe(crate::core::utils::cursor::<Release>(SystemCursorIcon::Default))
                        .observe(handle_forfeit_combat_click)
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    localization.get("general.forfeit_combat", lang),
                                    "bold",
                                    BUTTON_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                            ));
                        });
                });
        });
}

fn spawn_player_panel(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: crate::core::settings::Language,
    player: &Player,
) {
    parent
        .spawn(Node {
            width: percent(LEFT_PANEL_WIDTH),
            height: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(12.),
                    align_items: AlignItems::Stretch,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: percent(COMBAT_IMAGE_COLUMN_WIDTH),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.),
                            align_items: AlignItems::Stretch,
                            align_self: AlignSelf::FlexStart,
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_character_portrait(
                                parent,
                                assets,
                                &player.name,
                                player.level(),
                                &player_image_key(player),
                                player.pet.as_ref(),
                            );
                            spawn_bar(parent, assets, true);
                            spawn_bar(parent, assets, false);
                            spawn_active_abilities(parent, assets, player);
                            spawn_consumables(parent, assets, player);
                        });

                    parent
                        .spawn(Node {
                            width: percent(COMBAT_STATS_COLUMN_WIDTH),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Stretch,
                            align_self: AlignSelf::FlexStart,
                            row_gap: Val::Px(8.),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_combat_stats(parent, assets, localization, lang, player.attack(), player.defense(), player.initiative());
                            if let Some(pet) = &player.pet {
                                spawn_pet_stats(parent, assets, localization, lang, pet);
                            }
                        });
                });
        });
}

fn player_image_key(player: &Player) -> String {
    match player.class {
        Class::Mage(ajah) => ajah.get_image_key(player),
        _ => player.class.get_image_key(player),
    }
}

fn spawn_character_portrait(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    name: &str,
    level: u32,
    portrait_key: &str,
    pet: Option<&Monster>,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                aspect_ratio: Some(COMBAT_PORTRAIT_ASPECT),
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(3.)),
                align_self: AlignSelf::FlexStart,
                ..default()
            },
            BorderColor::all(BUTTON_BORDER_COLOR),
            ImageNode::new(assets.image(portrait_key.to_string())).with_mode(NodeImageMode::Stretch),
        ))
        .with_children(|parent| {
            spawn_portrait_label(parent, assets, name, level);
            if let Some(pet) = pet {
                spawn_combat_pet_overlay(parent, assets, pet);
            }
            spawn_equipment_slot_column(
                parent,
                assets,
                2.0,
                2.0,
                &[EquipSlot::Accessory, EquipSlot::Accessory2],
                true,
            );
            spawn_equipment_slot_column(
                parent,
                assets,
                2.0,
                2.0,
                &[
                    EquipSlot::Helmet,
                    EquipSlot::Chestplate,
                    EquipSlot::WeaponLH,
                    EquipSlot::WeaponRH,
                    EquipSlot::Gloves,
                    EquipSlot::Boots,
                ],
                false,
            );
        });
}

fn pet_image_key(pet: &Monster) -> String {
    std::path::Path::new(&pet.image)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&pet.image)
        .to_lowercase()
}

fn spawn_combat_pet_overlay(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    pet: &Monster,
) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(3.),
                bottom: Val::Px(3.),
                width: percent(55.),
                aspect_ratio: Some(0.92),
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            BorderColor::all(BUTTON_BORDER_COLOR),
            ImageNode::new(assets.image(pet_image_key(pet))).with_mode(NodeImageMode::Stretch),
            Interaction::default(),
            Pickable::default(),
            InfoTooltip::Pet,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(6.),
                    top: Val::Px(6.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((add_text(capitalize_words(&pet.name), "bold", 1.8, assets), TextColor(BUTTON_TEXT_COLOR)));
                });
        });
}

fn spawn_portrait_label(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    name: &str,
    level: u32,
) {
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.),
            top: Val::Px(10.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(1.),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((add_text(capitalize_words(name), "bold", 2.8, assets), TextColor(BUTTON_TEXT_COLOR)));
            parent.spawn((add_text(format!("Level {}", level), "medium", 2.2, assets), TextColor(Color::WHITE)));
        });
}

fn spawn_combat_stats(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: crate::core::settings::Language,
    attack: u32,
    defense: u32,
    initiative: u32,
) {
    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.),
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|parent| {
            spawn_combat_stat_row(parent, assets, localization, lang, "attack", "general.attack", attack, PlayingStat::Attack, 100.0, 2.2, 4.5, true, true);
            spawn_combat_stat_row(parent, assets, localization, lang, "defense", "general.defense", defense, PlayingStat::Defense, 100.0, 2.2, 4.5, true, true);
            spawn_combat_stat_row(parent, assets, localization, lang, "initiative", "general.initiative", initiative, PlayingStat::Initiative, 100.0, 2.2, 4.5, true, true);
        });
}

fn spawn_pet_stats(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: crate::core::settings::Language,
    pet: &Monster,
) {
    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.),
            margin: UiRect::top(Val::Px(4.)),
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|parent| {
            spawn_combat_stat_row(parent, assets, localization, lang, "attack", "Atk.", pet.attack, PlayingStat::Attack, 42.0, 1.2, 2.2, false, false);
            spawn_combat_stat_row(parent, assets, localization, lang, "defense", "Def.", pet.defense, PlayingStat::Defense, 42.0, 1.2, 2.2, false, false);
            spawn_combat_stat_row(parent, assets, localization, lang, "initiative", "Init.", pet.initiative, PlayingStat::Initiative, 42.0, 1.2, 2.2, false, false);
        });
}

fn spawn_combat_stat_row(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: crate::core::settings::Language,
    image_key: &str,
    title_key: &str,
    value: u32,
    stat: PlayingStat,
    card_width: f32,
    label_font_size: f32,
    value_font_size: f32,
    show_tooltip: bool,
    localize_label: bool,
) {
    let mut cmd = parent.spawn(Node {
            width: percent(card_width),
            aspect_ratio: Some(1.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(3.),
            border: UiRect::all(Val::Px(2.)),
            position_type: PositionType::Relative,
            ..default()
        });
    cmd.insert(BackgroundColor(PLACEHOLDER_COLOR))
        .insert(BorderColor::all(BUTTON_BORDER_COLOR))
        .insert(Interaction::default())
        .insert(Pickable::default());
    if show_tooltip {
        cmd.insert(InfoTooltip::Combat(stat));
    }
    cmd.with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
                ImageNode {
                    image: assets.image(image_key),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(1., 1., 1., 0.30),
                    ..default()
                },
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.),
                    height: percent(100.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(1.),
                    padding: UiRect::all(Val::Px(8.)),
                    ..default()
                },
            ))
            .with_children(|parent| {
                let label = if localize_label {
                    localization.get(title_key, lang)
                } else {
                    title_key.to_string()
                };
                parent.spawn((
                    add_text(label, "medium", label_font_size, assets),
                    TextColor(BUTTON_TEXT_COLOR),
                ));
                parent.spawn((
                    add_text(format!("{}", value), "bold", value_font_size, assets),
                    TextColor(Color::WHITE),
                ));
            });
        });
}

fn spawn_equipment_slot_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    left: f32,
    top: f32,
    slots: &[EquipSlot],
    is_left_column: bool,
) {
    let (width, height) = if is_left_column {
        (16.0, 30.0)
    } else {
        (16.0, 96.0)
    };

    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: if is_left_column { Val::Percent(left) } else { Val::Auto },
            right: if is_left_column { Val::Auto } else { Val::Percent(left) },
            top: Val::Percent(top),
            width: percent(width),
            height: percent(height),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: Val::Px(1.),
            ..default()
        })
        .with_children(|parent| {
            for slot in slots {
                parent
                    .spawn((
                        Node {
                            width: percent(100.),
                            aspect_ratio: Some(1.),
                            border: UiRect::all(Val::Px(1.)),
                            ..default()
                        },
                        BackgroundColor(PLACEHOLDER_COLOR),
                        BorderColor::all(BUTTON_BORDER_COLOR),
                        ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
                        Interaction::default(),
                        Button,
                        Pickable::default(),
                        *slot,
                    ))
                    .observe(crate::core::utils::cursor::<Over>(SystemCursorIcon::Pointer))
                    .observe(crate::core::utils::cursor::<Out>(SystemCursorIcon::Default))
                    .observe(crate::core::ui::playing::handle_equipment_slot_click);
            }
        });
}

fn spawn_active_abilities(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    player: &Player,
) {
    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(1.),
                    ..default()
                })
                .with_children(|parent| {
                    for (index, hotkey) in ACTIVE_HOTKEYS.iter().enumerate() {
                        let ability_key = player.active_abilities.get(index).and_then(|opt| opt.as_deref());
                        spawn_hover_card(
                            parent,
                            assets,
                            ability_key.map(|key| key.to_string()),
                            ability_key
                                .map(|key| format!("build_{key}"))
                                .unwrap_or_else(|| "stone".to_string()),
                            *hotkey,
                            ability_key.is_some(),
                            true,
                            ability_key.map(|key| RightColumnTooltip::Ability(key.to_string())),
                            COMBAT_ABILITY_CARD_SIZE,
                            false,
                        );
                    }
                });
        });
}

fn spawn_consumables(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    player: &Player,
) {
    let consumables: Vec<_> = player
        .inventory
        .iter()
        .filter_map(|key| match get_equipment(key) {
            Some(Equipment::Consumable(item)) => Some((key.clone(), item)),
            _ => None,
        })
        .collect();

    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(2.),
                    row_gap: Val::Px(2.),
                    ..default()
                })
                .with_children(|parent| {
                    for (index, (key, item)) in consumables.iter().take(6).enumerate() {
                        spawn_hover_card(
                            parent,
                            assets,
                            Some(key.clone()),
                            item.image.clone(),
                            CONSUMABLE_HOTKEYS[index],
                            true,
                            true,
                            Some(RightColumnTooltip::Equipment(key.clone())),
                            COMBAT_CONSUMABLE_CARD_SIZE,
                            true,
                        );
                    }
                });
        });
}

fn spawn_hover_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    tooltip_key: Option<String>,
    image_key: String,
    label: &str,
    has_border: bool,
    show_hotkey: bool,
    tooltip: Option<RightColumnTooltip>,
    card_size: f32,
    dark_background: bool,
) {
    let mut cmd = parent.spawn((
        Node {
            width: Val::Vw(card_size),
            height: Val::Vw(card_size),
            position_type: PositionType::Relative,
            border: if has_border {
                UiRect::all(Val::Px(2.))
            } else {
                UiRect::all(Val::Px(1.))
            },
            ..default()
        },
        BorderColor::all(BUTTON_BORDER_COLOR),
        BackgroundColor(if dark_background {
            Color::srgba(0.05, 0.05, 0.08, 0.58)
        } else if tooltip_key.is_some() {
            SELECTED_COLOR
        } else {
            BAR_BG_COLOR
        }),
        Interaction::default(),
        Button,
        Pickable::default(),
    ));

    if let Some(tooltip) = tooltip {
        cmd.insert(tooltip);
    }

    cmd.insert(ImageNode::new(assets.image(image_key)).with_mode(NodeImageMode::Stretch))
        .observe(recolor::<Over>(NORMAL_BUTTON_COLOR))
        .observe(recolor::<Out>(if dark_background {
            Color::srgba(0.05, 0.05, 0.08, 0.58)
        } else if tooltip_key.is_some() {
            SELECTED_COLOR
        } else {
            BAR_BG_COLOR
        }))
        .observe(crate::core::utils::cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(crate::core::utils::cursor::<Out>(SystemCursorIcon::Default));

    cmd.with_children(|parent| {
        if show_hotkey {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(1.),
                        bottom: Val::Px(-1.),
                        padding: UiRect::axes(Val::Px(2.), Val::Px(0.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0., 0., 0., 0.7)),
                    Pickable::IGNORE,
                ))
                .with_children(|parent| {
                    parent.spawn((add_text(label, "bold", 1.4, &assets), TextColor(BUTTON_TEXT_COLOR), Pickable::IGNORE));
                });
        } else {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(2.),
                    left: Val::Px(2.),
                    right: Val::Px(2.),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|parent| {
                parent.spawn((add_text(capitalize_words(label), "medium", 1.4, &assets), TextColor(BUTTON_TEXT_COLOR), Pickable::IGNORE));
            });
        }
    });
}

fn spawn_monster_panel(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: crate::core::settings::Language,
    monster: &Monster,
) {
    parent
        .spawn(Node {
            width: percent(RIGHT_PANEL_WIDTH),
            height: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.),
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(12.),
                    align_items: AlignItems::Stretch,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: percent(COMBAT_STATS_COLUMN_WIDTH),
                            flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Stretch,
                                align_self: AlignSelf::FlexStart,
                                row_gap: Val::Px(8.),
                                ..default()
                            })
                        .with_children(|parent| {
                            spawn_combat_stats(
                                parent,
                                assets,
                                localization,
                                lang,
                                monster.attack,
                                monster.defense,
                                monster.initiative,
                            );
                        });

                    parent
                        .spawn(Node {
                            width: percent(COMBAT_IMAGE_COLUMN_WIDTH),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.),
                            align_items: AlignItems::Stretch,
                                align_self: AlignSelf::FlexStart,
                                ..default()
                            })
                        .with_children(|parent| {
                            spawn_monster_portrait(
                                parent,
                                assets,
                                &monster_display_name(monster),
                                monster.level,
                                &monster.image,
                            );
                            spawn_monster_health_bar(parent, assets, localization, lang, monster);
                        });
                });
        });
}

fn monster_display_name(monster: &Monster) -> String {
    let mut name = capitalize_words(&monster.name);
    if monster.kind == MonsterKind::Dragon && !name.to_lowercase().ends_with("dragon") {
        name.push_str(" Dragon");
    }
    name
}

fn spawn_monster_portrait(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    name: &str,
    level: u32,
    image_key: &str,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                aspect_ratio: Some(COMBAT_PORTRAIT_ASPECT),
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(3.)),
                align_self: AlignSelf::FlexStart,
                ..default()
            },
            BorderColor::all(BUTTON_BORDER_COLOR),
            ImageNode::new(assets.image(image_key.to_string())).with_mode(NodeImageMode::Stretch),
        ))
        .with_children(|parent| {
            spawn_portrait_label(parent, assets, name, level);
        });
}

fn spawn_monster_health_bar(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: crate::core::settings::Language,
    monster: &Monster,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: Val::Px(36.),
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(2.)),
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    width: percent(if monster.max_health == 0 {
                        0.0
                    } else {
                        (monster.health as f32 / monster.max_health as f32).clamp(0.0, 1.0) * 100.0
                    }),
                    height: percent(100.),
                    ..default()
                },
                BackgroundColor(HEALTH_COLOR),
                CombatMonsterHealthFill,
            ));
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    width: percent(100.),
                    height: percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(
                            format!(
                                "{} / {} {}",
                                monster.health,
                                monster.max_health,
                                localization.get("general.health", lang)
                            ),
                            "bold",
                            1.9,
                            assets,
                        ),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn handle_forfeit_combat_click(
    _event: On<Pointer<Click>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    play_audio_msg.write(PlayAudioMsg::new("button"));
    next_game_state.set(GameState::Playing);
}
