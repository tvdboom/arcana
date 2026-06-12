use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use strum::IntoEnumIterator;

pub use crate::core::actions::{handle_playing_action_clicks, Action, ActionButton};
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::build::wearable::WearableSlot;
use crate::core::build::equipment::Equipment;
use crate::core::build::modifiers::Modifier;
use crate::core::build::weapons::{Category, Hand};
use crate::core::catalog::{get_ability, get_equipment, get_perk};
use crate::core::classes::Class;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::buttons::DisabledButton;
use crate::core::menu::utils::{add_root_node, add_text, recolor, spawn_rich_text_row};
use crate::core::player::{Attribute, Player};
use crate::core::settings::{Language, Settings};
use crate::core::ui::creation::SelectionItem;
pub use crate::core::ui::level_up::{manage_level_up_overlay, LevelUpPending};
pub use crate::core::ui::toast::ToastContainer;
pub use crate::core::ui::tooltip::*;
use crate::core::ui::modal::{spawn_modal, ActiveModal, ModalAction};
use crate::core::utils::cursor;
use crate::utils::{capitalize_words, NameFromEnum};
use bevy::window::SystemCursorIcon;

const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const MANA_COLOR: Color = Color::srgb_u8(40, 80, 185);

// Viewport-relative icon sizes (scale with window width)
const ICON_ACTION: Val = Val::Vh(8.5); // action button circles
const ICON_BADGE: Val = Val::Vw(1.9); // equipped badge overlay
const ICON_STAT: Val = Val::Vw(2.4); // gold / AP stat icons

#[derive(Component)]
pub struct PlayingCmp;

/// Simple text stats that are refreshed every frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayingStat {
    ClassLine,
    CharRace,
    CharClass,
    CharSex,
    CharAge,
    CharHeight,
    CharWeight,
    Health,
    Mana,
    Money,
    Attack,
    Defense,
    Initiative,
    ActionPoints,
    PetHealth,
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
pub struct PetHealthBarFill;

#[derive(Component)]
pub struct EquipmentList;

#[derive(Component)]
pub struct AbilitiesList;

#[derive(Component)]
pub struct PerksList;

#[derive(Component)]
pub struct RightScrollbarTrack;

#[derive(Component)]
pub struct RightScrollbarThumb;

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum RightTab {
    #[default]
    Equipment,
    Abilities,
    Perks,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct RightTabBtn(pub RightTab);

#[derive(Component)]
pub struct EquipmentListWrapper;
#[derive(Component)]
pub struct AbilitiesListWrapper;
#[derive(Component)]
pub struct PerksListWrapper;

/// The equipment image-slots overlaid on the character portrait.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EquipSlot {
    Helmet,
    Chestplate,
    WeaponLH,
    WeaponRH,
    Gloves,
    Boots,
    Accessory,
    Accessory2,
}

#[derive(Component)]
pub struct PetImage;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InfoTooltip {
    Gold,
    ActionPoints,
    Combat(PlayingStat),
    Action(Action),
    Pet,
}

fn portrait_key(player: &Player) -> String {
    match player.class {
        Class::Mage(ajah) => ajah.get_image_key(player),
        _ => player.class.get_image_key(player),
    }
}

fn class_line(player: &Player, localization: &Localization, lang: Language) -> String {
    format!("{} {}", localization.get("general.level", lang), player.level)
}

fn playing_title(player: &Player) -> String {
    if let Some(ref pet) = player.pet {
        if !pet.name.trim().is_empty() {
            return format!("{} & {}", player.name, pet.name);
        }
    }
    player.name.clone()
}

fn localized_class_name(player: &Player, localization: &Localization, lang: Language) -> String {
    match player.class {
        Class::Mage(ajah) => format!(
            "{} {}",
            localization.get(&format!("ajah.{}", ajah.to_lowername()), lang),
            localization.get("class.mage", lang)
        ),
        _ => localization.get(&format!("class.{}", player.class.to_lowername()), lang),
    }
}

fn name_with_level(
    name: &str,
    prefix: &str,
    _level: u8,
    localization: &Localization,
    lang: Language,
) -> String {
    let key = format!("{}.{}", prefix, name.replace(" ", "_").to_lowercase());
    let raw_name = localization.get_opt(&key, lang).unwrap_or_else(|| name.to_string());
    capitalize_words(&raw_name)
}

fn signed_line(label: impl Into<String>, value: i32) -> String {
    let label = label.into();
    if value > 0 {
        format!("{}: +{}", label, value)
    } else {
        format!("{}: {}", label, value)
    }
}

fn weapon_bonus_lines(
    player: &Player,
    localization: &Localization,
    lang: Language,
    value_for: impl Fn(&Equipment) -> i32,
) -> Vec<String> {
    player
        .equipped_equipment()
        .into_iter()
        .filter_map(|weapon| {
            let value = value_for(&weapon);
            let prefix = match weapon {
                Equipment::Weapon(_) => "weapon",
                Equipment::Wearable(_) => "wearable",
            };
            let key = format!("{}.{}", prefix, weapon.name().replace(" ", "_").to_lowercase());
            let raw_name =
                localization.get_opt(&key, lang).unwrap_or_else(|| weapon.name().to_string());
            let localized_name = capitalize_words(&raw_name);
            (value != 0).then(|| format!("[equipment] {}", signed_line(localized_name, value)))
        })
        .collect()
}

fn combat_breakdown(
    stat: PlayingStat,
    player: &Player,
    localization: &Localization,
    lang: Language,
) -> Vec<String> {
    match stat {
        PlayingStat::Attack => {
            let mut lines = vec![format!("[base] {}", signed_line(localization.get("general.base", lang), 5))];
            lines.push(format!("[strength] {}", signed_line(
                localization.get("attribute.strength", lang),
                player.strength() as i32 - 10,
            )));
            lines.extend(weapon_bonus_lines(player, localization, lang, |weapon| weapon.attack()));
            lines.extend(perk_bonus_lines(player, localization, lang, stat));
            lines
        },
        PlayingStat::Defense => {
            let mut lines = vec![format!("[constitution] {}", signed_line(
                localization.get("attribute.constitution", lang),
                player.constitution() as i32 / 4,
            ))];
            lines.extend(weapon_bonus_lines(player, localization, lang, |weapon| weapon.defense()));
            lines.extend(perk_bonus_lines(player, localization, lang, stat));
            lines
        },
        PlayingStat::Initiative => {
            let mut lines = vec![format!("[dexterity] {}", signed_line(
                localization.get("attribute.dexterity", lang),
                player.dexterity() as i32 / 2,
            ))];
            lines.extend(weapon_bonus_lines(player, localization, lang, |weapon| {
                weapon.initiative()
            }));
            if matches!(player.class, Class::Assassin) {
                lines.push(format!("[assassin] {}", signed_line(localization.get("class.assassin", lang), 2)));
            }
            lines.extend(perk_bonus_lines(player, localization, lang, stat));
            lines
        },
        _ => vec![],
    }
}

fn perk_bonus_lines(
    player: &Player,
    localization: &Localization,
    lang: Language,
    stat: PlayingStat,
) -> Vec<String> {
    let mut lines = Vec::new();
    for perk_key in &player.perks {
        if let Some(perk) = get_perk(perk_key) {
            for modifier in &perk.modifiers {
                match (stat, modifier) {
                    (PlayingStat::Attack, Modifier::AttackModifier(val)) => {
                        let key = format!("perk.{}", perk.name.replace(" ", "_").to_lowercase());
                        let raw_name = localization
                            .get_opt(&key, lang)
                            .unwrap_or_else(|| perk.name.to_string());
                        let localized_name = capitalize_words(&raw_name);
                        lines.push(format!("[perk] {}", signed_line(localized_name, *val)));
                    },
                    (PlayingStat::Defense, Modifier::DefenseModifier(val)) => {
                        let key = format!("perk.{}", perk.name.replace(" ", "_").to_lowercase());
                        let raw_name = localization
                            .get_opt(&key, lang)
                            .unwrap_or_else(|| perk.name.to_string());
                        let localized_name = capitalize_words(&raw_name);
                        lines.push(format!("[perk] {}", signed_line(localized_name, *val)));
                    },
                    (PlayingStat::Initiative, Modifier::InitiativeModifier(val)) => {
                        let key = format!("perk.{}", perk.name.replace(" ", "_").to_lowercase());
                        let raw_name = localization
                            .get_opt(&key, lang)
                            .unwrap_or_else(|| perk.name.to_string());
                        let localized_name = capitalize_words(&raw_name);
                        lines.push(format!("[perk] {}", signed_line(localized_name, *val)));
                    },
                    _ => {},
                }
            }
        }
    }
    lines
}

