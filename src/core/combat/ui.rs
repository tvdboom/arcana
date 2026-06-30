use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::core::network::DuelState;

#[cfg(target_arch = "wasm32")]
#[derive(Resource)]
pub struct DuelState {
    pub opponent: Option<crate::core::player::Player>,
}

use crate::core::combat::mechanics::DuelActive;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::get_equipment;
use crate::core::catalog::equipment::Equipment;
use crate::core::classes::Class;
use crate::core::constants::{
    BAR_BG_COLOR, BUTTON_BORDER_COLOR, BUTTON_TEXT_COLOR, HOVERED_BUTTON_COLOR, LABEL_TEXT_SIZE,
    NORMAL_BUTTON_COLOR, PLACEHOLDER_COLOR, PRESSED_BUTTON_COLOR, SELECTED_COLOR,
};
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::monsters::{ActiveMonster, Monster, MonsterKind};
use crate::core::player::Player;
use crate::core::settings::Settings;
use crate::core::ui::creation::SelectionItem;
use crate::core::ui::playing::{
    EquipSlot, InfoTooltip, PetHealthBarFill, RightColumnTooltip, StatLabel,
};
use crate::core::PlayingStat;
use crate::utils::capitalize_words;
use bevy::window::SystemCursorIcon;

const ACTIVE_HOTKEYS: [&str; 5] = ["Q", "W", "E", "R", "T"];
const LEFT_PANEL_WIDTH: f32 = 46.0;
const RIGHT_PANEL_WIDTH: f32 = 46.0;
const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const MANA_COLOR: Color = Color::srgb_u8(40, 80, 185);
const BUTTON_TEXT_SIZE: f32 = 2.2;
const COMBAT_PORTRAIT_ASPECT: f32 = 0.88;
const COMBAT_IMAGE_COLUMN_WIDTH: f32 = 80.0;
const COMBAT_STATS_COLUMN_WIDTH: f32 = 20.0;
const COMBAT_CONSUMABLE_CARD_SIZE: f32 = 4.2;
const COMBAT_ABILITY_CARD_SIZE: f32 = 5.6;
const CONSUMABLE_HOTKEYS: [&str; 8] = ["A", "S", "D", "F", "G", "H", "J", "K"];

#[derive(Component)]
pub struct CombatCmp;

#[derive(Component)]
pub struct CombatMonsterHealthFill;

#[derive(Component)]
pub struct CombatMonsterHealthText;

#[derive(Component)]
pub struct CombatPlayerPortrait;

#[derive(Component)]
pub struct CombatEnemyPortrait;

#[derive(Component)]
pub struct CombatEnemyManaFill;

#[derive(Component)]
pub struct CombatEnemyManaText;

#[derive(Component)]
pub struct CombatPausedOverlay;

#[derive(Component)]
pub struct AbilityCardImage {
    pub slot: usize,
}

#[derive(Component)]
pub struct CombatPortraitName {
    pub is_player: bool,
}

#[derive(Component)]
pub struct CombatPortraitLevel {
    pub is_player: bool,
}

#[derive(Component)]
pub struct CombatStatLabel {
    pub title_key: String,
}

#[derive(Component)]
pub struct CombatPetName;

