use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::Class;
use crate::core::constants::*;
use crate::core::consumables::Consumable;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::player::{Attribute, Player};
use crate::core::settings::{Language, Settings};
use crate::core::ui::creation::SelectionItem;
use crate::core::utils::cursor;
use crate::core::weapons::Weapon;
use crate::utils::NameFromEnum;
use bevy::window::SystemCursorIcon;

const PANEL_COLOR: Color = Color::srgba_u8(10, 18, 45, 230);
const BAR_BG_COLOR: Color = Color::srgba_u8(0, 0, 0, 160);
const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const MANA_COLOR: Color = Color::srgb_u8(40, 80, 185);
const PLACEHOLDER_COLOR: Color = Color::srgba_u8(40, 40, 55, 220);

#[derive(Component)]
pub struct PlayingCmp;

#[derive(Component)]
pub struct PlayingLog(pub Vec<String>);

#[derive(Component)]
pub struct LogTextContainer;

#[derive(Component)]
pub struct ActionButton(pub &'static str);

/// Simple text stats that are refreshed every frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayingStat {
    ClassLine,
    RaceLine,
    Health,
    Mana,
    Money,
    Attack,
    Armor,
    Initiative,
}

#[derive(Component)]
pub struct StatLabel(pub PlayingStat);

#[derive(Component)]
pub struct AttrValue(pub Attribute);

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct ManaBarFill;

#[derive(Component)]
pub struct EquipmentList;

#[derive(Component)]
pub struct AbilitiesList;

#[derive(Component)]
pub struct PerksList;

/// The five equipment image-slots overlaid on the character portrait.
#[derive(Component, Clone, Copy)]
pub enum EquipSlot {
    Helmet,
    Weapon,
    Offhand,
    Armor,
    Boots,
}

#[derive(Component)]
pub struct PetImage;

fn portrait_key(player: &Player) -> String {
    match player.class {
        Class::Mage(ajah) => ajah.get_image_key(player),
        _ => player.class.get_image_key(player),
    }
}

fn class_line(player: &Player, localization: &Localization, lang: Language) -> String {
    let class_name = match player.class {
        Class::Mage(ajah) => format!("{} {}", ajah.to_title(), localization.get("mage", lang)),
        _ => localization.get(&player.class.to_lowername(), lang),
    };
    format!("{} {} {}", localization.get("level", lang), player.level, class_name)
}

fn characteristics_text(player: &Player, localization: &Localization, lang: Language) -> String {
    let (age, height, weight) = player.vitals();
    let sex_val = match player.sex {
        crate::core::player::Sex::Male => localization.get("male", lang),
        crate::core::player::Sex::Female => localization.get("female", lang),
    };
    format!(
        "{}: {}\n{}: {}\n{}: {}\n{}: {} cm\n{}: {} kg",
        localization.get("race", lang),
        localization.get(&player.race.to_lowername(), lang),
        localization.get("sex", lang),
        sex_val,
        localization.get("age", lang),
        age,
        localization.get("height", lang),
        height,
        localization.get("weight", lang),
        weight,
    )
}

/// Format the bonus characteristics of a weapon, e.g. "+6 Attack, +10 Crit".
fn weapon_stat_lines(weapon: &Weapon, localization: &Localization, lang: Language) -> Vec<String> {
    let stats = weapon.stats();
    let mut lines = Vec::new();
    let mut push = |val: i32, key: &str| {
        if val != 0 {
            let sign = if val > 0 { "+" } else { "" };
            lines.push(format!("{}{} {}", sign, val, localization.get(key, lang)));
        }
    };
    push(stats.attack, "attack");
    push(stats.armor, "armor");
    push(stats.crit, "crit");
    push(stats.initiative, "initiative");
    lines
}

/// A bordered placeholder box (used wherever an item/ability image will go later).
fn spawn_placeholder(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, size: Val) {
    parent.spawn((
        Node {
            width: size,
            height: size,
            flex_shrink: 0.,
            border: UiRect::all(Val::Px(2.)),
            ..default()
        },
        BackgroundColor(PLACEHOLDER_COLOR),
        BorderColor::all(BUTTON_BORDER_COLOR),
        ImageNode::new(assets.image("stone")).with_mode(NodeImageMode::Stretch),
    ));
}