/// A bordered placeholder box (used wherever an item/ability image will go later).
fn spawn_placeholder(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    image_key: &str,
    size: Val,
) {
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
        ImageNode::new(assets.image(format!("build_{}", image_key))).with_mode(NodeImageMode::Stretch),
    ));
}

/// A list entry: placeholder image on the left, a name and detail lines on the right.
fn spawn_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    image_key: &str,
    name: String,
    name_key: Option<String>,
    lines: Vec<String>,
    tooltip: Option<RightColumnTooltip>,
) {
    let mut cmd = parent.spawn((
        Node {
            width: percent(100.),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            flex_shrink: 0.,
            column_gap: Val::Px(8.),
            padding: UiRect::all(Val::Px(6.)),
            margin: UiRect::bottom(Val::Px(6.)),
            border: UiRect::all(Val::Px(1.)),
            ..default()
        },
        BackgroundColor(BAR_BG_COLOR),
        BorderColor::all(BUTTON_BORDER_COLOR),
    ));

    if let Some(t) = tooltip {
        cmd.insert((
            Button,
            Interaction::default(),
            Pickable::default(),
            t.clone(),
        ));
        cmd.observe(recolor::<Over>(HOVERED_BUTTON_COLOR));
        cmd.observe(recolor::<Out>(BAR_BG_COLOR));
        cmd.observe(cursor::<Over>(SystemCursorIcon::Pointer));
        cmd.observe(cursor::<Out>(SystemCursorIcon::Default));
        if let RightColumnTooltip::Perk(_) = t {
            cmd.observe(handle_perk_card_click);
        }
    }

    cmd.with_children(|parent| {
        spawn_placeholder(parent, assets, image_key, ICON_ITEM);

        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|parent| {
                let mut name_cmd = parent
                    .spawn((add_text(name, "bold", 2.3, assets), TextColor(BUTTON_TEXT_COLOR)));
                if let Some(key) = name_key {
                    name_cmd.insert(LocalizedText(key));
                }

                for line in lines {
                    spawn_rich_text_row(parent, assets, line, 2.0, "medium", Color::WHITE);
                }
            });
    });
}

pub fn handle_perk_card_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    card_q: Query<&RightColumnTooltip>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if event.button == PointerButton::Secondary {
        if let Ok(RightColumnTooltip::Perk(perk_name)) = card_q.get(event.entity) {
            let lang = settings.language;
            let action = ModalAction::RemovePerk {
                perk_name: perk_name.clone(),
            };
            spawn_modal(&mut commands, &assets, &localization, lang, action, &mut play_audio_msg);
        }
    }
}

/// One of the three combat-stat boxes (attack / defense / initiative).
fn spawn_combat_stat(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
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
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(PLACEHOLDER_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Interaction::default(),
            Pickable::default(),
            InfoTooltip::Combat(stat),
        ))
        .with_children(|parent| {
            // Semi-transparent icon background
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
                    image: assets.image(stat.to_lowername()),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(1., 1., 1., 0.3),
                    ..default()
                },
            ));
            
            let label_key = format!("general.{}", stat.to_lowername());
            parent.spawn((
                add_text(localization.get(&label_key, lang), "medium", 2.2, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(label_key),
            ));
            parent.spawn((
                add_text("", "bold", 4.5, assets),
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
    existing_screen_q: Query<Entity, With<PlayingCmp>>,
) {
    if existing_screen_q.iter().next().is_some() {
        return;
    }

    let lang = settings.language;

    let (mut root_node, pickable) = add_root_node(true);
    root_node.padding = UiRect::all(Val::Px(0.));

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode {
                image: assets.image("basebg"),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.40, 0.40, 0.40, 1.0),
                ..default()
            },
            PlayingCmp,
        ))
        .with_children(|parent| {
            // Toast container: stacks notifications top-right of the whole screen
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(8.),
                    right: Val::Percent(2.),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.),
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                PlayingCmp,
                ToastContainer,
                GlobalZIndex(600),
            ));

            // Content column: maintains aspect ratio on wide screens, centered horizontally
            parent
                .spawn(Node {
                    height: percent(100.),
                    max_width: percent(100.),
                    aspect_ratio: Some(16. / 9.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(4.),
                    align_self: AlignSelf::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // Character name, top centered with banner background.
                    parent
                        .spawn(Node {
                            align_self: AlignSelf::Center,
                            width: Val::Auto,
                            min_width: Val::Vh(50.0),
                            height: Val::Vh(7.11),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::horizontal(Val::Px(100.0)),
                            margin: UiRect {
                                top: Val::Vh(2.67),
                                bottom: Val::Vh(1.78),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            // Background banner image stretches to match parent's size
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: percent(100.0),
                                    height: percent(100.0),
                                    ..default()
                                },
                                ImageNode::new(assets.image("banner"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));

                            // Name text sits on top
                            parent.spawn((
                                add_text(playing_title(&player), "bold", 4.2, &assets),
                                TextColor(BUTTON_TEXT_COLOR),
                            ));
                        });

                    // Three main columns.
                    parent
                        .spawn(Node {
                            width: percent(100.),
                            height: percent(66.),
                            flex_shrink: 0.,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Stretch,
                            column_gap: Val::Px(0.),
                            padding: UiRect::horizontal(Val::Px(26.)),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Column 1: Character portrait image
                            spawn_image_column(parent, &assets, &player);

                            // Column 2: Stats (level, bars, characteristics, attributes, combat)
                            spawn_stats_column(parent, &assets, &localization, lang, &player);

                            // Column 3: Scrollable equipment, abilities and perks
                            spawn_right_column(parent, &assets, &localization, lang);
                        });

                    // Bottom row: Action buttons
                    parent
                        .spawn(Node {
                            width: percent(100.),
                            height: Val::Vh(14.5),
                            flex_shrink: 0.,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.),
                            padding: UiRect {
                                top: Val::Vh(1.5),
                                bottom: Val::Px(4.),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_playing_action_button(
                                parent,
                                Action::Rest,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Study,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Work,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Train,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Craft,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Shop,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Hunt,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Quest,
                                &assets,
                                &localization,
                                lang,
                            );
                            spawn_playing_action_button(
                                parent,
                                Action::Duel,
                                &assets,
                                &localization,
                                lang,
                            );
                        });
                });
        });
}

/// Column 1: Character portrait image with equipment slot overlays and pet.
fn spawn_image_column(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, player: &Player) {
    parent
        .spawn((Node {
            width: percent(33.5),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect::all(Val::Px(6.)),
            ..default()
        },))
        .with_children(|parent| {
            // Portrait (relative container for equipment slot / pet overlays)
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        aspect_ratio: Some(0.88),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(3.)),
                        margin: UiRect::top(Val::Px(2.)),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(portrait_key(player)))
                        .with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    // Left column: two accessory slots
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(2.),
                            top: Val::Percent(2.),
                            width: percent(16.),
                            height: percent(30.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            row_gap: Val::Px(1.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in [EquipSlot::Accessory, EquipSlot::Accessory2] {
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
                                        ImageNode::new(assets.image("stone"))
                                            .with_mode(NodeImageMode::Stretch),
                                        Interaction::default(),
                                        Button,
                                        Pickable::default(),
                                        slot,
                                    ))
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(handle_equipment_slot_click);
                            }
                        });

                    // Right column: 6 equipment slots
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            right: Val::Percent(2.),
                            top: Val::Percent(2.),
                            width: percent(16.),
                            height: percent(96.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            row_gap: Val::Px(1.),
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
                                        ImageNode::new(assets.image("stone"))
                                            .with_mode(NodeImageMode::Stretch),
                                        Interaction::default(),
                                        Button,
                                        Pickable::default(),
                                        slot,
                                    ))
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(handle_equipment_slot_click);
                            }
                        });

                    // Pet image, bottom-left overlay — larger
                    if player.pet.is_some() {
                        let pet = player.pet.as_ref().unwrap();
                        parent
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(3.),
                                    bottom: Val::Px(3.),
                                    width: percent(55.),
                                    aspect_ratio: Some(0.92),
                                    border: UiRect::all(Val::Px(2.)),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::FlexEnd,
                                    ..default()
                                },
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                ImageNode::new(assets.image(pet.kind.to_lowername()))
                                    .with_mode(NodeImageMode::Stretch),
                                PetImage,
                                Interaction::default(),
                                Pickable::default(),
                                InfoTooltip::Pet,
                            ))
                            .with_children(|parent| {
                                // Thicker health bar container at the bottom of the pet image
                                parent
                                    .spawn((
                                        Node {
                                            width: percent(90.),
                                            height: Val::Px(24.),
                                            border: UiRect::all(Val::Px(1.5)),
                                            align_self: AlignSelf::Center,
                                            margin: UiRect::bottom(Val::Px(4.)),
                                            position_type: PositionType::Relative,
                                            ..default()
                                        },
                                        BackgroundColor(BAR_BG_COLOR),
                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                    ))
                                    .with_children(|parent| {
                                        // Health bar fill
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
                                                    add_text("", "bold", 1.4, assets),
                                                    TextColor(Color::WHITE),
                                                    StatLabel(PlayingStat::PetHealth),
                                                ));
                                            });
                                    });
                            });
                    }
                });
        });
}

