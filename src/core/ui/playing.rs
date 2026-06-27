use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::core::actions::craft::CraftSeed;
pub use crate::core::actions::{handle_playing_action_clicks, Action, ActionButton};
use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::catalog::{get_ability, get_artifact, get_equipment, get_perk};
use crate::core::catalog::effects::Effect;
use crate::core::catalog::equipment::Equipment;
use crate::core::catalog::modifiers::Modifier;
use crate::core::catalog::weapons::{Category, Hand};
use crate::core::catalog::wearables::WearableSlot;
use crate::core::classes::Class;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::buttons::DisabledButton;
use crate::core::menu::utils::{add_root_node, add_text, recolor, spawn_rich_text_row};
use crate::core::player::{Attribute, Player};
use crate::core::settings::{Language, Settings};
use crate::core::states::GameState;
use crate::core::ui::creation::SelectionItem;
pub use crate::core::ui::level_up::{manage_level_up_overlay, LevelUpPending};
use crate::core::ui::modal::{spawn_modal, ActiveModal, ModalAction};
use crate::core::ui::scrollbar::{
    on_scrollbar_thumb_drag, ScrollableContainer, ScrollbarThumb, ScrollbarTrack,
};
pub use crate::core::ui::toast::ToastContainer;
pub use crate::core::ui::tooltip::*;
use crate::core::utils::cursor;
use crate::utils::{capitalize_words, NameFromEnum};
use bevy::window::{CursorIcon, SystemCursorIcon};
use std::path::Path;

const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const MANA_COLOR: Color = Color::srgb_u8(40, 80, 185);

// Viewport-relative icon sizes (scale with window width)
const ICON_ACTION: Val = Val::Vh(8.5); // action button circles
const ICON_BADGE: Val = Val::Vw(1.9); // equipped badge overlay

const EMPTY_SLOT_COLOR: Color = Color::srgba(0.08, 0.08, 0.14, 0.8);
const ACTIVE_HOTKEY_SLOT_SIZE: Val = Val::Vw(5.6);

/// Hotkey letters for the 5 ability slots.
const ABILITY_HOTKEYS: [&str; 5] = ["Q", "W", "E", "R", "T"];

#[derive(Component, Clone, Copy)]
pub struct ActiveHotkeySlot {
    pub index: usize,
}

#[derive(Component)]
pub struct DraggingSlot;

#[derive(Component)]
pub struct PrecombatDragGhost;
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
pub struct XpBarFill;

#[derive(Component)]
pub struct XpText;

#[derive(Component)]
pub struct ManaBarFill;

#[derive(Component)]
pub struct PetHealthBarFill;

#[derive(Component)]
pub struct EquipmentList;

#[derive(Component)]
pub struct ConsumablesList;

#[derive(Component)]
pub struct AbilitiesList;

#[derive(Component)]
pub struct PerksList;

#[derive(Resource, Default, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum RightTab {
    #[default]
    Equipment,
    Consumables,
    Abilities,
    Perks,
    Artifacts,
}

/// Remembers each right-panel tab's scroll offset so switching tabs and back
/// restores the previous scroll position instead of resetting to the top.
/// `pending` holds an offset waiting to be applied once the newly-shown tab's
/// content has actually been laid out (its `ComputedNode` content size is only
/// valid a frame or two after the wrapper toggles from `display: none`).
#[derive(Resource, Default)]
pub struct RightTabScroll {
    offsets: std::collections::HashMap<RightTab, f32>,
    current: RightTab,
    pending: Option<f32>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct RightTabBtn(pub RightTab);

#[derive(Component)]
pub struct EquipmentListWrapper;
#[derive(Component)]
pub struct ConsumablesListWrapper;
#[derive(Component)]
pub struct AbilitiesListWrapper;
#[derive(Component)]
pub struct PerksListWrapper;
#[derive(Component)]
pub struct ArtifactsListWrapper;
#[derive(Component)]
pub struct ArtifactsList;

/// Marker for the playing screen's right-column scroll viewport. Used to gate
/// hover/click on cards that are scrolled outside the visible (clipped) area.
#[derive(Component)]
pub struct RightColumnScroll;

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
    Xp,
    Combat(PlayingStat),
    #[allow(dead_code)]
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
    format!("{} {}", localization.get("general.level", lang), player.level())
}

fn playing_title(player: &Player) -> String {
    if let Some(ref pet) = player.pet {
        if !pet.name.trim().is_empty() {
            return format!("{} & {}", player.name, pet.name);
        }
    }
    player.name.clone()
}

fn pet_image_key(pet: &crate::core::monsters::Monster) -> String {
    Path::new(&pet.image).file_stem().and_then(|s| s.to_str()).unwrap_or(&pet.image).to_lowercase()
}

fn localized_class_name(player: &Player, localization: &Localization, lang: Language) -> String {
    match player.class {
        Class::Mage(ajah) => format!(
            "{} {}",
            localization.get(format!("ajah.{}", ajah.to_lowername()), lang),
            localization.get("class.mage", lang)
        ),
        _ => localization.get(format!("class.{}", player.class.to_lowername()), lang),
    }
}