/// A list entry: placeholder image on the left, a name and detail lines on the right.
fn spawn_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    name: String,
    name_key: Option<String>,
    lines: Vec<String>,
) {
    parent
        .spawn((
            Node {
                width: percent(100.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.),
                padding: UiRect::all(Val::Px(6.)),
                margin: UiRect::bottom(Val::Px(6.)),
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            spawn_placeholder(parent, assets, Val::Px(40.));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    let mut name_cmd = parent.spawn((
                        add_text(name, "bold", 1.7, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                    if let Some(key) = name_key {
                        name_cmd.insert(LocalizedText(key));
                    }

                    for line in lines {
                        parent.spawn((
                            add_text(line, "medium", 1.4, assets),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}

/// One of the three combat-stat boxes (attack / armor / initiative).
fn spawn_combat_stat(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    label_key: &str,
    image_key: &str,
    stat: PlayingStat,
) {
    parent
        .spawn((
            Node {
                width: percent(30.),
                aspect_ratio: Some(1.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(4.),
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            BackgroundColor(PLACEHOLDER_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            ImageNode::new(assets.image(image_key)).with_mode(NodeImageMode::Stretch),
        ))
        .with_children(|parent| {
            parent.spawn((
                add_text(localization.get(label_key, lang), "medium", 1.5, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(label_key.to_string()),
            ));
            parent.spawn((
                add_text("", "bold", 3.0, assets),
                TextColor(Color::WHITE),
                StatLabel(stat),
            ));
        });
}

pub fn setup_playing_screen(
    mut commands: Commands,
    settings: Res<Settings>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    player: Res<Player>,
) {
    let lang = settings.language;

    commands.spawn((
        PlayingLog(vec![if lang == Language::Spanish {
            "¡Bienvenido a Arcana! Comienza tu aventura.".to_string()
        } else {
            "Welcome to Arcana! Start your adventure.".to_string()
        }]),
        PlayingCmp,
    ));

    let (mut root_node, pickable) = add_root_node(true);
    root_node.justify_content = JustifyContent::FlexStart;
    root_node.padding = UiRect::all(Val::Px(0.));
    root_node.row_gap = Val::Px(6.);

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode::new(assets.image("bg3")).with_mode(NodeImageMode::Stretch),
            PlayingCmp,
        ))
        .with_children(|parent| {
            // Character name, top centered.
            parent
                .spawn(Node {
                    width: percent(100.),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::vertical(Val::Px(6.)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(&player.name, "bold", TITLE_TEXT_SIZE, &assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            // Two main columns.
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_grow: 1.,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(8.),
                    padding: UiRect::horizontal(Val::Px(8.)),
                    ..default()
                })
                .with_children(|parent| {
                    // Left main section: Character column (Takes 49% width)
                    spawn_character_column(parent, &assets, &localization, lang, &player);

                    // Right main section: Scrollable equipment and abilities column (Takes 49% width)
                    spawn_right_column(parent, &assets, &localization, lang);
                });

            // Bottom row: Action buttons all in one single horizontal row (No logs panel)
            parent
                .spawn(Node {
                    width: percent(100.),
                    height: Val::Px(110.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    padding: UiRect::vertical(Val::Px(4.)),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_playing_action_button(parent, "hunt", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "shop", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "quest", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "train", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "craft", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "rest", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "inventory", &assets, &localization, lang);
                });
        });
}

fn spawn_character_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) {
    parent
        .spawn((
            Node {
                width: percent(48.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(10.)),
                row_gap: Val::Px(8.),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|parent| {
            // Expanded portrait (relative container for children slot/pet/title overlays)
            parent
                .spawn((
                    Node {
                        width: percent(100.), // Way larger!
                        aspect_ratio: Some(1.),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(3.)),
                        margin: UiRect::bottom(Val::Px(4.)),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(portrait_key(player)))
                        .with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    // Five equipment slots stacked on the top-right - made larger!
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(4.),
                            top: Val::Px(4.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in [
                                EquipSlot::Helmet,
                                EquipSlot::Weapon,
                                EquipSlot::Offhand,
                                EquipSlot::Armor,
                                EquipSlot::Boots,
                            ] {
                                parent.spawn((
                                    Node {
                                        width: Val::Px(50.),
                                        height: Val::Px(50.),
                                        border: UiRect::all(Val::Px(1.)),
                                        ..default()
                                    },
                                    BackgroundColor(PLACEHOLDER_COLOR),
                                    BorderColor::all(BUTTON_BORDER_COLOR),
                                    ImageNode::new(assets.image("stone"))
                                        .with_mode(NodeImageMode::Stretch),
                                    slot,
                                ));
                            }
                        });

                    // Pet image, bottom-left overlay
                    if player.pet.is_some() {
                        parent.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(3.),
                                bottom: Val::Px(3.),
                                width: percent(40.),
                                aspect_ratio: Some(1.),
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            BorderColor::all(BUTTON_BORDER_COLOR),
                            ImageNode::new(
                                assets.image(player.pet.unwrap().to_lowername()),
                            )
                            .with_mode(NodeImageMode::Stretch),
                            PetImage,
                        ));
                    }
                });

            // Level text (placed level line vertically below the portrait first)
            parent.spawn((
                add_text(class_line(player, localization, lang), "bold", 2.2, assets),
                TextColor(BUTTON_TEXT_COLOR),
                StatLabel(PlayingStat::ClassLine),
                Node {
                    margin: UiRect::vertical(Val::Px(2.)),
                    ..default()
                },
            ));

            // Health bar (thicker)
            spawn_bar(parent, assets, true);
            // Mana bar (thinner)
            spawn_bar(parent, assets, false);

            // Double Column layout: Characteristics and Attributes under the image
            parent.spawn(Node {
                width: percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            }).with_children(|parent| {
                // Column A: Characteristics
                parent.spawn(Node {
                    width: percent(48.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.),
                    ..default()
                }).with_children(|parent| {
                    parent.spawn((
                        add_text(localization.get("characteristics", lang), "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText("characteristics".to_string()),
                    ));
                    parent.spawn((
                        add_text(characteristics_text(player, localization, lang), "medium", 1.5, assets),
                        TextColor(Color::WHITE),
                        StatLabel(PlayingStat::RaceLine),
                    ));
                });

                // Column B: Attributes
                parent.spawn(Node {
                    width: percent(48.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.),
                    ..default()
                }).with_children(|parent| {
                    parent.spawn((
                        add_text(localization.get("attributes", lang), "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        LocalizedText("attributes".to_string()),
                    ));
                    parent.spawn(Node {
                        width: percent(100.),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.),
                        ..default()
                    }).with_children(|parent| {
                        for attr in Attribute::iter() {
                            parent.spawn(Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            }).with_children(|parent| {
                                parent.spawn((
                                    add_text(
                                        localization.get(&attr.to_lowername(), lang),
                                        "medium",
                                        1.5,
                                        assets,
                                    ),
                                    TextColor(Color::WHITE),
                                    LocalizedText(attr.to_lowername()),
                                ));
                                parent.spawn((
                                    add_text("", "bold", 1.5, assets),
                                    TextColor(BUTTON_TEXT_COLOR),
                                    AttrValue(attr),
                                ));
                            });
                        }
                    });
                });
            });

            // Combat stats: attack / armor / initiative.
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::top(Val::Px(6.)),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_combat_stat(parent, assets, localization, lang, "attack", "sword", PlayingStat::Attack);
                    spawn_combat_stat(parent, assets, localization, lang, "armor", "shield", PlayingStat::Armor);
                    spawn_combat_stat(parent, assets, localization, lang, "initiative", "boots_icon", PlayingStat::Initiative);
                });
        });
}

fn spawn_bar(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, is_health: bool) {
    let bar_height = if is_health { Val::Px(24.) } else { Val::Px(14.) };
    let font_size = if is_health { 1.5 } else { 1.1 };
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: bar_height,
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            // Fill.
            let mut fill = parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    top: Val::Px(0.),
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
                BackgroundColor(if is_health { HEALTH_COLOR } else { MANA_COLOR }),
            ));
            if is_health {
                fill.insert(HealthBarFill);
            } else {
                fill.insert(ManaBarFill);
            }

            // Text overlay.
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
                        StatLabel(if is_health { PlayingStat::Health } else { PlayingStat::Mana }),
                    ));
                });
        });
}

#[derive(Component)]
pub struct ScrollableContainer;

pub fn scroll_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut ScrollPosition, &Node), With<ScrollableContainer>>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll, _node) in &mut query {
            // Scroll offset speed factor
            scroll.y -= event.y * 30.0;
            if scroll.y < 0.0 {
                scroll.y = 0.0;
            }
        }
    }
}