/// Column 2: Level, health/mana bars, characteristics, attributes, combat stats.
fn spawn_stats_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) {
    parent
        .spawn((Node {
            width: percent(32.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect {
                left: Val::Px(12.),
                right: Val::Px(12.),
                top: Val::Px(8.),
                bottom: Val::Px(8.),
            },
            row_gap: Val::Px(4.),
            overflow: Overflow::clip(),
            ..default()
        },))
        .with_children(|parent| {
            // Level / class text + AP on the right
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(class_line(player, localization, lang), "bold", 3.0, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        StatLabel(PlayingStat::ClassLine),
                    ));
                    parent
                        .spawn((
                            Node {
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
                                    width: ICON_STAT,
                                    height: ICON_STAT,
                                    flex_shrink: 0.,
                                    ..default()
                                },
                                ImageNode::new(assets.image("ap"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text(format!("{}", player.ap), "bold", 2.4, assets),
                                TextColor(Color::WHITE),
                                StatLabel(PlayingStat::ActionPoints),
                            ));
                        });
                });

            // Health bar
            spawn_bar(parent, assets, true);
            // Mana bar (same height as health)
            spawn_bar(parent, assets, false);

            // Spacer between bars and characteristics
            parent.spawn(Node {
                height: Val::Px(20.),
                ..default()
            });

            // Characteristics and Attributes side by side
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::Px(36.),
                    ..default()
                })
                .with_children(|parent| {
                    // Left: Characteristics
                    parent
                        .spawn(Node {
                            width: percent(45.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    localization.get("general.characteristics", lang),
                                    "bold",
                                    2.2,
                                    assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("general.characteristics".to_string()),
                            ));
                            parent
                                .spawn(Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    let char_rows = [
                                        ("general.race", PlayingStat::CharRace),
                                        ("general.class", PlayingStat::CharClass),
                                        ("general.sex", PlayingStat::CharSex),
                                        ("general.age", PlayingStat::CharAge),
                                        ("general.height", PlayingStat::CharHeight),
                                        ("general.weight", PlayingStat::CharWeight),
                                    ];
                                    for (key, stat) in char_rows {
                                        parent
                                            .spawn(Node {
                                                width: percent(100.),
                                                flex_direction: FlexDirection::Row,
                                                justify_content: JustifyContent::SpaceBetween,
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    add_text(
                                                        localization.get(key, lang),
                                                        "medium",
                                                        1.8,
                                                        assets,
                                                    ),
                                                    TextColor(Color::WHITE),
                                                    LocalizedText(key.to_string()),
                                                ));
                                                parent.spawn((
                                                    add_text("", "bold", 1.8, assets),
                                                    TextColor(Color::WHITE),
                                                    StatLabel(stat),
                                                ));
                                            });
                                    }
                                });
                        });

                    // Right: Attributes
                    parent
                        .spawn(Node {
                            width: percent(45.),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    localization.get("general.attributes", lang),
                                    "bold",
                                    2.2,
                                    assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("general.attributes".to_string()),
                            ));
                            parent
                                .spawn(Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    for attr in Attribute::iter() {
                                        parent
                                            .spawn(Node {
                                                width: percent(100.),
                                                flex_direction: FlexDirection::Row,
                                                justify_content: JustifyContent::SpaceBetween,
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    add_text(
                                                        localization.get(
                                                            &format!(
                                                                "attribute.{}",
                                                                attr.to_lowername()
                                                            ),
                                                            lang,
                                                        ),
                                                        "medium",
                                                        1.8,
                                                        assets,
                                                    ),
                                                    TextColor(Color::WHITE),
                                                    LocalizedText(format!(
                                                        "attribute.{}",
                                                        attr.to_lowername()
                                                    )),
                                                ));
                                                parent.spawn((
                                                    add_text("", "bold", 1.8, assets),
                                                    TextColor(Color::WHITE),
                                                    AttrValue(attr),
                                                ));
                                            });
                                    }
                                });
                        });
                });

            // Spacer between attributes and combat stats
            parent.spawn(Node {
                height: Val::Px(20.),
                ..default()
            });

            // Combat stats: attack / armor / initiative
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        PlayingStat::Attack,
                    );
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        PlayingStat::Defense,
                    );
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        PlayingStat::Initiative,
                    );
                });
        });
}

fn spawn_bar(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, is_health: bool) {
    let bar_height = Val::Px(36.);
    let font_size = 1.9;
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
                BackgroundColor(if is_health {
                    HEALTH_COLOR
                } else {
                    MANA_COLOR
                }),
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
                        StatLabel(if is_health {
                            PlayingStat::Health
                        } else {
                            PlayingStat::Mana
                        }),
                    ));
                });
        });
}

#[derive(Component)]
pub struct ScrollableContainer;

pub fn scroll_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
) {
    for event in mouse_wheel_events.read() {
        for (mut scroll, computed) in &mut query {
            // Scroll offset speed factor
            scroll.y -= event.y * 30.0;
            let max_scroll = (computed.content_size().y - computed.size().y).max(0.0);
            scroll.y = scroll.y.clamp(0.0, max_scroll);
        }
    }
}

fn on_right_scrollbar_thumb_drag(
    ev: On<Pointer<Drag>>,
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    track_q: Query<&ComputedNode, With<RightScrollbarTrack>>,
) {
    let Ok((mut scroll, scroll_node)) = scroll_q.single_mut() else {
        return;
    };
    let Ok(track_node) = track_q.single() else {
        return;
    };

    let viewport_height = scroll_node.size().y;
    let content_height = scroll_node.content_size().y;
    let max_scroll = (content_height - viewport_height).max(0.0);
    if max_scroll <= 0.0 || content_height <= 0.0 {
        scroll.y = 0.0;
        return;
    }

    let track_height = track_node.size().y;
    if track_height <= 1.0 {
        return;
    }
    let min_thumb_height = 32.0_f32.min(track_height);
    let thumb_height =
        (viewport_height / content_height * track_height).clamp(min_thumb_height, track_height);
    let max_thumb_top = (track_height - thumb_height).max(1.0);
    scroll.y = (scroll.y + ev.delta.y * max_scroll / max_thumb_top).clamp(0.0, max_scroll);
}

pub fn update_right_scrollbar_system(
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    mut track_q: Query<
        (&ComputedNode, &mut Visibility),
        (With<RightScrollbarTrack>, Without<RightScrollbarThumb>, Without<ScrollableContainer>),
    >,
    mut thumb_q: Query<
        &mut Node,
        (With<RightScrollbarThumb>, Without<RightScrollbarTrack>, Without<ScrollableContainer>),
    >,
) {
    let Ok((mut scroll, scroll_node)) = scroll_q.single_mut() else {
        return;
    };
    let Ok((track_computed, mut track_visibility)) = track_q.single_mut() else {
        return;
    };
    let Ok(mut thumb_node) = thumb_q.single_mut() else {
        return;
    };

    let viewport_height = scroll_node.size().y;
    let content_height = scroll_node.content_size().y;
    let max_scroll = (content_height - viewport_height).max(0.0);

    if max_scroll <= 1.0 || content_height <= viewport_height {
        scroll.y = 0.0;
        *track_visibility = Visibility::Hidden;
        return;
    }

    *track_visibility = Visibility::Visible;
    scroll.y = scroll.y.clamp(0.0, max_scroll);

    let track_height = track_computed.size().y;
    if track_height <= 1.0 {
        *track_visibility = Visibility::Hidden;
        return;
    }
    let min_thumb_height = 32.0_f32.min(track_height);
    let thumb_height =
        (viewport_height / content_height * track_height).clamp(min_thumb_height, track_height);
    let max_thumb_top = (track_height - thumb_height).max(0.0);
    let thumb_top = if max_scroll > 0.0 {
        scroll.y / max_scroll * max_thumb_top
    } else {
        0.0
    };

    thumb_node.height = Val::Px(thumb_height);
    thumb_node.top = Val::Px(thumb_top);
}