pub fn setup_combat_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    active_monster: Option<Res<ActiveMonster>>,
    combat_speed: Res<crate::core::combat::mechanics::CombatSpeed>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    existing_combat_q: Query<Entity, With<CombatCmp>>,
    duel_active: Option<Res<DuelActive>>,
    duel_state: Option<Res<DuelState>>,
) {
    if !existing_combat_q.is_empty() {
        return;
    }
    play_audio_msg.write(PlayAudioMsg::new("horn"));

    let active_monster =
        active_monster.expect("ActiveMonster resource missing when entering combat");
    let monster = &active_monster.monster;
    let lang = settings.language;

    let is_pvp = duel_active.is_some();
    let opponent = duel_state.as_ref().and_then(|d| d.opponent.as_ref());

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
                    spawn_monster_panel(parent, &assets, &localization, lang, monster, is_pvp, opponent);
                });

            parent
                .spawn(if is_pvp {
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.),
                        margin: UiRect::left(Val::Vh(-9.0)),
                        bottom: Val::Px(24.),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.),
                        ..default()
                    }
                } else {
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(24.),
                        bottom: Val::Px(24.),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.),
                        ..default()
                    }
                })
                .with_children(|parent| {
                    if !is_pvp {
                        parent.spawn((
                            add_text(combat_speed.label(), "bold", LABEL_TEXT_SIZE, &assets),
                            TextColor(Color::WHITE),
                            crate::core::combat::mechanics::CombatSpeedText,
                        ));
                    }
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
                            crate::core::combat::mechanics::CombatEndButton,
                        ))
                        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                        .observe(crate::core::utils::cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(crate::core::utils::cursor::<Out>(SystemCursorIcon::Default))
                        .observe(crate::core::utils::cursor::<Release>(SystemCursorIcon::Default))
                        .observe(crate::core::combat::mechanics::handle_combat_end_button_click)
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    localization.get("general.forfeit_combat", lang),
                                    "bold",
                                    BUTTON_TEXT_SIZE,
                                    &assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                crate::core::combat::mechanics::CombatEndButtonText,
                            ));
                        });
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: percent(0.),
                        left: percent(0.),
                        width: percent(100.),
                        height: percent(100.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                    GlobalZIndex(985),
                    Visibility::Hidden,
                    CombatPausedOverlay,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        add_text(localization.get("general.paused", lang), "bold", 8.0, &assets),
                        TextColor(Color::WHITE),
                        LocalizedText("general.paused".to_string()),
                    ));
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
                            row_gap: Val::Px(0.),
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
                            spawn_combat_resource_bar(parent, assets, true, true);
                            spawn_combat_resource_bar(parent, assets, false, true);
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
                            spawn_combat_stats(
                                parent,
                                assets,
                                localization,
                                lang,
                                player.attack(),
                                player.defense(),
                                player.initiative(),
                                true,
                            );
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
            ImageNode::new(assets.image(portrait_key.to_string()))
                .with_mode(NodeImageMode::Stretch),
            CombatPlayerPortrait,
        ))
        .with_children(|parent| {
            spawn_portrait_label(parent, assets, name, level, true);
            if let Some(pet) = pet {
                spawn_combat_pet_overlay(parent, assets, pet);
            }
            spawn_equipment_slot_column(
                parent,
                assets,
                2.0,
                14.0,
                &[EquipSlot::Accessory, EquipSlot::Accessory2],
                true,
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
                true,
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
                    parent.spawn((
                        add_text(capitalize_words(&pet.name), "bold", 2.2, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        CombatPetName,
                    ));
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(5.),
                        right: Val::Percent(5.),
                        bottom: Val::Px(4.),
                        height: Val::Px(20.),
                        border: UiRect::all(Val::Px(1.5)),
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
                            width: percent(100.),
                            height: percent(100.),
                            ..default()
                        },
                        BackgroundColor(HEALTH_COLOR),
                        PetHealthBarFill,
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
                                add_text("", "bold", 1.2, assets),
                                TextColor(Color::WHITE),
                                StatLabel(PlayingStat::PetHealth),
                            ));
                        });
                });
        });
}

fn spawn_combat_enemy_pet_overlay(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    pet: &Monster,
) {
    let ratio = if pet.max_health > 0 {
        (pet.health as f32 / pet.max_health as f32).clamp(0.0, 1.0) * 100.0
    } else {
        100.0
    };

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
                    parent.spawn((
                        add_text(capitalize_words(&pet.name), "bold", 2.2, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(5.),
                        right: Val::Percent(5.),
                        bottom: Val::Px(4.),
                        height: Val::Px(20.),
                        border: UiRect::all(Val::Px(1.5)),
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
                            width: percent(ratio),
                            height: percent(100.),
                            ..default()
                        },
                        BackgroundColor(HEALTH_COLOR),
                    ));

                    // Text overlay
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
                                    format!("{} / {}", pet.health, pet.max_health),
                                    "bold",
                                    1.2,
                                    assets,
                                ),
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