pub fn name_with_level(
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
    if value >= 0 {
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
                Equipment::Consumable(_) => "consumable",
                Equipment::Artifact(_) => "artifact",
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
            let mut lines =
                vec![format!("[base] {}", signed_line(localization.get("general.base", lang), 5))];
            lines.push(format!(
                "[strength] {}",
                signed_line(localization.get("attribute.strength", lang), player.strength_mod())
            ));
            let training_bonus = player.training_bonus_for_skill("attack");
            if training_bonus > 0 {
                lines.push(format!(
                    "[training] {}",
                    signed_line(localization.get("general.training", lang), training_bonus as i32)
                ));
            }
            lines.extend(weapon_bonus_lines(player, localization, lang, |weapon| weapon.attack()));
            lines.extend(perk_bonus_lines(player, localization, lang, stat));
            lines
        },
        PlayingStat::Defense => {
            let mut lines =
                vec![format!("[base] {}", signed_line(localization.get("general.base", lang), 5))];
            lines.push(format!(
                "[constitution] {}",
                signed_line(
                    localization.get("attribute.constitution", lang),
                    player.constitution_mod()
                )
            ));
            let training_bonus = player.training_bonus_for_skill("defense");
            if training_bonus > 0 {
                lines.push(format!(
                    "[training] {}",
                    signed_line(localization.get("general.training", lang), training_bonus as i32)
                ));
            }
            lines.extend(weapon_bonus_lines(player, localization, lang, |weapon| weapon.defense()));
            lines.extend(perk_bonus_lines(player, localization, lang, stat));
            lines
        },
        PlayingStat::Initiative => {
            let mut lines =
                vec![format!("[base] {}", signed_line(localization.get("general.base", lang), 5))];
            lines.push(format!(
                "[dexterity] {}",
                signed_line(localization.get("attribute.dexterity", lang), player.dexterity_mod())
            ));
            let training_bonus = player.training_bonus_for_skill("initiative");
            if training_bonus > 0 {
                lines.push(format!(
                    "[training] {}",
                    signed_line(localization.get("general.training", lang), training_bonus as i32)
                ));
            }
            lines.extend(weapon_bonus_lines(player, localization, lang, |weapon| {
                weapon.initiative()
            }));
            if matches!(player.class, Class::Assassin) {
                lines.push(format!(
                    "[assassin] {}",
                    signed_line(localization.get("class.assassin", lang), 2)
                ));
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

fn clear_tooltips(commands: &mut Commands, tooltip_q: &Query<Entity, With<TooltipNode>>) {
    for entity in tooltip_q.iter() {
        commands.entity(entity).try_despawn();
    }
}

fn spawn_active_hotkey_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    localization: &Localization,
    settings: &Settings,
    player: &Player,
    slot_index: usize,
    tooltip_q: &Query<Entity, With<TooltipNode>>,
    windows: &Query<&Window>,
) {
    clear_tooltips(commands, tooltip_q);

    let lang = settings.language;
    let equipped_key = player.active_abilities.get(slot_index).and_then(|opt| opt.as_deref());
    if let Some(key) = equipped_key {
        if let Some(ability) = get_ability(key) {
            let title =
                name_with_level(&ability.name, "ability", ability.level as u8, &localization, lang);
            let lines = ability.full_description(lang, &localization);
            spawn_item_tooltip(
                commands,
                assets,
                title,
                lines,
                windows,
                None,
                Some(ability.name.clone()),
                0.0,
            );
        }
    }
}

/// Sets the cursor and tooltip for whichever active hotkey slot is currently
/// hovered (pointer + tooltip if it holds an ability, default otherwise).
fn refresh_hovered_hotkey_slot(
    commands: &mut Commands,
    assets: &WorldAssets,
    localization: &Localization,
    settings: &Settings,
    player: &Player,
    window_e: Entity,
    slot_state_q: &Query<(&bevy::ui::RelativeCursorPosition, &ActiveHotkeySlot)>,
    tooltip_q: &Query<Entity, With<TooltipNode>>,
    windows: &Query<&Window>,
) {
    let hovered =
        slot_state_q.iter().find_map(|(rel, slot)| rel.cursor_over().then_some(slot.index));

    let filled = hovered
        .filter(|idx| player.active_abilities.get(*idx).and_then(|opt| opt.as_ref()).is_some());

    if let Some(idx) = filled {
        commands.entity(window_e).insert(CursorIcon::from(SystemCursorIcon::Pointer));
        spawn_active_hotkey_tooltip(
            commands,
            assets,
            localization,
            settings,
            player,
            idx,
            tooltip_q,
            windows,
        );
    } else {
        commands.entity(window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
        clear_tooltips(commands, tooltip_q);
    }
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
        ImageNode::new(assets.image(format!("build_{}", image_key)))
            .with_mode(NodeImageMode::Stretch),
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
    is_equipped: bool,
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
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(BAR_BG_COLOR),
        BorderColor::all(BUTTON_BORDER_COLOR),
    ));

    if let Some(t) = tooltip {
        cmd.insert((Button, Interaction::default(), Pickable::default(), t.clone()));
        cmd.observe(recolor::<Over>(HOVERED_BUTTON_COLOR));
        cmd.observe(recolor::<Out>(BAR_BG_COLOR));
        cmd.observe(cursor::<Over>(SystemCursorIcon::Pointer));
        cmd.observe(cursor::<Out>(SystemCursorIcon::Default));
        if let RightColumnTooltip::Perk(_) = t {
            cmd.observe(handle_perk_card_click);
        }
        if let RightColumnTooltip::Ability(_) = t {
            cmd.observe(handle_active_ability_card_click);
        }
    }

    cmd.with_children(|parent| {
        if is_equipped {
            parent
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(4.),
                    top: Val::Px(4.),
                    overflow: Overflow::clip(),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: ICON_BADGE,
                            height: ICON_BADGE,
                            ..default()
                        },
                        ImageNode::new(assets.image("equipped")).with_mode(NodeImageMode::Stretch),
                    ));
                });
        }

        spawn_placeholder(parent, assets, image_key, ICON_ITEM);

        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
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
    scroll_q: Query<&bevy::ui::RelativeCursorPosition, With<RightColumnScroll>>,
) {
    if let Some(rel) = scroll_q.iter().next() {
        if !rel.cursor_over() {
            return;
        }
    }
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
pub fn spawn_combat_stat(
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
        .observe(crate::core::ui::utils::global_click_listener)
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
                GlobalZIndex(970),
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
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: percent(66.),
                                flex_shrink: 0.,
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Stretch,
                                column_gap: Val::Px(0.),
                                padding: UiRect::horizontal(Val::Px(26.)),
                                ..default()
                            },
                            crate::core::ui::utils::PlayScreenColumnsContainer,
                        ))
                        .with_children(|parent| {
                            // Column 1: Character portrait image
                            spawn_image_column(parent, &assets, &player, Some(GlobalZIndex(850)));

                            // Column 2: Stats (level, bars, characteristics, attributes, combat)
                            spawn_stats_column(parent, &assets, &localization, lang, &player);

                            // Column 3: Scrollable equipment, abilities and perks
                            spawn_right_column(parent, &assets, &localization, lang);
                        });

                    // Bottom row: Action buttons
                    parent
                        .spawn((
                            Node {
                                width: percent(100.),
                                height: Val::Vh(14.5),
                                flex_shrink: 0.,
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.),
                                padding: UiRect {
                                    top: Val::Vh(6.5),
                                    bottom: Val::Px(0.),
                                    ..default()
                                },
                                ..default()
                            },
                            GlobalZIndex(890),
                        ))
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
                            #[cfg(not(target_arch = "wasm32"))]
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
pub fn spawn_image_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    player: &Player,
    z_index: Option<GlobalZIndex>,
) {
    let mut cmd = parent.spawn(Node {
        width: percent(33.5),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        padding: UiRect::all(Val::Px(6.)),
        ..default()
    });

    if let Some(z) = z_index {
        cmd.insert(z);
    }

    cmd.with_children(|parent| {
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
                if let Some(pet) = &player.pet {
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
                            ImageNode::new(assets.image(pet_image_key(pet)))
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
pub fn spawn_stats_column(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
) {
    parent
        .spawn((
            Node {
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
            },
            crate::core::ui::utils::PlayScreenColumns2And3,
        ))
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
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    class_line(player, localization, lang),
                                    "bold",
                                    3.0,
                                    assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                StatLabel(PlayingStat::ClassLine),
                            ));

                            // Progress bar container (small bar)
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(80.),
                                        height: Val::Px(14.),
                                        border: UiRect::all(Val::Px(1.)),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::FlexStart,
                                        ..default()
                                    },
                                    BorderColor::all(BUTTON_BORDER_COLOR),
                                    BackgroundColor(BAR_BG_COLOR),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Node {
                                            width: percent(0.),
                                            height: percent(100.),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_TEXT_COLOR),
                                        XpBarFill,
                                    ));
                                });

                            // XP Text "X/10 XP"
                            parent
                                .spawn((
                                    Node {
                                        ..default()
                                    },
                                    Interaction::default(),
                                    Pickable::default(),
                                    InfoTooltip::Xp,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text("0/10 XP".to_string(), "medium", 1.8, assets),
                                        TextColor(Color::WHITE),
                                        XpText,
                                    ));
                                });
                        });
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(16.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Gold icon + text
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

                            // AP icon + text
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
                });

            // Health bar
            spawn_bar(parent, assets, true);
            // Mana bar (same height as health)
            spawn_bar(parent, assets, false);

            // Spacer between bars and characteristics
            parent.spawn(Node {
                height: Val::Px(8.),
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
                                                            format!(
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
                flex_grow: 1.0,
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
                    spawn_combat_stat(parent, assets, localization, lang, PlayingStat::Attack);
                    spawn_combat_stat(parent, assets, localization, lang, PlayingStat::Defense);
                    spawn_combat_stat(parent, assets, localization, lang, PlayingStat::Initiative);
                });

            // Spacer to push active abilities to the bottom!
            parent.spawn(Node {
                height: Val::Px(14.),
                ..default()
            });

            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    for index in 0..5 {
                        spawn_active_hotkey_slot(
                            parent,
                            assets,
                            index,
                            player.active_abilities.get(index).and_then(|opt| opt.as_deref()),
                            ABILITY_HOTKEYS[index],
                        );
                    }
                });
        });
}