/// Shows/hides a tooltip when hovering over equipment slots.
pub fn equip_slot_tooltip_system(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    level_up: Res<LevelUpPending>,
    active_modal: Res<ActiveModal>,
    slot_q: Query<(&Interaction, &EquipSlot)>,
    changed_slot_q: Query<(), (With<EquipSlot>, Changed<Interaction>)>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if active_modal.active {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if level_up.active {
        return;
    }

    if changed_slot_q.is_empty() {
        return;
    }

    let mut hovered_slot = None;
    for (interaction, slot) in &slot_q {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            hovered_slot = Some(slot);
            break;
        }
    }

    for entity in tooltip_q.iter() {
        commands.entity(entity).try_despawn();
    }

    if let Some(slot) = hovered_slot {
        let lang = settings.language;
        let equipped_key = match slot {
            EquipSlot::Helmet => player.helmet.as_deref(),
            EquipSlot::Accessory => player.accessory.as_deref(),
            EquipSlot::Accessory2 => player.accessory2.as_deref(),
            EquipSlot::WeaponLH => player.weapon_lh.as_deref(),
            EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
            EquipSlot::Chestplate => player.armor.as_deref(),
            EquipSlot::Boots => player.boots.as_deref(),
            EquipSlot::Gloves => player.gloves.as_deref(),
        };

        if let Some(key) = equipped_key {
            if let Some(weapon) = get_equipment(key) {
                let prefix = match weapon {
                    Equipment::Weapon(_) => "weapon",
                    Equipment::Wearable(_) => "wearable",
                };
                let name = name_with_level(
                    weapon.name(),
                    prefix,
                    weapon.level() as u8,
                    &localization,
                    lang,
                );
                let stat_lines = weapon.full_description(lang, &localization);

                spawn_item_tooltip(
                    &mut commands,
                    &assets,
                    name,
                    stat_lines,
                    &windows,
                    Some(weapon.price()),
                    Some(weapon.name().to_string()),
                );
            }
        }
    }
}

#[derive(Component, Clone)]
pub enum RightColumnTooltip {
    Ability(String),
    Perk(String),
    Equipment(String),
}

pub fn right_column_tooltip_system(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    level_up: Res<LevelUpPending>,
    active_modal: Res<ActiveModal>,
    right_tab: Res<RightTab>,
    card_q: Query<(&Interaction, &RightColumnTooltip)>,
    changed_card_q: Query<(), (With<RightColumnTooltip>, Changed<Interaction>)>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if active_modal.active {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if changed_card_q.is_empty() {
        return;
    }

    let mut hovered_card = None;
    for (interaction, card) in &card_q {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let is_active_tab = if level_up.active {
                true
            } else {
                match card {
                    RightColumnTooltip::Ability(_) => *right_tab == RightTab::Abilities,
                    RightColumnTooltip::Perk(_) => *right_tab == RightTab::Perks,
                    RightColumnTooltip::Equipment(_) => *right_tab == RightTab::Equipment,
                }
            };
            if is_active_tab {
                hovered_card = Some(card);
                break;
            }
        }
    }

    for entity in tooltip_q.iter() {
        commands.entity(entity).try_despawn();
    }

    if let Some(card) = hovered_card {
        let lang = settings.language;
        match card {
            RightColumnTooltip::Ability(name) => {
                if let Some(ability) = get_ability(name) {
                    let title = name_with_level(
                        &ability.name,
                        "ability",
                        ability.level as u8,
                        &localization,
                        lang,
                    );
                    let lines = ability.full_description(lang, &localization);
                    spawn_item_tooltip(&mut commands, &assets, title, lines, &windows, None, Some(ability.name.clone()));
                }
            },
            RightColumnTooltip::Perk(name) => {
                if let Some(perk) = get_perk(name) {
                    let title = name_with_level(
                        &perk.name,
                        "perk",
                        perk.level as u8,
                        &localization,
                        lang,
                    );
                    let lines = perk.full_description(lang, &localization);
                    spawn_item_tooltip(&mut commands, &assets, title, lines, &windows, None, Some(perk.name.clone()));
                }
            },
            RightColumnTooltip::Equipment(name) => {
                if let Some(equipment) = get_equipment(name) {
                    let prefix = match equipment {
                        Equipment::Weapon(_) => "weapon",
                        Equipment::Wearable(_) => "wearable",
                    };
                    let title = name_with_level(
                        equipment.name(),
                        prefix,
                        equipment.level() as u8,
                        &localization,
                        lang,
                    );
                    let lines = equipment.full_description(lang, &localization);
                    spawn_item_tooltip(&mut commands, &assets, title, lines, &windows, Some(equipment.price()), Some(equipment.name().to_string()));
                }
            },
        }
    }
}

pub fn info_tooltip_system(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    level_up: Res<LevelUpPending>,
    active_modal: Res<ActiveModal>,
    info_q: Query<(&Interaction, &InfoTooltip)>,
    changed_info_q: Query<(), (With<InfoTooltip>, Changed<Interaction>)>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if active_modal.active {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if level_up.active {
        return;
    }

    if changed_info_q.is_empty() {
        return;
    }

    let mut hovered_tooltip = None;
    for (interaction, tooltip) in &info_q {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            hovered_tooltip = Some(tooltip);
            break;
        }
    }

    for entity in tooltip_q.iter() {
        commands.entity(entity).try_despawn();
    }

    if let Some(tooltip) = hovered_tooltip {
        let lang = settings.language;

        if matches!(tooltip, InfoTooltip::Pet) {
            spawn_pet_tooltip(&mut commands, &assets, &localization, lang, &player, &windows);
            return;
        }

        if let InfoTooltip::Action(action) = tooltip {
            let name = action.to_lowername();
            let action_name = localization.get(&format!("general.{name}"), lang);
            let desc = localization.get(&format!("general.{name}_desc"), lang);
            spawn_action_tooltip(
                &mut commands,
                &assets,
                action_name,
                action.ap_cost(),
                desc,
                &windows,
            );
            return;
        }

        let (title, lines) = match tooltip {
            InfoTooltip::Gold => (
                localization.get("general.gold", lang),
                vec![localization.get("general.gold_desc", lang)],
            ),
            InfoTooltip::ActionPoints => (
                localization.get("general.active_points", lang),
                vec![localization.get("general.active_points_desc", lang)],
            ),
            InfoTooltip::Combat(stat) => {
                let title_key = match stat {
                    PlayingStat::Attack => "general.attack",
                    PlayingStat::Defense => "general.defense",
                    PlayingStat::Initiative => "general.initiative",
                    _ => "",
                };
                (
                    localization.get(title_key, lang),
                    combat_breakdown(*stat, &player, &localization, lang),
                )
            },
            InfoTooltip::Action(_) | InfoTooltip::Pet => unreachable!(),
        };

        spawn_item_tooltip(&mut commands, &assets, title, lines, &windows, None, None);
    }
}