fn spawn_portrait_label(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    name: &str,
    level: u32,
    is_player: bool,
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
            parent.spawn((
                add_text(capitalize_words(name), "bold", 2.8, assets),
                TextColor(BUTTON_TEXT_COLOR),
                CombatPortraitName { is_player },
            ));
            parent.spawn((
                add_text(format!("Level {}", level), "medium", 2.2, assets),
                TextColor(Color::WHITE),
                CombatPortraitLevel { is_player },
            ));
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
    is_player: bool,
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
            spawn_combat_stat_row(
                parent,
                assets,
                localization,
                lang,
                "attack",
                "general.attack",
                attack,
                PlayingStat::Attack,
                100.0,
                2.2,
                4.5,
                true,
                true,
                is_player,
            );
            spawn_combat_stat_row(
                parent,
                assets,
                localization,
                lang,
                "defense",
                "general.defense",
                defense,
                PlayingStat::Defense,
                100.0,
                2.2,
                4.5,
                true,
                true,
                is_player,
            );
            spawn_combat_stat_row(
                parent,
                assets,
                localization,
                lang,
                "initiative",
                "general.initiative",
                initiative,
                PlayingStat::Initiative,
                100.0,
                2.2,
                4.5,
                true,
                true,
                is_player,
            );
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
            spawn_combat_stat_row(
                parent,
                assets,
                localization,
                lang,
                "attack",
                "general.atk",
                pet.attack,
                PlayingStat::Attack,
                42.0,
                1.2,
                2.2,
                false,
                true,
                false,
            );
            spawn_combat_stat_row(
                parent,
                assets,
                localization,
                lang,
                "defense",
                "general.def",
                pet.defense,
                PlayingStat::Defense,
                42.0,
                1.2,
                2.2,
                false,
                true,
                false,
            );
            spawn_combat_stat_row(
                parent,
                assets,
                localization,
                lang,
                "initiative",
                "general.init",
                pet.initiative,
                PlayingStat::Initiative,
                42.0,
                1.2,
                2.2,
                false,
                true,
                false,
            );
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
    _is_player: bool,
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
        parent
            .spawn((Node {
                position_type: PositionType::Absolute,
                width: percent(100.),
                height: percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(1.),
                padding: UiRect::all(Val::Px(8.)),
                ..default()
            },))
            .with_children(|parent| {
                let label = if localize_label {
                    localization.get(title_key, lang)
                } else {
                    title_key.to_string()
                };
                parent.spawn((
                    add_text(label, "medium", label_font_size, assets),
                    TextColor(BUTTON_TEXT_COLOR),
                    CombatStatLabel {
                        title_key: title_key.to_string(),
                    },
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
    is_player: bool,
) {
    let (width, height) = if is_left_column {
        (16.0, 30.0)
    } else {
        (16.0, 96.0)
    };

    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: if is_left_column {
                Val::Percent(left)
            } else {
                Val::Auto
            },
            right: if is_left_column {
                Val::Auto
            } else {
                Val::Percent(left)
            },
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
                        crate::core::combat::mechanics::CombatSlot { is_player },
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
            margin: UiRect::top(Val::Px(6.)),
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
                        let ability_key =
                            player.active_abilities.get(index).and_then(|opt| opt.as_deref());
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
                            Some(crate::core::combat::mechanics::CombatCard::Ability(index)),
                            true,
                        );
                    }
                });
        });
}