pub fn spawn_bar(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, is_health: bool) {
    let bar_height = Val::Px(36.);
    let font_size = 1.9;
    parent
        .spawn((
            Node {
                width: percent(100.),
                height: bar_height,
                position_type: PositionType::Relative,
                border: UiRect::all(Val::Px(2.)),
                flex_shrink: 0.,
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
                    Equipment::Consumable(_) => "consumable",
                    Equipment::Artifact(_) => "artifact",
                };
                let name = name_with_level(
                    weapon.name(),
                    prefix,
                    weapon.level() as u8,
                    &localization,
                    lang,
                );
                let stat_lines = weapon.full_description(lang, &localization);
                let sell_price = weapon.sell_price(player.charisma_mod());

                spawn_item_tooltip(
                    &mut commands,
                    &assets,
                    name,
                    stat_lines,
                    &windows,
                    Some(sell_price),
                    Some(weapon.name().to_string()),
                    if matches!(weapon, Equipment::Artifact(_)) {
                        64.0
                    } else {
                        0.0
                    },
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
    _state: Res<State<GameState>>,
    player: Res<Player>,
    card_q: Query<(&Interaction, &RightColumnTooltip)>,
    changed_card_q: Query<(), (With<RightColumnTooltip>, Changed<Interaction>)>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
    ghost_q: Query<Entity, With<PrecombatDragGhost>>,
) {
    if active_modal.active || !ghost_q.is_empty() {
        clear_tooltips(&mut commands, &tooltip_q);
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
                matches!(*_state.get(), GameState::Combat)
                    || match card {
                        RightColumnTooltip::Ability(_) => *right_tab == RightTab::Abilities,
                        RightColumnTooltip::Perk(_) => *right_tab == RightTab::Perks,
                        RightColumnTooltip::Equipment(_) => {
                            *right_tab == RightTab::Equipment
                                || *right_tab == RightTab::Consumables
                                || *right_tab == RightTab::Artifacts
                        },
                    }
            };
            if is_active_tab {
                hovered_card = Some(card);
                break;
            }
        }
    }

    clear_tooltips(&mut commands, &tooltip_q);

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
                    spawn_item_tooltip(
                        &mut commands,
                        &assets,
                        title,
                        lines,
                        &windows,
                        None,
                        Some(ability.name.clone()),
                        0.0,
                    );
                }
            },
            RightColumnTooltip::Perk(name) => {
                if let Some(perk) = get_perk(name) {
                    let title =
                        name_with_level(&perk.name, "perk", perk.level as u8, &localization, lang);
                    let lines = perk.full_description(lang, &localization);
                    spawn_item_tooltip(
                        &mut commands,
                        &assets,
                        title,
                        lines,
                        &windows,
                        None,
                        Some(perk.name.clone()),
                        0.0,
                    );
                }
            },
            RightColumnTooltip::Equipment(name) => {
                if let Some(equipment) = get_equipment(name) {
                    let prefix = match equipment {
                        Equipment::Weapon(_) => "weapon",
                        Equipment::Wearable(_) => "wearable",
                        Equipment::Consumable(_) => "consumable",
                        Equipment::Artifact(_) => "artifact",
                    };
                    let title = name_with_level(
                        equipment.name(),
                        prefix,
                        equipment.level() as u8,
                        &localization,
                        lang,
                    );
                    let lines = equipment.full_description(lang, &localization);
                    let sell_price = equipment.sell_price(player.charisma_mod());
                    let extra_width = if matches!(equipment, Equipment::Artifact(_)) {
                        64.0
                    } else {
                        0.0
                    };
                    spawn_item_tooltip(
                        &mut commands,
                        &assets,
                        title,
                        lines,
                        &windows,
                        Some(sell_price),
                        Some(equipment.name().to_string()),
                        extra_width,
                    );
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
            let action_name = localization.get(format!("general.{name}"), lang);
            let desc = localization.get(format!("general.{name}_desc"), lang);
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
            InfoTooltip::Xp => (
                localization.get("general.xp", lang),
                vec![localization.get("general.xp_desc", lang)],
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

        spawn_item_tooltip(&mut commands, &assets, title, lines, &windows, None, None, 0.0);
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
        .spawn((
            Node {
                width: percent(33.5),
                height: percent(97.0),
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
            },
            crate::core::ui::utils::PlayScreenColumns2And3,
        ))
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
                            for (tab, key) in [
                                (RightTab::Equipment, "general.equipment"),
                                (RightTab::Consumables, "general.consumables"),
                                (RightTab::Artifacts, "general.artifacts"),
                                (RightTab::Abilities, "general.abilities"),
                                (RightTab::Perks, "general.perks"),
                            ] {
                                let is_active = tab == RightTab::Equipment;
                                let bg_color = if is_active {
                                    NORMAL_BUTTON_COLOR
                                } else {
                                    Color::srgba_u8(20, 20, 35, 200)
                                };
                                let label = localization.get(key, lang);
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
            let mut container_cmd = parent.spawn((
                Node {
                    width: percent(100.),
                    height: percent(100.),
                    min_height: Val::Px(0.),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollableContainer,
                ScrollPosition::default(),
                Interaction::default(),
                bevy::ui::RelativeCursorPosition::default(),
                RightColumnScroll,
            ));
            let container_entity = container_cmd.id();
            container_cmd.with_children(|parent| {
                // Equipment wrapper (visible by default)
                parent
                    .spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 0.,
                            overflow: Overflow::clip(),
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
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            EquipmentList,
                        ));
                    });

                // Consumables wrapper (hidden by default)
                parent
                    .spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 0.,
                            display: Display::None,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        ConsumablesListWrapper,
                        Visibility::Hidden,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                margin: UiRect::bottom(Val::Px(15.)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            ConsumablesList,
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
                            overflow: Overflow::clip(),
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
                                overflow: Overflow::clip(),
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
                            overflow: Overflow::clip(),
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
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            PerksList,
                        ));
                    });

                // Artifacts wrapper (hidden by default)
                parent
                    .spawn((
                        Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 0.,
                            display: Display::None,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        ArtifactsListWrapper,
                        Visibility::Hidden,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: percent(100.),
                                flex_direction: FlexDirection::Column,
                                flex_shrink: 0.,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            ArtifactsList,
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
                    ScrollbarTrack {
                        container: container_entity,
                    },
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
                            ScrollbarThumb {
                                container: container_entity,
                            },
                        ))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(on_scrollbar_thumb_drag);
                });
        });
}