fn spawn_right_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    parent
        .spawn((
            Node {
                width: percent(48.),
                height: percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(10.)),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: percent(100.),
                    height: percent(100.),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip(),
                    ..default()
                },
                ScrollableContainer,
                ScrollPosition::default(),
            )).with_children(|parent| {
                // Title row: "Equipment" on the left, gold icon + amount on the right.
                parent
                    .spawn(Node {
                        width: percent(100.),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        margin: UiRect::bottom(Val::Px(4.)),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            add_text(localization.get("equipment", lang), "bold", 2.0, assets),
                            TextColor(BUTTON_TEXT_COLOR),
                            LocalizedText("equipment".to_string()),
                        ));

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
                                        width: Val::Px(30.),
                                        height: Val::Px(30.),
                                        ..default()
                                    },
                                    ImageNode::new(assets.image("gold"))
                                        .with_mode(NodeImageMode::Stretch),
                                ));
                                parent.spawn((
                                    add_text("", "bold", 2.0, assets),
                                    TextColor(BUTTON_TEXT_COLOR),
                                    StatLabel(PlayingStat::Money),
                                ));
                            });
                    });

                // Dynamic equipment list
                parent.spawn((
                    Node {
                        width: percent(100.),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::bottom(Val::Px(15.)),
                        ..default()
                    },
                    EquipmentList,
                ));

                // Title: Abilities
                parent.spawn((
                    add_text(localization.get("abilities", lang), "bold", 2.0, assets),
                    TextColor(BUTTON_TEXT_COLOR),
                    LocalizedText("abilities".to_string()),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Node {
                        width: percent(100.),
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::bottom(Val::Px(15.)),
                        ..default()
                    },
                    AbilitiesList,
                ));

                // Title: Perks
                parent.spawn((
                    add_text(localization.get("perks", lang), "bold", 2.0, assets),
                    TextColor(BUTTON_TEXT_COLOR),
                    LocalizedText("perks".to_string()),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Node {
                        width: percent(100.),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    PerksList,
                ));
            });
        });
}