fn spawn_consumables(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, player: &Player) {
    let mut consumables: Vec<_> = player
        .equipped_consumables
        .iter()
        .filter(|key| player.inventory.iter().any(|inv| inv == *key))
        .filter_map(|key| match get_equipment(key) {
            Some(Equipment::Consumable(item)) => Some((key.clone(), item)),
            _ => None,
        })
        .collect();
    consumables.sort_by(|a, b| b.1.level.cmp(&a.1.level).then(a.1.name.cmp(&b.1.name)));

    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.),
            margin: UiRect::top(Val::Px(6.)),
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
                    for (index, (key, item)) in
                        consumables.iter().take(CONSUMABLE_HOTKEYS.len()).enumerate()
                    {
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
                            Some(crate::core::combat::mechanics::CombatCard::Consumable(
                                key.clone(),
                            )),
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
    combat_card: Option<crate::core::combat::mechanics::CombatCard>,
    is_player: bool,
) {
    use crate::core::combat::mechanics::{
        handle_combat_card_click, AbilityCooldownOverlay, CombatCard, ConsumableCardRoot,
    };
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

    if let Some(card) = combat_card.clone() {
        cmd.insert(card.clone());
        cmd.observe(handle_combat_card_click);
        if let CombatCard::Ability(slot) = card {
            cmd.insert(AbilityCardImage {
                slot,
            });
        }
        if let CombatCard::Consumable(key) = &card {
            cmd.insert(ConsumableCardRoot {
                key: key.clone(),
                is_player,
            });
        }
    }

    cmd.with_children(|parent| {
        // Cooldown / disabled dark overlay for abilities (driven by combat state).
        if let Some(CombatCard::Ability(slot)) = combat_card.clone() {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    right: Val::Px(0.),
                    bottom: Val::Px(0.),
                    width: percent(100.),
                    height: percent(0.),
                    ..default()
                },
                BackgroundColor(Color::srgba(0., 0., 0., 0.95)),
                Pickable::IGNORE,
                AbilityCooldownOverlay {
                    slot,
                },
            ));

            // Cooldown remaining text overlay
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    right: Val::Px(0.),
                    top: Val::Px(0.),
                    bottom: Val::Px(0.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            )).with_children(|parent| {
                parent.spawn((
                    add_text("", "bold", 2.2, assets),
                    TextColor(Color::WHITE),
                    crate::core::combat::mechanics::AbilityCooldownText {
                        slot,
                    },
                ));
            });
        }
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
                    parent.spawn((
                        add_text(label, "bold", 1.4, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        Pickable::IGNORE,
                    ));
                });
        } else {
            parent
                .spawn((
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
                    parent.spawn((
                        add_text(capitalize_words(label), "medium", 1.4, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        Pickable::IGNORE,
                    ));
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
    is_pvp: bool,
    opponent: Option<&Player>,
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
                                false,
                            );
                        });

                    parent
                        .spawn(Node {
                            width: percent(COMBAT_IMAGE_COLUMN_WIDTH),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(0.),
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
                                if is_pvp { opponent.and_then(|opp| opp.pet.as_ref()) } else { None },
                                is_pvp,
                            );
                            spawn_monster_health_bar(parent, assets, localization, lang, monster);
                            if is_pvp {
                                spawn_enemy_mana_bar(parent, assets, localization, lang, opponent);
                                if let Some(opp) = opponent {
                                    spawn_enemy_active_abilities(parent, assets, opp);
                                    spawn_enemy_consumables(parent, assets, opp);
                                }
                            }
                        });
                });
        });
}

fn monster_display_name(monster: &Monster) -> String {
    let name = capitalize_words(&monster.name);
    if monster.kind != MonsterKind::Dragon {
        return name;
    }

    let mut parts = name.split_whitespace();
    let color = parts.next().unwrap_or("Dragon");
    let stage = parts.collect::<Vec<_>>().join(" ");
    if stage.is_empty() {
        format!("{color} Dragon")
    } else {
        format!("{color} Dragon ({stage})")
    }
}

fn spawn_monster_portrait(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    name: &str,
    level: u32,
    image_key: &str,
    pet: Option<&Monster>,
    is_pvp: bool,
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
            CombatEnemyPortrait,
        ))
        .with_children(|parent| {
            spawn_portrait_label(parent, assets, name, level, false);
            if let Some(pet) = pet {
                spawn_combat_enemy_pet_overlay(parent, assets, pet);
            }
            if is_pvp {
                spawn_equipment_slot_column(
                    parent,
                    assets,
                    2.0,
                    14.0,
                    &[EquipSlot::Accessory, EquipSlot::Accessory2],
                    true,
                    false,
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
                    false,
                );
            }
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
                border: UiRect {
                    left: Val::Px(2.),
                    right: Val::Px(2.),
                    top: Val::Px(0.),
                    bottom: Val::Px(2.),
                },
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
                        CombatMonsterHealthText,
                    ));
                });
        });
}