fn handle_tab_click(
    ev: On<Pointer<Click>>,
    btn_q: Query<&RightTabBtn>,
    mut right_tab: ResMut<RightTab>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(ev.entity) {
        if *right_tab != btn.0 {
            *right_tab = btn.0;
            play_audio_msg.write(PlayAudioMsg::new("button"));
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
        .spawn((Node {
            width: percent(33.5),
            height: percent(91.5),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect {
                left: Val::Px(8.),
                right: Val::Px(22.),
                top: Val::Px(8.),
                bottom: Val::Px(0.),
            },
            position_type: PositionType::Relative,
            ..default()
        },))
        .with_children(|parent| {
            // Tab row + gold icon at the top
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.,
                    margin: UiRect::bottom(Val::Px(4.)),
                    ..default()
                })
                .with_children(|parent| {
                    // Left: three tab buttons
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.),
                            ..default()
                        })
                        .with_children(|parent| {
                            for (tab, key, fallback) in [
                                (RightTab::Equipment, "equipment", "Equipment"),
                                (RightTab::Abilities, "abilities", "Abilities"),
                                (RightTab::Perks, "perks_tab", "Perks"),
                            ] {
                                let is_active = tab == RightTab::Equipment;
                                let bg_color = if is_active {
                                    NORMAL_BUTTON_COLOR
                                } else {
                                    Color::srgba_u8(20, 20, 35, 200)
                                };
                                let label = localization
                                    .get_opt(key, lang)
                                    .unwrap_or_else(|| fallback.to_string());
                                parent
                                    .spawn((
                                        Node {
                                            padding: UiRect::axes(Val::Px(10.), Val::Px(5.)),
                                            border: UiRect::all(Val::Px(1.)),
                                            ..default()
                                        },
                                        BackgroundColor(bg_color),
                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                        Button,
                                        Interaction::default(),
                                        Pickable {
                                            should_block_lower: true,
                                            is_hoverable: true,
                                        },
                                        RightTabBtn(tab),
                                    ))
                                    .observe(handle_tab_click)
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            add_text(label, "bold", 1.8, assets),
                                            TextColor(BUTTON_TEXT_COLOR),
                                            LocalizedText(key.to_string()),
                                        ));
                                    });
                            }
                        });

                    // Right: gold icon + amount
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
                                    width: ICON_STAT,
                                    height: ICON_STAT,
                                    ..default()
                                },
                                ImageNode::new(assets.image("gold"))
                                    .with_mode(NodeImageMode::Stretch),
                            ));
                            parent.spawn((
                                add_text("", "bold", 2.4, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                                StatLabel(PlayingStat::Money),
                            ));
                        });
                });

            // Separator between tabs and content list
            parent.spawn((
                Node {
                    width: percent(100.),
                    height: Val::Px(1.),
                    margin: UiRect {
                        top: Val::Px(10.),
                        bottom: Val::Px(18.),
                        ..default()
                    },
                    ..default()
                },
                BackgroundColor(BUTTON_BORDER_COLOR),
            ));

            // Scrollable content area
            parent
                .spawn((
                    Node {
                        width: percent(100.),
                        height: percent(100.),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollableContainer,
                    ScrollPosition::default(),
                ))
                .with_children(|parent| {
                    // Equipment wrapper (visible by default)
                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                ..default()
                            },
                            EquipmentListWrapper,
                            Visibility::Inherited,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    flex_shrink: 0.,
                                    margin: UiRect::bottom(Val::Px(15.)),
                                    ..default()
                                },
                                EquipmentList,
                            ));
                        });

                    // Abilities wrapper (hidden by default)
                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                display: Display::None,
                                ..default()
                            },
                            AbilitiesListWrapper,
                            Visibility::Hidden,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    flex_shrink: 0.,
                                    margin: UiRect::bottom(Val::Px(15.)),
                                    ..default()
                                },
                                AbilitiesList,
                            ));
                        });

                    // Perks wrapper (hidden by default)
                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                display: Display::None,
                                ..default()
                            },
                            PerksListWrapper,
                            Visibility::Hidden,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Node {
                                    width: percent(100.),
                                    flex_direction: FlexDirection::Column,
                                    flex_shrink: 0.,
                                    ..default()
                                },
                                PerksList,
                            ));
                        });
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(10.),
                        top: Val::Px(85.),
                        bottom: Val::Px(0.),
                        right: Val::Px(3.),
                        border_radius: BorderRadius::all(Val::Px(5.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 170)),
                    Visibility::Hidden,
                    RightScrollbarTrack,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(100.),
                                height: Val::Px(32.),
                                top: Val::Px(0.),
                                border_radius: BorderRadius::all(Val::Px(5.)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba_u8(230, 205, 120, 240)),
                            Button,
                            Interaction::default(),
                            Pickable::default(),
                            RightScrollbarThumb,
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(on_right_scrollbar_thumb_drag);
                });
        });
}

#[derive(SystemParam)]
pub struct RebuildPlayingListsQueries<'w, 's> {
    pub equip_q: Query<'w, 's, Entity, With<EquipmentList>>,
    pub abil_q: Query<'w, 's, Entity, With<AbilitiesList>>,
    pub perk_q: Query<'w, 's, Entity, With<PerksList>>,
    pub slot_q: Query<'w, 's, (&'static EquipSlot, &'static mut ImageNode)>,
    pub slot_vis_q: Query<'w, 's, (&'static EquipSlot, &'static mut Visibility)>,
    pub children_q: Query<'w, 's, &'static Children>,
    pub equip_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<EquipmentListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub abil_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<AbilitiesListWrapper>,
            Without<EquipmentListWrapper>,
            Without<PerksListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub perk_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<PerksListWrapper>,
            Without<EquipmentListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub tab_btn_q: Query<
        'w,
        's,
        (
            Entity,
            &'static RightTabBtn,
            &'static mut BackgroundColor,
            &'static mut BorderColor,
            &'static mut Node,
        ),
        (
            With<RightTabBtn>,
            Without<EquipmentListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
            Without<EquipSlot>,
        ),
    >,
    pub text_color_q: Query<'w, 's, &'static mut TextColor>,
    pub scroll_q: Query<'w, 's, &'static mut ScrollPosition, With<ScrollableContainer>>,
}