/// Rebuilds the equipment, abilities and perks lists whenever the player or
/// language changes (these have a variable number of entries).
pub fn rebuild_playing_lists(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    player: Res<Player>,
    equip_q: Query<Entity, With<EquipmentList>>,
    abil_q: Query<Entity, With<AbilitiesList>>,
    perk_q: Query<Entity, With<PerksList>>,
    mut slot_q: Query<(&EquipSlot, &mut ImageNode)>,
    children_q: Query<&Children>,
) {
    let lang = settings.language;

    // Update the five equipment image-slots on the portrait.
    for (slot, mut image) in &mut slot_q {
        let equipped = match slot {
            EquipSlot::Helmet => player.helmet,
            EquipSlot::Weapon => player.weapon_2h.or(player.weapon_lh),
            EquipSlot::Offhand => player.weapon_rh,
            EquipSlot::Armor => player.armor,
            EquipSlot::Boots => player.boots,
        };
        image.image = match equipped {
            Some(weapon) => assets.image(weapon.image_key()),
            None => assets.image("stone"),
        };
    }

    let clear = |commands: &mut Commands, entity: Entity, children_q: &Query<&Children>| {
        if let Ok(children) = children_q.get(entity) {
            for child in children.iter() {
                commands.entity(child).try_despawn();
            }
        }
    };

    // Equipment list.
    if let Ok(entity) = equip_q.single() {
        clear(&mut commands, entity, &children_q);
        commands.entity(entity).with_children(|parent| {
            let mut empty = true;
            let mut slots: Vec<&Weapon> = Vec::new();
            for slot in [
                &player.helmet,
                &player.armor,
                &player.boots,
                &player.weapon_lh,
                &player.weapon_rh,
                &player.weapon_2h,
            ] {
                if let Some(weapon) = slot {
                    slots.push(weapon);
                }
            }
            for weapon in slots {
                empty = false;
                spawn_card(
                    parent,
                    &assets,
                    localization.get(&weapon.to_lowername(), lang),
                    Some(weapon.to_lowername()),
                    weapon_stat_lines(weapon, &localization, lang),
                );
            }
            for consumable in &player.consumables {
                empty = false;
                spawn_card(
                    parent,
                    &assets,
                    localization.get(&consumable.to_lowername(), lang),
                    Some(consumable.to_lowername()),
                    vec![],
                );
            }
            if empty {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
    }

    // Abilities list.
    if let Ok(entity) = abil_q.single() {
        clear(&mut commands, entity, &children_q);
        commands.entity(entity).with_children(|parent| {
            if player.abilities.is_empty() {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            for ability in &player.abilities {
                let key = ability.to_lowername();
                spawn_card(
                    parent,
                    &assets,
                    localization.get(&key, lang),
                    Some(key.clone()),
                    vec![localization.get(&format!("{}_desc", key), lang)],
                );
            }
        });
    }

    // Perks list.
    if let Ok(entity) = perk_q.single() {
        clear(&mut commands, entity, &children_q);
        commands.entity(entity).with_children(|parent| {
            if player.perks.is_empty() {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            for perk in &player.perks {
                let key = perk.to_lowername();
                spawn_card(
                    parent,
                    &assets,
                    localization.get(&key, lang),
                    Some(key.clone()),
                    vec![localization.get(&format!("{}_desc", key), lang)],
                );
            }
        });
    }
}

pub fn update_playing_screen(
    player: Res<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut text_q: Query<(&mut Text, &StatLabel)>,
    mut attr_q: Query<(&mut Text, &AttrValue), Without<StatLabel>>,
    mut hbar_q: Query<&mut Node, (With<HealthBarFill>, Without<ManaBarFill>)>,
    mut mbar_q: Query<&mut Node, (With<ManaBarFill>, Without<HealthBarFill>)>,
    log_q: Query<&PlayingLog>,
    mut log_text_q: Query<&mut Text, (With<LogTextContainer>, Without<StatLabel>, Without<AttrValue>)>,
) {
    let lang = settings.language;

    for (mut text, stat) in &mut text_q {
        text.0 = match stat.0 {
            PlayingStat::ClassLine => class_line(&player, &localization, lang),
            PlayingStat::RaceLine => characteristics_text(&player, &localization, lang),
            PlayingStat::Health => format!(
                "{} / {} {}",
                player.health.max(0.) as i32,
                player.max_health() as i32,
                localization.get("health", lang)
            ),
            PlayingStat::Mana => format!(
                "{} / {} {}",
                player.mana.max(0.) as i32,
                player.max_mana() as i32,
                localization.get("mana", lang)
            ),
            PlayingStat::Money => format!("{} {}", player.money, localization.get("money", lang)),
            PlayingStat::Attack => format!("{}", player.attack_damage()),
            PlayingStat::Armor => format!("{}", player.armor_value()),
            PlayingStat::Initiative => format!("{}", player.initiative()),
        };
    }

    for (mut text, attr) in &mut attr_q {
        let val = match attr.0 {
            Attribute::Strength => player.strength(),
            Attribute::Dexterity => player.dexterity(),
            Attribute::Constitution => player.constitution(),
            Attribute::Intelligence => player.intelligence(),
            Attribute::Wisdom => player.wisdom(),
            Attribute::Charisma => player.charisma(),
        };
        text.0 = format!("{}", val);
    }

    if let Ok(mut node) = hbar_q.single_mut() {
        let ratio = (player.health / player.max_health()).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Ok(mut node) = mbar_q.single_mut() {
        let ratio = (player.mana / player.max_mana()).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }

    if let Some(log) = log_q.iter().next() {
        if let Some(mut text) = log_text_q.iter_mut().next() {
            text.0 = log.0.join("\n");
        }
    }
}

pub fn spawn_playing_action_button(
    parent: &mut ChildSpawnerCommands,
    action: &'static str,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    let action_label = localization.get(action, lang);
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(3.),
            margin: UiRect::all(Val::Px(2.)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(58.),
                        height: Val::Px(58.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.)),
                        border_radius: BorderRadius::all(percent(50.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(format!("action_{}", action)))
                        .with_mode(NodeImageMode::Stretch),
                    Button,
                    ActionButton(action),
                ))
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default));

            parent.spawn((
                add_text(action_label, "bold", 1.5, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(action.to_string()),
            ));
        });
}



pub fn handle_playing_action_clicks(
    mut player: ResMut<Player>,
    mut log_q: Query<&mut PlayingLog>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    interaction_q: Query<(&Interaction, &ActionButton), (Changed<Interaction>, With<Button>)>,
) {
    use rand::RngExt;
    let mut log_updated = false;
    let mut log_msgs = Vec::new();
    let lang = settings.language;

    for (interaction, action) in &interaction_q {
        if *interaction == Interaction::Pressed {
            play_audio_msg.write(PlayAudioMsg::new("button"));
            
            let cost_gold = match action.0 {
                "craft" => 15,
                "shop" => 30,
                "rest" => 10,
                _ => 0,
            };

            if player.money < cost_gold {
                play_audio_msg.write(PlayAudioMsg::new("error"));
                let err_msg = if lang == Language::Spanish {
                    format!("-�No hay suficiente oro para {}! (Necesitas {} oro)", action.0, cost_gold)
                } else {
                    format!("Not enough gold to {}! (Need {} gold)", action.0, cost_gold)
                };
                log_msgs.push(err_msg);
                log_updated = true;
                continue;
            }

            player.money -= cost_gold;

            // Handle the specific action
            let (ap_cost, log_msg) = match action.0 {
                "hunt" => {
                    let gold_earned = rand::rng().random_range(10..=20);
                    player.money += gold_earned;
                    let msg = if lang == Language::Spanish {
                        format!("Fuiste a cazar y obtuviste {} oro.", gold_earned)
                    } else {
                        format!("You went hunting and found {} gold.", gold_earned)
                    };
                    (2, msg)
                }
                "shop" => {
                    let items = vec![Consumable::HealingPotion, Consumable::ManaPotion, Consumable::PoisonVial, Consumable::HerbBlend];
                    use rand::seq::IndexedRandom;
                    let item = items.choose(&mut rand::rng()).copied().unwrap_or(Consumable::HealingPotion);
                    player.consumables.push(item);
                    let item_name = localization.get(&item.to_lowername(), lang);
                    let msg = if lang == Language::Spanish {
                        format!("Compraste una {} en la tienda.", item_name)
                    } else {
                        format!("You bought a {} from the shop.", item_name)
                    };
                    (1, msg)
                }
                "quest" => {
                    let gold_earned = rand::rng().random_range(20..=40);
                    let mut found_item = None;
                    if rand::rng().random_bool(0.5) {
                        let upgrade_types = vec![
                            Weapon::IronHelmet, Weapon::IronChestplate, Weapon::IronBoots, Weapon::SteelSword, 
                            Weapon::IronShield, Weapon::WizardStaff, Weapon::MageRobes, Weapon::ClothShoes, 
                            Weapon::LeatherArmor, Weapon::SilentBoots, Weapon::AssassinDagger, Weapon::ThiefDagger, 
                            Weapon::OakWand, Weapon::LeafyGarb
                        ];
                        use rand::seq::IndexedRandom;
                        if let Some(&upg) = upgrade_types.choose(&mut rand::rng()) {
                            found_item = Some(upg);
                            match upg {
                                Weapon::IronHelmet => player.helmet = Some(upg),
                                Weapon::IronChestplate | Weapon::MageRobes | Weapon::LeatherArmor | Weapon::LeafyGarb => player.armor = Some(upg),
                                Weapon::IronBoots | Weapon::ClothShoes | Weapon::SilentBoots | Weapon::LeatherBoots => player.boots = Some(upg),
                                Weapon::WizardStaff => {
                                    player.weapon_2h = Some(upg);
                                    player.weapon_lh = None;
                                    player.weapon_rh = None;
                                }
                                Weapon::SteelSword | Weapon::AssassinDagger | Weapon::ThiefDagger | Weapon::OakWand => {
                                    player.weapon_lh = Some(upg);
                                }
                                Weapon::IronShield => {
                                    player.weapon_rh = Some(upg);
                                }
                            }
                        }
                    }
                    player.money += gold_earned;
                    let msg = if let Some(item) = found_item {
                        let item_name = localization.get(&item.to_lowername(), lang);
                        if lang == Language::Spanish {
                            format!("-�Misi+�n completada! Ganaste {} oro y encontraste: {}.", gold_earned, item_name)
                        } else {
                            format!("Quest completed! Gained {} gold and found: {}.", gold_earned, item_name)
                        }
                    } else {
                        if lang == Language::Spanish {
                            format!("Misi+�n completada. Ganaste {} oro.", gold_earned)
                        } else {
                            format!("Quest completed. Gained {} gold.", gold_earned)
                        }
                    };
                    (3, msg)
                }
                "train" => {
                    let attr_idx = rand::rng().random_range(0..6);
                    let (attr_name, new_val) = match attr_idx {
                        0 => { player.strength += 1; (localization.get("strength", lang), player.strength) }
                        1 => { player.dexterity += 1; (localization.get("dexterity", lang), player.dexterity) }
                        2 => { player.constitution += 1; (localization.get("constitution", lang), player.constitution) }
                        3 => { player.intelligence += 1; (localization.get("intelligence", lang), player.intelligence) }
                        4 => { player.wisdom += 1; (localization.get("wisdom", lang), player.wisdom) }
                        _ => { player.charisma += 1; (localization.get("charisma", lang), player.charisma) }
                    };
                    let msg = if lang == Language::Spanish {
                        format!("Entrenaste duro. Tu {} aument+� a {}.", attr_name, new_val)
                    } else {
                        format!("You trained hard. Your {} increased to {}.", attr_name, new_val)
                    };
                    (2, msg)
                }
                "craft" => {
                    let items = vec![Weapon::IronHelmet, Weapon::IronChestplate, Weapon::IronBoots, Weapon::SteelSword, Weapon::IronShield];
                    use rand::seq::IndexedRandom;
                    let item = items.choose(&mut rand::rng()).copied().unwrap_or(Weapon::IronHelmet);
                    match item {
                        Weapon::IronHelmet => player.helmet = Some(item),
                        Weapon::IronChestplate => player.armor = Some(item),
                        Weapon::IronBoots => player.boots = Some(item),
                        Weapon::SteelSword => player.weapon_lh = Some(item),
                        Weapon::IronShield => player.weapon_rh = Some(item),
                        _ => {}
                    }
                    let item_name = localization.get(&item.to_lowername(), lang);
                    let msg = if lang == Language::Spanish {
                        format!("Fabricaste una {}.", item_name)
                    } else {
                        format!("You crafted a {}.", item_name)
                    };
                    (2, msg)
                }
                "rest" => {
                    player.health = player.max_health();
                    player.mana = player.max_mana();
                    let msg = if lang == Language::Spanish {
                        "Restauraste completamente tu salud.".to_string()
                    } else {
                        "You completely restored your health.".to_string()
                    };
                    (1, msg)
                }
                "inventory" => {
                    let msg = if lang == Language::Spanish {
                        "Abriste el inventario.".to_string()
                    } else {
                        "Opened your inventory.".to_string()
                    };
                    (0, msg)
                }
                _ => (0, "".to_string()),
            };

            log_msgs.push(log_msg);

            // Deduct action points
            if player.ap <= ap_cost {
                player.level += 1;
                player.ap = 10 + (player.level as u32) * 2;
                player.strength += 1;
                player.dexterity += 1;
                player.constitution += 1;
                player.intelligence += 1;
                player.wisdom += 1;
                player.charisma += 1;
                play_audio_msg.write(PlayAudioMsg::new("victory"));
                let lvl_msg = if lang == Language::Spanish {
                    format!("-�SUBIDA DE NIVEL! -�Has alcanzado el nivel {}!", player.level)
                } else {
                    format!("LEVEL UP! You reached Level {}!", player.level)
                };
                log_msgs.push(lvl_msg);
            } else {
                player.ap -= ap_cost;
            }

            log_updated = true;
        }
    }

    if log_updated {
        if let Ok(mut log) = log_q.single_mut() {
            for m in log_msgs {
                log.0.push(m);
                if log.0.len() > 3 {
                    log.0.remove(0);
                }
            }
        }
    }
}