fn spawn_enemy_mana_bar(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: crate::core::settings::Language,
    opponent: Option<&Player>,
) {
    let max_mana = opponent.map(|o| o.max_mana() as f32).unwrap_or(100.0);
    let mana = opponent.map(|o| o.mana() as f32).unwrap_or(100.0);
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: Val::Px(36.),
                position_type: PositionType::Relative,
                border: UiRect {
                    left: Val::Px(2.),
                    right: Val::Px(2.),
                    top: Val::Px(0.),
                    bottom: Val::Px(2.),
                },
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
                    width: percent(if max_mana == 0.0 {
                        0.0
                    } else {
                        (mana / max_mana).clamp(0.0, 1.0) * 100.0
                    }),
                    height: percent(100.),
                    ..default()
                },
                BackgroundColor(MANA_COLOR),
                CombatEnemyManaFill,
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
                                "{} / {} (+{}) {}",
                                mana.round() as i32,
                                max_mana.round() as i32,
                                opponent.map(|o| o.mana_regen() as i32).unwrap_or(0),
                                localization.get("general.mana", lang)
                            ),
                            "bold",
                            1.9,
                            assets,
                        ),
                        TextColor(Color::WHITE),
                        CombatEnemyManaText,
                    ));
                });
        });
}

fn spawn_enemy_active_abilities(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    opponent: &Player,
) {
    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.),
            margin: UiRect::top(Val::Px(6.)),
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
                    for index in 0..ACTIVE_HOTKEYS.len() {
                        let ability_key =
                            opponent.active_abilities.get(index).and_then(|opt| opt.as_deref());
                        spawn_hover_card(
                            parent,
                            assets,
                            ability_key.map(|key| key.to_string()),
                            ability_key
                                .map(|key| format!("build_{key}"))
                                .unwrap_or_else(|| "stone".to_string()),
                            "",
                            ability_key.is_some(),
                            false,
                            ability_key.map(|key| RightColumnTooltip::Ability(key.to_string())),
                            COMBAT_ABILITY_CARD_SIZE,
                            false,
                            None,
                            false,
                        );
                    }
                });
        });
}

fn spawn_enemy_consumables(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    opponent: &Player,
) {
    let mut consumables: Vec<_> = opponent
        .equipped_consumables
        .iter()
        .filter(|key| opponent.inventory.iter().any(|inv| inv == *key))
        .filter_map(|key| match get_equipment(key) {
            Some(Equipment::Consumable(item)) => Some((key.clone(), item)),
            _ => None,
        })
        .collect();
    consumables.sort_by(|a, b| b.1.level.cmp(&a.1.level).then(a.1.name.cmp(&b.1.name)));

    parent
        .spawn(Node {
            width: percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.),
            margin: UiRect::top(Val::Px(6.)),
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
                    for (_index, (key, item)) in
                        consumables.iter().take(CONSUMABLE_HOTKEYS.len()).enumerate()
                    {
                        spawn_hover_card(
                            parent,
                            assets,
                            Some(key.clone()),
                            item.image.clone(),
                            "",
                            true,
                            false,
                            Some(RightColumnTooltip::Equipment(key.clone())),
                            COMBAT_CONSUMABLE_CARD_SIZE,
                            true,
                            None,
                            false,
                        );
                    }
                });
        });
}

fn spawn_combat_resource_bar(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    is_health: bool,
    omit_top_border: bool,
) {
    let bar_height = Val::Px(36.);
    let font_size = 1.9;
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: bar_height,
                position_type: PositionType::Relative,
                border: UiRect {
                    left: Val::Px(2.),
                    right: Val::Px(2.),
                    top: if omit_top_border {
                        Val::Px(0.)
                    } else {
                        Val::Px(2.)
                    },
                    bottom: Val::Px(2.),
                },
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            let mut fill = parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
                BackgroundColor(if is_health {
                    HEALTH_COLOR
                } else {
                    MANA_COLOR
                }),
            ));
            if is_health {
                fill.insert(crate::core::ui::playing::HealthBarFill);
            } else {
                fill.insert(crate::core::ui::playing::ManaBarFill);
            }

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
                        add_text("", "bold", font_size, assets),
                        TextColor(Color::WHITE),
                        StatLabel(if is_health {
                            PlayingStat::Health
                        } else {
                            PlayingStat::Mana
                        }),
                    ));
                });
        });
}