#[derive(SystemParam)]
pub struct RebuildPlayingListsQueries<'w, 's> {
    pub equip_q: Query<'w, 's, Entity, With<EquipmentList>>,
    pub consumable_q: Query<'w, 's, Entity, With<ConsumablesList>>,
    pub abil_q: Query<'w, 's, Entity, With<AbilitiesList>>,
    pub perk_q: Query<'w, 's, Entity, With<PerksList>>,
    pub artifact_q: Query<'w, 's, Entity, With<ArtifactsList>>,
    pub slot_q: Query<'w, 's, (&'static EquipSlot, &'static mut ImageNode)>,
    pub slot_vis_q: Query<'w, 's, (&'static EquipSlot, &'static mut Visibility)>,
    pub children_q: Query<'w, 's, &'static Children>,
    pub equip_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<EquipmentListWrapper>,
            Without<ConsumablesListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
            Without<ArtifactsListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub consumable_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<ConsumablesListWrapper>,
            Without<EquipmentListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
            Without<ArtifactsListWrapper>,
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
            Without<ConsumablesListWrapper>,
            Without<PerksListWrapper>,
            Without<ArtifactsListWrapper>,
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
            Without<ConsumablesListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<ArtifactsListWrapper>,
            Without<EquipSlot>,
            Without<RightTabBtn>,
        ),
    >,
    pub artifact_wrap_q: Query<
        'w,
        's,
        (&'static mut Node, &'static mut Visibility),
        (
            With<ArtifactsListWrapper>,
            Without<EquipmentListWrapper>,
            Without<ConsumablesListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
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
            Without<ConsumablesListWrapper>,
            Without<AbilitiesListWrapper>,
            Without<PerksListWrapper>,
            Without<ArtifactsListWrapper>,
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
    mut tab_scroll: ResMut<RightTabScroll>,
    _game_state: Res<State<GameState>>,
    mut queries: RebuildPlayingListsQueries<'_, '_>,
) {
    let lang = settings.language;

    let is_lh_two_hand = player
        .weapon_lh
        .as_deref()
        .and_then(get_equipment)
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
        // Still hide WeaponRH if two-handed is equipped!
        let visible = if slot == &EquipSlot::WeaponRH && is_lh_two_hand {
            false
        } else {
            visible
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
                .filter(|eq| !matches!(eq, Equipment::Consumable(_) | Equipment::Artifact(_)))
                .collect();
            equipped_items.sort_by(|a, b| b.level().cmp(&a.level()).then(a.name().cmp(b.name())));
            for weapon in &equipped_items {
                empty = false;
                let prefix = match weapon {
                    Equipment::Weapon(_) => "weapon",
                    Equipment::Wearable(_) => "wearable",
                    Equipment::Consumable(_) => "consumable",
                    Equipment::Artifact(_) => "artifact",
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
                        price: weapon.sell_price(player.charisma_mod()),
                    },
                    false,
                );
            }

            // Inventory items (unequipped), sorted by level then name
            let mut inventory_items: Vec<Equipment> = player
                .inventory
                .iter()
                .filter_map(|key| get_equipment(key))
                .filter(|eq| !matches!(eq, Equipment::Consumable(_) | Equipment::Artifact(_)))
                .collect();
            inventory_items.sort_by(|a, b| b.level().cmp(&a.level()).then(a.name().cmp(b.name())));
            for weapon in &inventory_items {
                empty = false;
                let prefix = match weapon {
                    Equipment::Weapon(_) => "weapon",
                    Equipment::Wearable(_) => "wearable",
                    Equipment::Consumable(_) => "consumable",
                    Equipment::Artifact(_) => "artifact",
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
                        price: weapon.sell_price(player.charisma_mod()),
                    },
                    false,
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

    // Consumables list.
    if let Some(entity) = queries.consumable_q.iter().next() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            let mut empty = true;
            let mut consumables_map = std::collections::HashMap::new();
            for key in &player.inventory {
                if let Some(Equipment::Consumable(_)) = get_equipment(key) {
                    *consumables_map.entry(key.clone()).or_insert(0) += 1;
                }
            }

            let mut sorted_consumables_keys: Vec<String> =
                consumables_map.keys().cloned().collect();
            sorted_consumables_keys.sort_by(|a, b| {
                let eq_a = get_equipment(a).unwrap();
                let eq_b = get_equipment(b).unwrap();
                let a_equipped = player.is_consumable_equipped(a);
                let b_equipped = player.is_consumable_equipped(b);
                b_equipped
                    .cmp(&a_equipped)
                    .then(eq_b.level().cmp(&eq_a.level()))
                    .then(eq_a.name().cmp(eq_b.name()))
            });

            for key in &sorted_consumables_keys {
                empty = false;
                let weapon = get_equipment(key).unwrap();
                let count = consumables_map.get(key).unwrap();
                let is_equipped = player.is_consumable_equipped(key);
                let display_name = if *count > 1 {
                    format!(
                        "{} (x{})",
                        name_with_level(
                            weapon.name(),
                            "consumable",
                            weapon.level() as u8,
                            &localization,
                            lang,
                        ),
                        count
                    )
                } else {
                    name_with_level(
                        weapon.name(),
                        "consumable",
                        weapon.level() as u8,
                        &localization,
                        lang,
                    )
                };
                spawn_equipment_card(
                    parent,
                    &assets,
                    weapon.name(),
                    display_name,
                    vec![weapon.description(lang, &localization)],
                    EquipmentCard {
                        key: weapon.name().to_string(),
                        is_equipped,
                        price: weapon.sell_price(player.charisma_mod()),
                    },
                    false,
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

    // Artifacts list.
    if let Some(entity) = queries.artifact_q.iter().next() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            let mut empty = true;
            let mut artifacts_map = std::collections::HashMap::new();
            for key in &player.inventory {
                if let Some(Equipment::Artifact(_)) = get_equipment(key) {
                    *artifacts_map.entry(key.clone()).or_insert(0) += 1;
                }
            }

            let mut sorted_artifacts_keys: Vec<String> = artifacts_map.keys().cloned().collect();
            sorted_artifacts_keys.sort_by(|a, b| {
                let eq_a = get_equipment(a).unwrap();
                let eq_b = get_equipment(b).unwrap();
                eq_b.level().cmp(&eq_a.level()).then(eq_a.name().cmp(eq_b.name()))
            });

            for key in &sorted_artifacts_keys {
                empty = false;
                let weapon = get_equipment(key).unwrap();
                let count = artifacts_map.get(key).unwrap();
                let display_name = if *count > 1 {
                    format!(
                        "{} (x{})",
                        name_with_level(
                            weapon.name(),
                            "artifact",
                            weapon.level() as u8,
                            &localization,
                            lang,
                        ),
                        count
                    )
                } else {
                    name_with_level(
                        weapon.name(),
                        "artifact",
                        weapon.level() as u8,
                        &localization,
                        lang,
                    )
                };
                spawn_equipment_card(
                    parent,
                    &assets,
                    weapon.name(),
                    display_name,
                    vec![weapon.description(lang, &localization)],
                    EquipmentCard {
                        key: weapon.name().to_string(),
                        is_equipped: false,
                        price: weapon.sell_price(player.charisma_mod()),
                    },
                    false,
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
            sorted_abilities.sort_by(|a, b| {
                let pos_a =
                    player.active_abilities.iter().position(|x| x.as_ref() == Some(&a.name));
                let pos_b =
                    player.active_abilities.iter().position(|x| x.as_ref() == Some(&b.name));
                match (pos_a, pos_b) {
                    (Some(idx_a), Some(idx_b)) => idx_a.cmp(&idx_b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => b.level.cmp(&a.level).then(a.name.cmp(&b.name)),
                }
            });
            for ability in &sorted_abilities {
                let is_equipped = player.active_abilities.contains(&Some(ability.name.clone()));
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
                    is_equipped,
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
            sorted_perks.sort_by(|a, b| b.level.cmp(&a.level).then(a.name.cmp(&b.name)));
            for perk in &sorted_perks {
                spawn_card(
                    parent,
                    &assets,
                    &perk.name,
                    name_with_level(&perk.name, "perk", perk.level as u8, &localization, lang),
                    None,
                    vec![perk.description(lang, &localization)],
                    Some(RightColumnTooltip::Perk(perk.name.clone())),
                    false,
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
    if let Ok((mut node, mut vis)) = queries.consumable_wrap_q.single_mut() {
        if *right_tab == RightTab::Consumables {
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
    if let Ok((mut node, mut vis)) = queries.artifact_wrap_q.single_mut() {
        if *right_tab == RightTab::Artifacts {
            *vis = Visibility::Inherited;
            node.display = Display::Flex;
        } else {
            *vis = Visibility::Hidden;
            node.display = Display::None;
        }
    }

    // On tab change, save the outgoing tab's scroll offset and queue a restore of
    // the incoming tab's remembered offset. The actual restore is deferred to
    // `restore_tab_scroll` because the freshly-shown wrapper's content size is not
    // laid out yet this frame (so writing scroll.y now would be reset to 0 by the
    // scrollbar system once it sees content_size <= viewport).
    if right_tab.is_changed() {
        if let Ok(mut scroll) = queries.scroll_q.single_mut() {
            let new_tab = *right_tab;
            if tab_scroll.current != new_tab {
                let outgoing = tab_scroll.current;
                tab_scroll.offsets.insert(outgoing, scroll.y);
                tab_scroll.current = new_tab;
            }
            let target = tab_scroll.offsets.get(&new_tab).copied().unwrap_or(0.0);
            scroll.y = target;
            tab_scroll.pending = Some(target);
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

/// Applies a queued per-tab scroll offset once the newly-shown tab's content has
/// been laid out. Runs every frame; when `content_size` finally exceeds the
/// viewport (layout settled) it writes the saved offset and clears the request.
pub fn restore_tab_scroll(
    mut tab_scroll: ResMut<RightTabScroll>,
    mut scroll_q: Query<(&mut ScrollPosition, &ComputedNode), With<RightColumnScroll>>,
) {
    let Some(target) = tab_scroll.pending else {
        return;
    };

    if target <= 0.0 {
        tab_scroll.pending = None;
        return;
    }

    let Ok((mut scroll, node)) = scroll_q.single_mut() else {
        return;
    };

    let max_scroll = (node.content_size().y - node.size().y).max(0.0);
    if max_scroll > 0.0 {
        scroll.y = target.min(max_scroll);
        tab_scroll.pending = None;
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
        (With<HealthBarFill>, Without<ManaBarFill>, Without<PetHealthBarFill>, Without<XpBarFill>),
    >,
    mut mbar_q: Query<
        &mut Node,
        (With<ManaBarFill>, Without<HealthBarFill>, Without<PetHealthBarFill>, Without<XpBarFill>),
    >,
    mut pet_hbar_q: Query<
        &mut Node,
        (With<PetHealthBarFill>, Without<HealthBarFill>, Without<ManaBarFill>, Without<XpBarFill>),
    >,
    mut xp_bar_q: Query<
        &mut Node,
        (With<XpBarFill>, Without<HealthBarFill>, Without<ManaBarFill>, Without<PetHealthBarFill>),
    >,
    mut xp_text_q: Query<&mut Text, (With<XpText>, Without<StatLabel>, Without<AttrValue>)>,
) {
    let lang = settings.language;

    for (mut text, stat) in &mut text_q {
        text.0 = match stat.0 {
            PlayingStat::ClassLine => class_line(&player, &localization, lang),
            PlayingStat::CharRace => {
                localization.get(format!("race.{}", player.race.to_lowername()), lang)
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
                "{} / {} (+{}) {}",
                player.health() as i32,
                player.max_health() as i32,
                player.health_regen(),
                localization.get("general.health", lang)
            ),
            PlayingStat::Mana => format!(
                "{} / {} (+{}) {}",
                player.mana() as i32,
                player.max_mana() as i32,
                player.mana_regen(),
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
        let ratio = (player.health() as f32 / player.max_health() as f32).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Ok(mut node) = mbar_q.single_mut() {
        let ratio = (player.mana() as f32 / player.max_mana() as f32).clamp(0., 1.) * 100.;
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

    let xp_progress = player.xp % 10;
    if let Ok(mut node) = xp_bar_q.single_mut() {
        let ratio = (xp_progress as f32 / 10.0).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Ok(mut text) = xp_text_q.single_mut() {
        text.0 = format!("{}/10 XP", xp_progress);
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
                ))
                .observe(handle_playing_action_clicks)
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(highlight_border::<Over>(HOVERED_BUTTON_COLOR, Val::Px(3.)))
                .observe(highlight_border::<Out>(BUTTON_BORDER_COLOR, Val::Px(2.)))
                .observe(highlight_border::<Press>(Color::srgb_u8(240, 190, 60), Val::Px(3.)))
                .observe(highlight_border::<Release>(HOVERED_BUTTON_COLOR, Val::Px(3.)))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .observe(cursor::<Release>(SystemCursorIcon::Pointer));

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
            Equipment::Consumable(c) => {
                for effect in &c.effects {
                    match effect {
                        Effect::Heal {
                            heal_pct,
                        } => {
                            let max_hp = player.max_health();
                            let heal_amount = (max_hp * heal_pct) / 100;
                            player.set_health(player.health() + heal_amount);
                        },
                        Effect::InstantMana {
                            amount,
                        } => {
                            player.set_mana(player.mana() + amount);
                        },
                        Effect::StatBoost {
                            attribute,
                            amount,
                            ..
                        } => match attribute {
                            Attribute::Strength => player.strength += amount,
                            Attribute::Dexterity => player.dexterity += amount,
                            Attribute::Constitution => player.constitution += amount,
                            Attribute::Intelligence => player.intelligence += amount,
                            Attribute::Wisdom => player.wisdom += amount,
                            Attribute::Charisma => player.charisma += amount,
                        },
                        _ => {},
                    }
                }
                if !player.inventory.iter().any(|inv| inv == key) {
                    player.equipped_consumables.retain(|eq| eq != key);
                }
                player.update_health_mana(old_hp, old_mp);
                return Some("button");
            },
            Equipment::Wearable(w) => match w.slot {
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
                    .and_then(get_equipment)
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
            Equipment::Artifact(_) => {
                // Cannot be equipped, put it back in inventory and return None
                player.inventory.push(key.to_string());
            },
        }
    }
    player.update_health_mana(old_hp, old_mp);
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
    player.update_health_mana(old_hp, old_mp);
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
    player.update_health_mana(old_hp, old_mp);
    res
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
    mut craft_seed: ResMut<CraftSeed>,
    mut next_game_state: ResMut<NextState<GameState>>,
    _game_state: Option<Res<State<GameState>>>,
    card_q: Query<&EquipmentCard>,
    scroll_q: Query<&bevy::ui::RelativeCursorPosition, With<RightColumnScroll>>,
) {
    if let Some(rel) = scroll_q.iter().next() {
        if !rel.cursor_over() {
            return;
        }
    }
    if *right_tab != RightTab::Equipment
        && *right_tab != RightTab::Consumables
        && *right_tab != RightTab::Artifacts
    {
        return;
    }
    if let Ok(card) = card_q.get(event.entity) {
        if *right_tab == RightTab::Consumables && event.button != PointerButton::Secondary {
            if card.is_equipped {
                player.equipped_consumables.retain(|k| k != &card.key);
                play_audio_msg.write(PlayAudioMsg::new("click"));
            } else if player.toggle_consumable_equipped(&card.key) {
                play_audio_msg.write(PlayAudioMsg::new("click"));
            } else {
                play_audio_msg.write(PlayAudioMsg::new("error"));
            }
            return;
        }

        if *right_tab == RightTab::Artifacts {
            // Right-click sells the artifact.
            if event.button == PointerButton::Secondary {
                let sell_price = card.price;
                let lang = settings.language;
                let action = ModalAction::SellItem {
                    key: card.key.clone(),
                    price: sell_price,
                    is_equipped: card.is_equipped,
                    slot: None,
                };
                spawn_modal(
                    &mut commands,
                    &assets,
                    &localization,
                    lang,
                    action,
                    &mut play_audio_msg,
                );
                return;
            }
            // Left-click opens the craft panel with this artifact on the bench.
            craft_seed.artifacts = vec![card.key.clone()];
            play_audio_msg.write(PlayAudioMsg::new("button"));
            next_game_state.set(GameState::Craft);
            return;
        }

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
            if let Some(Equipment::Consumable(ref c)) = get_equipment(&card.key) {
                let mut has_heal = false;
                let mut has_mana = false;
                for effect in &c.effects {
                    match effect {
                        Effect::Heal {
                            ..
                        } => has_heal = true,
                        Effect::InstantMana {
                            ..
                        } => has_mana = true,
                        _ => {},
                    }
                }

                let blocked = if has_heal && has_mana {
                    player.missing_health == 0 && player.missing_mana == 0
                } else if has_heal {
                    player.missing_health == 0
                } else if has_mana {
                    player.missing_mana == 0
                } else {
                    false
                };

                if blocked {
                    play_audio_msg.write(PlayAudioMsg::new("error"));
                    return;
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
    game_state: Option<Res<State<GameState>>>,
    slot_q: Query<&EquipSlot>,
) {
    if let Some(state) = game_state {
        if *state.get() == GameState::Combat {
            return;
        }
    }

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
                    let sell_price = eq.sell_price(player.charisma_mod());
                    let lang = settings.language;
                    let action = ModalAction::SellItem {
                        key: key_str.clone(),
                        price: sell_price,
                        is_equipped: true,
                        slot: Some(*slot),
                    };
                    spawn_modal(
                        &mut commands,
                        &assets,
                        &localization,
                        lang,
                        action,
                        &mut play_audio_msg,
                    );
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

pub fn spawn_equipment_card<'a>(
    parent: &'a mut ChildSpawnerCommands,
    assets: &WorldAssets,
    image_key: &str,
    name: String,
    lines: Vec<String>,
    card: EquipmentCard,
    highlight_equipped: bool,
) -> EntityCommands<'a> {
    let is_equipped = card.is_equipped;
    let price = card.price;
    let card_key = card.key.clone();
    let tooltip = RightColumnTooltip::Equipment(card.key.clone());
    let out_color = if is_equipped && highlight_equipped {
        SELECTED_COLOR
    } else {
        BAR_BG_COLOR
    };
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
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(out_color),
        BorderColor::all(BUTTON_BORDER_COLOR),
        Button,
        Interaction::default(),
        Pickable::default(),
        card,
        tooltip,
    ));
    cmd.observe(recolor::<Over>(HOVERED_BUTTON_COLOR));
    cmd.observe(recolor::<Out>(out_color));
    cmd.observe(cursor::<Over>(SystemCursorIcon::Pointer));
    cmd.observe(cursor::<Out>(SystemCursorIcon::Default));
    cmd.observe(handle_equipment_card_click);
    cmd.with_children(|parent| {
        // Top-right corner: equipped badge (if equipped) + price display
        parent
            .spawn((Node {
                position_type: PositionType::Absolute,
                right: Val::Px(4.),
                top: Val::Px(4.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.),
                overflow: Overflow::clip(),
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
                        ImageNode::new(assets.image("equipped")).with_mode(NodeImageMode::Stretch),
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
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn((add_text(name, "bold", 2.3, assets), TextColor(BUTTON_TEXT_COLOR)));

                if let Some(artifact) = get_artifact(&card_key) {
                    let k = artifact.kind;
                    spawn_rich_text_row(
                        parent,
                        assets,
                        format!("[{}] {}", k.to_string().to_lowercase(), k),
                        2.0,
                        "medium",
                        Color::WHITE,
                    );
                } else {
                    for line in lines {
                        spawn_rich_text_row(parent, assets, line, 2.0, "medium", Color::WHITE);
                    }
                }
            });
    });
    cmd
}

pub fn update_action_buttons(
    _player: Res<Player>,
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
    for (entity, _action_btn, mut bg, mut border, mut img, disabled) in &mut btn_q {
        if disabled.is_some() {
            commands.entity(entity).remove::<DisabledButton>();
            bg.0 = NORMAL_BUTTON_COLOR;
            *border = BorderColor::all(BUTTON_BORDER_COLOR);
            img.color = Color::WHITE;
        }
    }
}

pub fn handle_active_ability_card_click(
    event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_q: Query<&RightColumnTooltip>,
    scroll_q: Query<&bevy::ui::RelativeCursorPosition, With<RightColumnScroll>>,
) {
    // Reject clicks whose cursor isn't actually over the visible (non-clipped)
    // scroll viewport. Scrolled-out cards can remain pickable at screen positions
    // outside the container, which would otherwise select them.
    if let Some(rel) = scroll_q.iter().next() {
        if !rel.cursor_over() {
            return;
        }
    }

    let Ok(RightColumnTooltip::Ability(ability_name)) = card_q.get(event.entity) else {
        return;
    };

    let is_currently_equipped = player.active_abilities.contains(&Some(ability_name.clone()));

    match event.button {
        // Right-click removes the ability from its active slot.
        PointerButton::Secondary => {
            if let Some(pos) =
                player.active_abilities.iter().position(|x| x.as_ref() == Some(ability_name))
            {
                player.active_abilities[pos] = None;
                play_audio_msg.write(PlayAudioMsg::new("button"));
            }
        },
        // Left-click selects the ability into a free slot, or errors if full.
        PointerButton::Primary => {
            if is_currently_equipped {
                if let Some(pos) =
                    player.active_abilities.iter().position(|x| x.as_ref() == Some(ability_name))
                {
                    player.active_abilities[pos] = None;
                    play_audio_msg.write(PlayAudioMsg::new("button"));
                }
                return;
            }
            if let Some(pos) = player.active_abilities.iter().position(|x| x.is_none()) {
                player.active_abilities[pos] = Some(ability_name.clone());
                play_audio_msg.write(PlayAudioMsg::new("button"));
            } else {
                play_audio_msg.write(PlayAudioMsg::new("error"));
            }
        },
        _ => {},
    }
}

fn spawn_active_hotkey_slot(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    index: usize,
    key: Option<&str>,
    hotkey: &str,
) {
    let mut cmd = parent.spawn((
        Node {
            width: ACTIVE_HOTKEY_SLOT_SIZE,
            height: ACTIVE_HOTKEY_SLOT_SIZE,
            border: UiRect::all(Val::Px(2.)),
            position_type: PositionType::Relative,
            ..default()
        },
        BorderColor::all(BUTTON_BORDER_COLOR),
        BackgroundColor(if key.is_some() {
            SELECTED_COLOR
        } else {
            EMPTY_SLOT_COLOR
        }),
        Interaction::default(),
        Pickable::default(),
        bevy::ui::RelativeCursorPosition::default(),
        ActiveHotkeySlot {
            index,
        },
    ));

    let image_key = match key {
        Some(k) => format!("build_{k}"),
        None => "stone".to_string(),
    };
    let image_color = match key {
        Some(_) => Color::WHITE,
        None => Color::NONE,
    };
    cmd.insert(ImageNode {
        image: assets.image(&image_key),
        color: image_color,
        ..default()
    });

    cmd.observe(handle_hotkey_slot_hover)
        .observe(handle_hotkey_slot_hover_out)
        .observe(handle_hotkey_drag_start)
        .observe(handle_hotkey_drag)
        .observe(handle_hotkey_drag_end)
        .observe(handle_hotkey_drop)
        .observe(handle_active_hotkey_slot_click)
        .with_children(|parent| {
            // Hotkey letter in the bottom-right corner.
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
                        add_text(hotkey, "bold", 1.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                        Pickable::IGNORE,
                    ));
                });
        });
}

pub fn handle_hotkey_slot_hover(
    event: On<Pointer<Over>>,
    mut commands: Commands,
    player: Res<Player>,
    slot_q: Query<&ActiveHotkeySlot>,
    ghost_q: Query<Entity, With<PrecombatDragGhost>>,
    window_e: Single<Entity, With<Window>>,
) {
    if !ghost_q.is_empty() {
        return;
    }
    let Ok(slot) = slot_q.get(event.entity) else {
        return;
    };
    let is_filled = player.active_abilities.get(slot.index).and_then(|opt| opt.as_ref()).is_some();
    if is_filled {
        commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Pointer));
    } else {
        commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
    }
}

pub fn handle_hotkey_slot_hover_out(
    _event: On<Pointer<Out>>,
    mut commands: Commands,
    ghost_q: Query<Entity, With<PrecombatDragGhost>>,
    window_e: Single<Entity, With<Window>>,
) {
    if !ghost_q.is_empty() {
        return;
    }
    commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
}

pub fn update_active_hotkey_slots(
    mut player: ResMut<Player>,
    assets: Res<WorldAssets>,
    dragging_q: Query<&ActiveHotkeySlot, With<DraggingSlot>>,
    mut slot_q: Query<(&ActiveHotkeySlot, &mut ImageNode, &mut BackgroundColor, Entity)>,
    mut known_abilities: Local<Vec<String>>,
) {
    // Check if player gained any new abilities
    let player_abilities = player.abilities.clone();
    for ability in &player_abilities {
        if !known_abilities.contains(ability) {
            let is_equipped = player.active_abilities.contains(&Some(ability.clone()));
            if !is_equipped {
                if let Some(pos) = player.active_abilities.iter().position(|x| x.is_none()) {
                    player.active_abilities[pos] = Some(ability.clone());
                }
            }
        }
    }
    *known_abilities = player_abilities;

    let dragged_slot = dragging_q.iter().next();

    for (slot, mut image_node, mut bg_color, _entity) in &mut slot_q {
        let is_dragged = dragged_slot.map(|d| d.index == slot.index).unwrap_or(false);

        let key = if is_dragged {
            None
        } else {
            player.active_abilities.get(slot.index).and_then(|opt| opt.as_deref())
        };

        if let Some(key) = key {
            image_node.image = assets.image(format!("build_{key}"));
            image_node.color = Color::WHITE;
            *bg_color = BackgroundColor(SELECTED_COLOR);
        } else {
            image_node.image = assets.image("stone");
            image_node.color = Color::NONE;
            *bg_color = BackgroundColor(EMPTY_SLOT_COLOR);
        }
    }
}

fn clear_drag_ghost(commands: &mut Commands, ghost_q: &Query<Entity, With<PrecombatDragGhost>>) {
    for entity in ghost_q {
        commands.entity(entity).try_despawn();
    }
}

pub fn handle_hotkey_drag_start(
    event: On<Pointer<DragStart>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    player: Res<Player>,
    slot_q: Query<&ActiveHotkeySlot>,
    ghost_q: Query<Entity, With<PrecombatDragGhost>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    window_e: Single<Entity, With<Window>>,
) {
    let Ok(slot) = slot_q.get(event.entity) else {
        return;
    };
    if event.button != PointerButton::Primary {
        return;
    }
    let key = player.active_abilities.get(slot.index).and_then(|opt| opt.as_deref());
    let Some(key) = key else {
        return;
    };

    clear_drag_ghost(&mut commands, &ghost_q);
    clear_tooltips(&mut commands, &tooltip_q);
    let pos = event.pointer_location.position;
    let left = pos.x - 36.0;
    let top = pos.y - 36.0;
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(top),
            width: ACTIVE_HOTKEY_SLOT_SIZE,
            height: ACTIVE_HOTKEY_SLOT_SIZE,
            border: UiRect::all(Val::Px(2.)),
            ..default()
        },
        BorderColor::all(BUTTON_BORDER_COLOR),
        BackgroundColor(Color::srgba(1., 1., 1., 0.25)),
        ImageNode::new(assets.image(format!("build_{key}"))).with_mode(NodeImageMode::Stretch),
        GlobalZIndex(1200),
        Pickable::IGNORE,
        PrecombatDragGhost,
    ));
    commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Move));

    commands.entity(event.entity).insert(DraggingSlot);
}

pub fn handle_hotkey_drag(
    event: On<Pointer<Drag>>,
    mut commands: Commands,
    window_e: Single<Entity, With<Window>>,
    mut ghost_node_q: Query<&mut Node, With<PrecombatDragGhost>>,
) {
    let pos = event.pointer_location.position;
    for mut node in ghost_node_q.iter_mut() {
        node.left = Val::Px(pos.x - 36.0);
        node.top = Val::Px(pos.y - 36.0);
    }
    commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Move));
}

pub fn handle_hotkey_drag_end(
    _event: On<Pointer<DragEnd>>,
    mut commands: Commands,
    player: Res<Player>,
    slot_state_q: Query<(&bevy::ui::RelativeCursorPosition, &ActiveHotkeySlot)>,
    ghost_q: Query<Entity, With<PrecombatDragGhost>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    window_e: Single<Entity, With<Window>>,
    dragging_q: Query<Entity, With<DraggingSlot>>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    windows: Query<&Window>,
) {
    clear_drag_ghost(&mut commands, &ghost_q);

    for entity in &dragging_q {
        commands.entity(entity).remove::<DraggingSlot>();
    }

    refresh_hovered_hotkey_slot(
        &mut commands,
        &assets,
        &localization,
        &settings,
        &player,
        *window_e,
        &slot_state_q,
        &tooltip_q,
        &windows,
    );
}

pub fn handle_hotkey_drop(
    event: On<Pointer<DragDrop>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    slot_q: Query<&ActiveHotkeySlot>,
    slot_state_q: Query<(&bevy::ui::RelativeCursorPosition, &ActiveHotkeySlot)>,
    mut commands: Commands,
    ghost_q: Query<Entity, With<PrecombatDragGhost>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    window_e: Single<Entity, With<Window>>,
    dragging_q: Query<Entity, With<DraggingSlot>>,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    windows: Query<&Window>,
) {
    clear_drag_ghost(&mut commands, &ghost_q);

    for entity in &dragging_q {
        commands.entity(entity).remove::<DraggingSlot>();
    }

    if let (Ok(target), Ok(source)) = (slot_q.get(event.entity), slot_q.get(event.dropped)) {
        if target.index != source.index
            && source.index < player.active_abilities.len()
            && target.index < player.active_abilities.len()
        {
            player.active_abilities.swap(source.index, target.index);
            play_audio_msg.write(PlayAudioMsg::new("button"));
        }
    }

    refresh_hovered_hotkey_slot(
        &mut commands,
        &assets,
        &localization,
        &settings,
        &player,
        *window_e,
        &slot_state_q,
        &tooltip_q,
        &windows,
    );
}

pub fn handle_active_hotkey_slot_click(
    event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    slot_q: Query<&ActiveHotkeySlot>,
    mut commands: Commands,
    window_e: Single<Entity, With<Window>>,
) {
    if event.button != PointerButton::Secondary {
        return;
    }

    let Ok(slot) = slot_q.get(event.entity) else {
        return;
    };

    if slot.index < player.active_abilities.len() && player.active_abilities[slot.index].is_some() {
        player.active_abilities[slot.index] = None;
        play_audio_msg.write(PlayAudioMsg::new("button"));
        commands.entity(*window_e).insert(CursorIcon::from(SystemCursorIcon::Default));
    }
}

pub fn active_hotkey_slot_tooltip_system(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    localization: Res<Localization>,
    settings: Res<Settings>,
    player: Res<Player>,
    level_up: Res<LevelUpPending>,
    active_modal: Res<ActiveModal>,
    slot_q: Query<(&bevy::ui::RelativeCursorPosition, &ActiveHotkeySlot)>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
    ghost_q: Query<Entity, With<PrecombatDragGhost>>,
    mut last_shown: Local<Option<usize>>,
) {
    if active_modal.active {
        clear_tooltips(&mut commands, &tooltip_q);
        *last_shown = None;
        return;
    }

    if level_up.active {
        return;
    }

    if !ghost_q.is_empty() {
        clear_tooltips(&mut commands, &tooltip_q);
        *last_shown = None;
        return;
    }

    // Re-evaluate the hovered slot every frame (no `Changed` gate) so the tooltip
    // reliably reappears after a drag ends. Geometry-based `cursor_over` is used
    // instead of `Interaction` because the press/hover state stays locked to the
    // dragged slot right after a drop, whereas the relative cursor position always
    // reflects the slot actually under the pointer.
    let hovered_index =
        slot_q.iter().find_map(|(rel, slot)| rel.cursor_over().then_some(slot.index));

    match hovered_index {
        Some(idx) => {
            // Respawn if the hovered slot changed OR if another tooltip system
            // (e.g. right_column/equip) cleared our tooltip when one of its own
            // targets changed interaction during/after the drag.
            if *last_shown != Some(idx) || tooltip_q.is_empty() {
                spawn_active_hotkey_tooltip(
                    &mut commands,
                    &assets,
                    &localization,
                    &settings,
                    &player,
                    idx,
                    &tooltip_q,
                    &windows,
                );
            }
            *last_shown = Some(idx);
        },
        None => {
            // Only clear the tooltip we own, so we don't clobber other systems'.
            if last_shown.is_some() {
                clear_tooltips(&mut commands, &tooltip_q);
                *last_shown = None;
            }
        },
    }
}