/// Rebuilds the equipment, abilities and perks lists whenever the player or
/// language changes (these have a variable number of entries).
pub fn rebuild_playing_lists(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    player: Res<Player>,
    right_tab: Res<RightTab>,
    mut queries: RebuildPlayingListsQueries<'_, '_>,
) {
    let lang = settings.language;

    let is_lh_two_hand = player
        .weapon_lh
        .as_deref()
        .and_then(|key| get_equipment(key))
        .map(|eq| match eq {
            Equipment::Weapon(w) => w.hand == Hand::TwoHand,
            _ => false,
        })
        .unwrap_or(false);

    // Update the equipment image-slots on the portrait.
    for (slot, mut image) in &mut queries.slot_q {
        let equipped_key = match slot {
            EquipSlot::Helmet => player.helmet.as_deref(),
            EquipSlot::Accessory => player.accessory.as_deref(),
            EquipSlot::Accessory2 => player.accessory2.as_deref(),
            EquipSlot::WeaponLH => player.weapon_lh.as_deref(),
            EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
            EquipSlot::Chestplate => player.armor.as_deref(),
            EquipSlot::Boots => player.boots.as_deref(),
            EquipSlot::Gloves => player.gloves.as_deref(),
        };
        image.image = match equipped_key {
            Some(key) => assets.image(format!("build_{}", key)),
            None => assets.image("stone"),
        };
    }

    // Show only filled slots; hide WeaponRH when a 2H weapon is also equipped
    for (slot, mut vis) in &mut queries.slot_vis_q {
        let visible = match slot {
            EquipSlot::Helmet => player.helmet.is_some(),
            EquipSlot::Accessory => player.accessory.is_some(),
            EquipSlot::Accessory2 => player.accessory2.is_some(),
            EquipSlot::WeaponLH => player.weapon_lh.is_some(),
            EquipSlot::WeaponRH => player.weapon_rh.is_some() && !is_lh_two_hand,
            EquipSlot::Chestplate => player.armor.is_some(),
            EquipSlot::Boots => player.boots.is_some(),
            EquipSlot::Gloves => player.gloves.is_some(),
        };
        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
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
    if let Ok(entity) = queries.equip_q.single() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            let mut empty = true;

            // Equipped items
            let equipped_slots = [
                (&player.helmet, "helmet"),
                (&player.accessory, "accessory"),
                (&player.accessory2, "accessory"),
                (&player.weapon_lh, "one_hand_weapon"),
                (&player.weapon_rh, "offhand"),
                (&player.armor, "armor"),
                (&player.boots, "boots"),
                (&player.gloves, "gloves"),
            ];

            // Collect equipped items and sort by level then name
            let mut equipped_items: Vec<Equipment> = equipped_slots
                .iter()
                .filter_map(|(slot_val, _)| slot_val.as_deref().and_then(get_equipment))
                .collect();
            equipped_items.sort_by(|a, b| a.level().cmp(&b.level()).then(a.name().cmp(b.name())));
            for weapon in &equipped_items {
                empty = false;
                let prefix = match weapon {
                    Equipment::Weapon(_) => "weapon",
                    Equipment::Wearable(_) => "wearable",
                };
                spawn_equipment_card(
                    parent,
                    &assets,
                    weapon.name(),
                    name_with_level(
                        weapon.name(),
                        prefix,
                        weapon.level() as u8,
                        &localization,
                        lang,
                    ),
                    vec![weapon.description(lang, &localization)],
                    EquipmentCard {
                        key: weapon.name().to_string(),
                        is_equipped: true,
                        price: weapon.price(),
                    },
                );
            }

            // Inventory items (unequipped), sorted by level then name
            let mut inventory_items: Vec<Equipment> =
                player.inventory.iter().filter_map(|key| get_equipment(key)).collect();
            inventory_items.sort_by(|a, b| a.level().cmp(&b.level()).then(a.name().cmp(b.name())));
            for weapon in &inventory_items {
                empty = false;
                let prefix = match weapon {
                    Equipment::Weapon(_) => "weapon",
                    Equipment::Wearable(_) => "wearable",
                };
                spawn_equipment_card(
                    parent,
                    &assets,
                    weapon.name(),
                    name_with_level(
                        weapon.name(),
                        prefix,
                        weapon.level() as u8,
                        &localization,
                        lang,
                    ),
                    vec![weapon.description(lang, &localization)],
                    EquipmentCard {
                        key: weapon.name().to_string(),
                        is_equipped: false,
                        price: weapon.price(),
                    },
                );
            }
            if empty {
                parent.spawn((
                    add_text(localization.get("general.none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
    }

    // Abilities list.
    if let Ok(entity) = queries.abil_q.single() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            if player.abilities.is_empty() {
                parent.spawn((
                    add_text(localization.get("general.none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            let mut sorted_abilities: Vec<_> =
                player.abilities.iter().filter_map(|key| get_ability(key)).collect();
            sorted_abilities.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for ability in &sorted_abilities {
                spawn_card(
                    parent,
                    &assets,
                    &ability.name,
                    name_with_level(
                        &ability.name,
                        "ability",
                        ability.level as u8,
                        &localization,
                        lang,
                    ),
                    None,
                    vec![ability.description(lang, &localization)],
                    Some(RightColumnTooltip::Ability(ability.name.clone())),
                );
            }
        });
    }

    // Perks list.
    if let Ok(entity) = queries.perk_q.single() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            if player.perks.is_empty() {
                parent.spawn((
                    add_text(localization.get("general.none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            let mut sorted_perks: Vec<_> =
                player.perks.iter().filter_map(|key| get_perk(key)).collect();
            sorted_perks.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for perk in &sorted_perks {
                spawn_card(
                    parent,
                    &assets,
                    &perk.name,
                    name_with_level(&perk.name, "perk", perk.level as u8, &localization, lang),
                    None,
                    vec![perk.description(lang, &localization)],
                    Some(RightColumnTooltip::Perk(perk.name.clone())),
                );
            }
        });
    }

    // Update wrapper visibility and display based on active tab.
    if let Ok((mut node, mut vis)) = queries.equip_wrap_q.single_mut() {
        if *right_tab == RightTab::Equipment {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }
    if let Ok((mut node, mut vis)) = queries.abil_wrap_q.single_mut() {
        if *right_tab == RightTab::Abilities {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }
    if let Ok((mut node, mut vis)) = queries.perk_wrap_q.single_mut() {
        if *right_tab == RightTab::Perks {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }

    // Reset scroll position to top if the tab was just changed.
    if right_tab.is_changed() {
        if let Ok(mut scroll) = queries.scroll_q.single_mut() {
            scroll.y = 0.0;
        }
    }

    // Update tab button background, border colors, text color of their children, and border thickness.
    for (entity, btn, mut bg, mut border, mut node) in &mut queries.tab_btn_q {
        let active = btn.0 == *right_tab;
        if active {
            *bg = BackgroundColor(NORMAL_BUTTON_COLOR);
            *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.2)); // Bright gold border
            node.border = UiRect::all(Val::Px(2.)); // Thicker border
        } else {
            *bg = BackgroundColor(Color::srgba_u8(12, 12, 18, 240)); // Much darker background
            *border = BorderColor::all(Color::srgba(0.2, 0.2, 0.2, 0.5)); // Muted border
            node.border = UiRect::all(Val::Px(1.));
        }

        if let Ok(children) = queries.children_q.get(entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = queries.text_color_q.get_mut(child) {
                    txt_col.0 = if active {
                        BUTTON_TEXT_COLOR // Bright gold for active
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.4) // Dimmed gray-white for inactive
                    };
                }
            }
        }
    }
}

pub fn tab_button_hover_system(
    right_tab: Res<RightTab>,
    mut tab_btn_q: Query<(Entity, &RightTabBtn, &Interaction, &mut BackgroundColor)>,
    children_q: Query<&Children>,
    mut text_color_q: Query<&mut TextColor>,
) {
    for (entity, btn, interaction, mut bg) in &mut tab_btn_q {
        let active = btn.0 == *right_tab;
        *bg = match (active, interaction) {
            (_, Interaction::Pressed) => BackgroundColor(Color::srgba_u8(30, 30, 50, 240)),
            (true, Interaction::Hovered) => BackgroundColor(NORMAL_BUTTON_COLOR),
            (false, Interaction::Hovered) => BackgroundColor(BUTTON_TEXT_COLOR),
            (true, Interaction::None) => BackgroundColor(NORMAL_BUTTON_COLOR),
            (false, Interaction::None) => BackgroundColor(Color::srgba_u8(12, 12, 18, 240)),
        };

        if let Ok(children) = children_q.get(entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = text_color_q.get_mut(child) {
                    txt_col.0 = match (active, interaction) {
                        (_, Interaction::Pressed) => Color::srgba(1.0, 1.0, 1.0, 0.4),
                        (true, _) => BUTTON_TEXT_COLOR,
                        (false, Interaction::Hovered) => Color::BLACK,
                        (false, Interaction::None) => Color::srgba(1.0, 1.0, 1.0, 0.4),
                    };
                }
            }
        }
    }
}

pub fn update_playing_screen(
    player: Res<Player>,
    settings: Res<Settings>,
    localization: Res<Localization>,
    mut text_q: Query<(&mut Text, &StatLabel)>,
    mut attr_q: Query<(&mut Text, &AttrValue), Without<StatLabel>>,
    mut hbar_q: Query<
        &mut Node,
        (With<HealthBarFill>, Without<ManaBarFill>, Without<PetHealthBarFill>),
    >,
    mut mbar_q: Query<
        &mut Node,
        (With<ManaBarFill>, Without<HealthBarFill>, Without<PetHealthBarFill>),
    >,
    mut pet_hbar_q: Query<
        &mut Node,
        (With<PetHealthBarFill>, Without<HealthBarFill>, Without<ManaBarFill>),
    >,
) {
    let lang = settings.language;

    for (mut text, stat) in &mut text_q {
        text.0 = match stat.0 {
            PlayingStat::ClassLine => class_line(&player, &localization, lang),
            PlayingStat::CharRace => {
                localization.get(&format!("race.{}", player.race.to_lowername()), lang)
            },
            PlayingStat::CharClass => localized_class_name(&player, &localization, lang),
            PlayingStat::CharSex => match player.sex {
                crate::core::player::Sex::Man => localization.get("general.man", lang),
                crate::core::player::Sex::Woman => localization.get("general.woman", lang),
            },
            PlayingStat::CharAge => {
                format!("{} {}", player.age, localization.get("general.years", lang))
            },
            PlayingStat::CharHeight => {
                let (height, _) = player.vitals();
                format!("{} cm", height)
            },
            PlayingStat::CharWeight => {
                let (_, weight) = player.vitals();
                format!("{} kg", weight)
            },
            PlayingStat::Health => format!(
                "{} / {} {}",
                player.health.max(0) as i32,
                player.max_health() as i32,
                localization.get("general.health", lang)
            ),
            PlayingStat::Mana => format!(
                "{} / {} {}",
                player.mana.max(0) as i32,
                player.max_mana() as i32,
                localization.get("general.mana", lang)
            ),
            PlayingStat::Money => format!("{}", player.gold),
            PlayingStat::Attack => format!("{}", player.attack()),
            PlayingStat::Defense => format!("{}", player.defense()),
            PlayingStat::Initiative => format!("{}", player.initiative()),
            PlayingStat::ActionPoints => format!("{}", player.ap),
            PlayingStat::PetHealth => {
                if let Some(ref pet) = player.pet {
                    let pet_max = pet.max_health as f32;
                    let pet_current = pet.health as f32;
                    format!(
                        "{} / {} {}",
                        pet_current.max(0.) as i32,
                        pet_max as i32,
                        localization.get("general.health", lang)
                    )
                } else {
                    "".to_string()
                }
            },
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
        let ratio = (player.health as f32 / player.max_health() as f32).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Ok(mut node) = mbar_q.single_mut() {
        let ratio = (player.mana as f32 / player.max_mana() as f32).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Some(ref pet) = player.pet {
        if let Ok(mut node) = pet_hbar_q.single_mut() {
            let pet_max = pet.max_health as f32;
            let pet_current = pet.health as f32;
            let ratio = (pet_current / pet_max).clamp(0., 1.) * 100.;
            node.width = percent(ratio);
        }
    }
}

pub fn highlight_border<E: std::fmt::Debug + Clone + Reflect>(
    color: Color,
    thickness: Val,
) -> impl Fn(On<Pointer<E>>, Query<(&mut BorderColor, &mut Node, Option<&DisabledButton>)>) {
    move |ev, mut q| {
        if let Ok((mut border_color, mut node, disabled)) = q.get_mut(ev.entity) {
            if disabled.is_none() {
                border_color.top = color;
                border_color.right = color;
                border_color.bottom = color;
                border_color.left = color;
                node.border = UiRect::all(thickness);
            }
        }
    }
}

pub fn spawn_playing_action_button(
    parent: &mut ChildSpawnerCommands,
    action: Action,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
) {
    let action_label = localization.get(action.to_lowername().as_str(), lang);
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(4.),
            margin: UiRect::horizontal(Val::Vw(0.9)),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: ICON_ACTION,
                        height: ICON_ACTION,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.)),
                        border_radius: BorderRadius::all(percent(50.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    ImageNode::new(assets.image(format!("action_{}", action.to_lowername())))
                        .with_mode(NodeImageMode::Stretch),
                    Button,
                    ActionButton(action),
                    InfoTooltip::Action(action),
                ))
                .observe(handle_playing_action_clicks)
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(highlight_border::<Over>(HOVERED_BUTTON_COLOR, Val::Px(3.)))
                .observe(highlight_border::<Out>(BUTTON_BORDER_COLOR, Val::Px(2.)))
                .observe(highlight_border::<Press>(Color::srgb_u8(240, 190, 60), Val::Px(3.)))
                .observe(highlight_border::<Release>(HOVERED_BUTTON_COLOR, Val::Px(3.)))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default));

            // Plain action name label below icon
            parent.spawn((
                add_text(action_label, "bold", 1.8, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(action.to_lowername().to_string()),
            ));
        });
}

#[derive(Component)]
pub struct EquipmentCard {
    pub key: String,
    pub is_equipped: bool,
    pub price: u32,
}

pub fn equip_item(player: &mut Player, key: &str) -> Option<&'static str> {
    let old_hp = player.max_health();
    let old_mp = player.max_mana();
    if let Some(equipment) = get_equipment(key) {
        // Remove from inventory first
        if let Some(pos) = player.inventory.iter().position(|k| k == key) {
            player.inventory.remove(pos);
        }

        match equipment {
            Equipment::Wearable(w) => match w.slot {
                WearableSlot::Consumable => {
                    let name = w.name.to_lowercase();
                    if name.contains("health") {
                        player.health = player.max_health();
                    } else if name.contains("mana") {
                        player.mana = player.max_mana();
                    } else if name.contains("strength") {
                        player.strength += 1;
                    } else if name.contains("dexterity") {
                        player.dexterity += 1;
                    } else if name.contains("constitution") {
                        player.constitution += 1;
                    } else if name.contains("intelligence") {
                        player.intelligence += 1;
                    } else if name.contains("wisdom") {
                        player.wisdom += 1;
                    } else if name.contains("charisma") {
                        player.charisma += 1;
                    } else if name.contains("rejuvenation") {
                        player.health = player.max_health();
                        player.mana = player.max_mana();
                    } else if name.contains("antidote") {
                        player.health = player.max_health();
                    }
                    player.adjust_health_mana_after_change(old_hp, old_mp);
                    return Some("button");
                },
                WearableSlot::Helmet => {
                    if let Some(old) = player.helmet.replace(key.to_string()) {
                        player.inventory.push(old);
                    }
                },
                WearableSlot::Chestplate => {
                    if let Some(old) = player.armor.replace(key.to_string()) {
                        player.inventory.push(old);
                    }
                },
                WearableSlot::Boots => {
                    if let Some(old) = player.boots.replace(key.to_string()) {
                        player.inventory.push(old);
                    }
                },
                WearableSlot::Gloves => {
                    if let Some(old) = player.gloves.replace(key.to_string()) {
                        player.inventory.push(old);
                    }
                },
                WearableSlot::Accessory => {
                    if player.accessory.is_none() {
                        player.accessory = Some(key.to_string());
                    } else if player.accessory2.is_none() {
                        player.accessory2 = Some(key.to_string());
                    } else {
                        if let Some(old) = player.accessory.replace(key.to_string()) {
                            player.inventory.push(old);
                        }
                    }
                },
            },
            Equipment::Weapon(w) => {
                let is_lh_two_hand = player
                    .weapon_lh
                    .as_deref()
                    .and_then(|k| get_equipment(k))
                    .map(|eq| match eq {
                        Equipment::Weapon(lh_w) => lh_w.hand == Hand::TwoHand,
                        _ => false,
                    })
                    .unwrap_or(false);

                if w.hand == Hand::TwoHand {
                    // Two-handed weapon: unequip both hands, then place in weapon_lh
                    if let Some(old_lh) = player.weapon_lh.take() {
                        player.inventory.push(old_lh);
                    }
                    if let Some(old_rh) = player.weapon_rh.take() {
                        player.inventory.push(old_rh);
                    }
                    player.weapon_lh = Some(key.to_string());
                } else if matches!(w.category, Category::Shield | Category::Book) {
                    // Unequip two-handed weapon from LH if present, then place in weapon_rh
                    if is_lh_two_hand {
                        if let Some(old_lh) = player.weapon_lh.take() {
                            player.inventory.push(old_lh);
                        }
                    }
                    if let Some(old) = player.weapon_rh.replace(key.to_string()) {
                        player.inventory.push(old);
                    }
                } else {
                    // One-handed weapon:
                    if is_lh_two_hand {
                        // LH has a two-hand weapon, unequip it
                        if let Some(old_lh) = player.weapon_lh.take() {
                            player.inventory.push(old_lh);
                        }
                    }
                    if player.weapon_lh.is_none() {
                        player.weapon_lh = Some(key.to_string());
                    } else if player.weapon_rh.is_none() {
                        player.weapon_rh = Some(key.to_string());
                    } else {
                        if let Some(old) = player.weapon_lh.replace(key.to_string()) {
                            player.inventory.push(old);
                        }
                    }
                }
            },
        }
    }
    player.adjust_health_mana_after_change(old_hp, old_mp);
    None
}

pub fn unequip_item(player: &mut Player, key: &str) {
    let old_hp = player.max_health();
    let old_mp = player.max_mana();
    let mut removed = false;
    if player.helmet.as_deref() == Some(key) {
        player.helmet = None;
        removed = true;
    } else if player.armor.as_deref() == Some(key) {
        player.armor = None;
        removed = true;
    } else if player.boots.as_deref() == Some(key) {
        player.boots = None;
        removed = true;
    } else if player.weapon_lh.as_deref() == Some(key) {
        player.weapon_lh = None;
        removed = true;
    } else if player.weapon_rh.as_deref() == Some(key) {
        player.weapon_rh = None;
        removed = true;
    } else if player.accessory.as_deref() == Some(key) {
        player.accessory = None;
        removed = true;
    } else if player.accessory2.as_deref() == Some(key) {
        player.accessory2 = None;
        removed = true;
    } else if player.gloves.as_deref() == Some(key) {
        player.gloves = None;
        removed = true;
    }
    if removed {
        player.inventory.push(key.to_string());
    }
    player.adjust_health_mana_after_change(old_hp, old_mp);
}

pub fn unequip_slot(player: &mut Player, slot: EquipSlot) -> bool {
    let old_hp = player.max_health();
    let old_mp = player.max_mana();
    let key_opt = match slot {
        EquipSlot::Helmet => player.helmet.take(),
        EquipSlot::Accessory => player.accessory.take(),
        EquipSlot::Accessory2 => player.accessory2.take(),
        EquipSlot::WeaponLH => player.weapon_lh.take(),
        EquipSlot::WeaponRH => player.weapon_rh.take(),
        EquipSlot::Chestplate => player.armor.take(),
        EquipSlot::Boots => player.boots.take(),
        EquipSlot::Gloves => player.gloves.take(),
    };
    let res = if let Some(key) = key_opt {
        player.inventory.push(key);
        true
    } else {
        false
    };
    player.adjust_health_mana_after_change(old_hp, old_mp);
    res
}

pub fn reward_equipment(player: &mut Player, key: String) {
    if let Some(equipment) = get_equipment(&key) {
        let is_empty = match equipment {
            Equipment::Wearable(w) => match w.slot {
                WearableSlot::Helmet => player.helmet.is_none(),
                WearableSlot::Chestplate => player.armor.is_none(),
                WearableSlot::Boots => player.boots.is_none(),
                WearableSlot::Gloves => player.gloves.is_none(),
                WearableSlot::Accessory => {
                    player.accessory.is_none() || player.accessory2.is_none()
                },
                WearableSlot::Consumable => false,
            },
            Equipment::Weapon(w) => {
                let is_lh_two_hand = player
                    .weapon_lh
                    .as_deref()
                    .and_then(|k| get_equipment(k))
                    .map(|eq| match eq {
                        Equipment::Weapon(lh_w) => lh_w.hand == Hand::TwoHand,
                        _ => false,
                    })
                    .unwrap_or(false);

                if w.hand == Hand::TwoHand {
                    player.weapon_lh.is_none() && player.weapon_rh.is_none()
                } else if matches!(w.category, Category::Shield | Category::Book) {
                    player.weapon_rh.is_none() && !is_lh_two_hand
                } else {
                    !is_lh_two_hand && (player.weapon_lh.is_none() || player.weapon_rh.is_none())
                }
            },
        };

        if is_empty {
            equip_item(player, &key);
        } else {
            player.inventory.push(key);
        }
    }
}

pub fn handle_equipment_card_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    right_tab: Res<RightTab>,
    card_q: Query<&EquipmentCard>,
) {
    if *right_tab != RightTab::Equipment {
        return;
    }
    if let Ok(card) = card_q.get(event.entity) {
        // Right-click: sell item for full price with confirmation modal
        if event.button == PointerButton::Secondary {
            let sell_price = card.price;
            let lang = settings.language;
            let action = ModalAction::SellItem {
                key: card.key.clone(),
                price: sell_price,
                is_equipped: card.is_equipped,
                slot: None,
            };
            spawn_modal(&mut commands, &assets, &localization, lang, action, &mut play_audio_msg);
            return;
        }

        // Left-click: equip/unequip
        if card.is_equipped {
            unequip_item(&mut player, &card.key);
            play_audio_msg.write(PlayAudioMsg::new("click"));
        } else {
            // Check consumable restrictions
            if let Some(eq) = get_equipment(&card.key) {
                if let Equipment::Wearable(ref w) = eq {
                    if w.slot == WearableSlot::Consumable {
                        let name = w.name.to_lowercase();
                        let blocked = if name.contains("rejuvenation") {
                            player.health >= player.max_health() && player.mana >= player.max_mana()
                        } else if name.contains("health") || name.contains("antidote") {
                            player.health >= player.max_health()
                        } else if name.contains("mana") {
                            player.mana >= player.max_mana()
                        } else {
                            false
                        };
                        if blocked {
                            play_audio_msg.write(PlayAudioMsg::new("error"));
                            return;
                        }
                    }
                }
            }
            let sound = equip_item(&mut player, &card.key).unwrap_or("click");
            play_audio_msg.write(PlayAudioMsg::new(sound));
        }
    }
}

pub fn handle_equipment_slot_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    slot_q: Query<&EquipSlot>,
) {
    if let Ok(slot) = slot_q.get(event.entity) {
        // Get the equipped item key for this slot
        let equipped_key = match slot {
            EquipSlot::Helmet => player.helmet.as_ref(),
            EquipSlot::Accessory => player.accessory.as_ref(),
            EquipSlot::Accessory2 => player.accessory2.as_ref(),
            EquipSlot::WeaponLH => player.weapon_lh.as_ref(),
            EquipSlot::WeaponRH => player.weapon_rh.as_ref(),
            EquipSlot::Chestplate => player.armor.as_ref(),
            EquipSlot::Boots => player.boots.as_ref(),
            EquipSlot::Gloves => player.gloves.as_ref(),
        };

        if let Some(key) = equipped_key {
            let key_str = key.to_string();

            // Right-click: sell equipped item for full price with confirmation modal
            if event.button == PointerButton::Secondary {
                if let Some(eq) = get_equipment(&key_str) {
                    let sell_price = eq.price();
                    let lang = settings.language;
                    let action = ModalAction::SellItem {
                        key: key_str.clone(),
                        price: sell_price,
                        is_equipped: true,
                        slot: Some(*slot),
                    };
                    spawn_modal(&mut commands, &assets, &localization, lang, action, &mut play_audio_msg);
                }
            } else {
                // Left-click: unequip item
                if unequip_slot(&mut player, *slot) {
                    play_audio_msg.write(PlayAudioMsg::new("click"));
                }
            }
        }
    }
}

fn spawn_equipment_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    image_key: &str,
    name: String,
    lines: Vec<String>,
    card: EquipmentCard,
) {
    let is_equipped = card.is_equipped;
    let price = card.price;
    let tooltip = RightColumnTooltip::Equipment(card.key.clone());
    parent
        .spawn((
            Node {
                width: percent(100.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                flex_shrink: 0.,
                column_gap: Val::Px(8.),
                padding: UiRect::all(Val::Px(6.)),
                margin: UiRect::bottom(Val::Px(6.)),
                border: UiRect::all(Val::Px(1.)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Button,
            Interaction::default(),
            Pickable::default(),
            card,
            tooltip,
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(BAR_BG_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(handle_equipment_card_click)
        .with_children(|parent| {
            // Top-right corner: equipped badge (if equipped) + price display
            parent
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(4.),
                    top: Val::Px(4.),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.),
                    ..default()
                },))
                .with_children(|parent| {
                    // Equipped badge (left of price)
                    if is_equipped {
                        parent.spawn((
                            Node {
                                width: ICON_BADGE,
                                height: ICON_BADGE,
                                ..default()
                            },
                            ImageNode::new(assets.image("equipped"))
                                .with_mode(NodeImageMode::Stretch),
                        ));
                    }

                    // Gold icon (same size as equipped badge)
                    parent.spawn((
                        Node {
                            width: ICON_BADGE,
                            height: ICON_BADGE,
                            ..default()
                        },
                        ImageNode::new(assets.image("gold")).with_mode(NodeImageMode::Stretch),
                    ));

                    // Price number (same color as weapon name)
                    parent.spawn((
                        add_text(format!("{}", price), "bold", 2.3, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });

            spawn_placeholder(parent, assets, image_key, ICON_ITEM);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((add_text(name, "bold", 2.3, assets), TextColor(BUTTON_TEXT_COLOR)));

                    for line in lines {
                        spawn_rich_text_row(parent, assets, line, 2.0, "medium", Color::WHITE);
                    }
                });
        });
}

pub fn update_action_buttons(
    player: Res<Player>,
    mut commands: Commands,
    mut btn_q: Query<(
        Entity,
        &ActionButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut ImageNode,
        Option<&DisabledButton>,
    )>,
) {
    for (entity, action_btn, mut bg, mut border, mut img, disabled) in &mut btn_q {
        let cost_ap = action_btn.0.ap_cost();
        let cost_gold = action_btn.0.gold_cost();
        let is_valid = player.ap >= cost_ap && player.gold >= cost_gold;

        if is_valid {
            if disabled.is_some() {
                commands.entity(entity).remove::<DisabledButton>();
                bg.0 = NORMAL_BUTTON_COLOR;
                *border = BorderColor::all(BUTTON_BORDER_COLOR);
                img.color = Color::WHITE;
            }
        } else {
            if disabled.is_none() {
                commands.entity(entity).insert(DisabledButton);
                bg.0 = DISABLED_BUTTON_COLOR;
                *border = BorderColor::all(DISABLED_BORDER_COLOR);
                img.color = Color::srgb(0.3, 0.3, 0.3);
            }
        }
    }
}
