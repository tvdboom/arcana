use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use strum::IntoEnumIterator;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::classes::Class;
use crate::core::constants::*;
use crate::core::localization::{Localization, LocalizedText};
use crate::core::menu::utils::{add_root_node, add_text, recolor};
use crate::core::player::{Attribute, Player};
use crate::core::settings::{Language, Settings};
use crate::core::ui::creation::SelectionItem;
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::window::SystemCursorIcon;

const BAR_BG_COLOR: Color = Color::srgba_u8(0, 0, 0, 160);
const HEALTH_COLOR: Color = Color::srgb_u8(170, 35, 35);
const MANA_COLOR: Color = Color::srgb_u8(40, 80, 185);
const PLACEHOLDER_COLOR: Color = Color::srgba_u8(40, 40, 55, 220);

// Viewport-relative icon sizes (scale with window width)
const ICON_ITEM: Val = Val::Vw(3.2);    // equipment / ability / perk card icons
const ICON_ACTION: Val = Val::Vh(8.5);  // action button circles
const ICON_BADGE: Val = Val::Vw(1.9);   // equipped badge overlay
const ICON_STAT: Val = Val::Vw(2.4);    // gold / AP stat icons

#[derive(Component)]
pub struct PlayingCmp;

#[derive(Component)]
pub struct ActionButton(pub &'static str);

/// Simple text stats that are refreshed every frame.
#[derive(Clone, Copy, PartialEq, Eq)]
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
    Armor,
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
pub struct GoldToast {
    pub timer: f32,
}

#[derive(Component)]
pub struct ToastContainer;

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

#[derive(Component)] pub struct EquipmentListWrapper;
#[derive(Component)] pub struct AbilitiesListWrapper;
#[derive(Component)] pub struct PerksListWrapper;

/// The equipment image-slots overlaid on the character portrait.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Helmet,
    Accessory,
    Accessory2,
    WeaponLH,
    WeaponRH,
    Armor,
    Boots,
    Gloves,
}

#[derive(Component)]
pub struct PetImage;

#[derive(Component)]
pub struct TooltipNode {
    width: f32,
    height: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InfoTooltip {
    Gold,
    ActionPoints,
    Combat(PlayingStat),
    Action(&'static str),
    Pet,
}

#[derive(Resource, Default)]
pub struct LevelUpPending {
    pub active: bool,
    pub new_level: u8,
    pub points_remaining: u8,
    pub attr_gains: [i8; 6],
    pub ability_choices: Vec<String>,
    pub perk_choices: Vec<String>,
    pub ability_chosen: Option<usize>,
    pub perk_chosen: Option<usize>,
}

#[derive(Component)] pub struct LevelUpOverlayCmp;
#[derive(Component)] pub struct LevelUpAttrPlusBtn(pub Attribute);
#[derive(Component)] pub struct LevelUpAttrMinusBtn(pub Attribute);
#[allow(unused)]
#[derive(Component)] pub struct LevelUpAttrPointsDisplay;
#[derive(Component)] pub struct LevelUpAbilityChoiceBtn(pub usize);
#[derive(Component)] pub struct LevelUpPerkChoiceBtn(pub usize);
#[allow(unused)]
#[derive(Component)] pub struct LevelUpConfirmBtn;

fn portrait_key(player: &Player) -> String {
    match player.class {
        Class::Mage(ajah) => ajah.get_image_key(player),
        _ => player.class.get_image_key(player),
    }
}

fn class_line(player: &Player, localization: &Localization, lang: Language) -> String {
    format!("{} {}", localization.get("level", lang), player.level)
}

fn playing_title(player: &Player) -> String {
    if player.pet.is_some() && !player.pet_name.trim().is_empty() {
        format!("{} & {}", player.name, player.pet_name)
    } else {
        player.name.clone()
    }
}

fn localized_class_name(player: &Player, localization: &Localization, lang: Language) -> String {
    match player.class {
        Class::Mage(ajah) => format!(
            "{} {}",
            localization.get(&ajah.to_lowername(), lang),
            localization.get("mage", lang)
        ),
        _ => localization.get(&player.class.to_lowername(), lang),
    }
}

pub fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn translate_game_term(name: &str, localization: &Localization, lang: Language) -> String {
    let name_lower = name.to_lowercase();
    if let Some(direct) = localization.get_opt(&name_lower, lang) {
        return direct;
    }
    if lang == Language::English {
        return capitalize_words(&name_lower);
    }
    
    let words: Vec<String> = name_lower.split_whitespace().map(|w| w.to_string()).collect();
    let mut trans_words = Vec::new();
    for word in words {
        let trans = match lang {
            Language::Spanish => match word.as_str() {
                "novice" => "novato",
                "apprentice" => "aprendiz",
                "adept" => "adepto",
                "expert" => "experto",
                "master" => "maestro",
                "bronze" => "bronce",
                "iron" => "hierro",
                "steel" => "acero",
                "mithril" => "mitril",
                "adamant" => "adamantio",
                "rawhide" => "cuero crudo",
                "leather" => "cuero",
                "cloth" => "tela",
                "quilted" => "acolchado",
                "enchanted" => "encantado",
                "copper" => "cobre",
                "pewter" => "peltre",
                "silver" => "plata",
                "gold" => "oro",
                "platinum" => "platino",
                "minor" => "menor",
                "standard" => "estándar",
                "major" => "mayor",
                "helm" | "helmet" => "casco",
                "tunic" => "túnica",
                "hauberk" => "cota",
                "robes" => "túnicas",
                "garb" => "manto",
                "boots" | "treads" | "greaves" | "soles" | "shoes" => "botas",
                "axe" => "hacha",
                "bow" => "arco",
                "spear" => "lanza",
                "dagger" => "daga",
                "staff" => "bastón",
                "wand" => "varita",
                "tome" => "tomo",
                "shield" => "escudo",
                "handwraps" | "gauntlets" | "gloves" | "vambraces" => "guantes",
                "necklace" => "collar",
                "potion" => "poción",
                "elixir" => "elixir",
                "vial" => "vial",
                "blend" => "mezcla",
                "vanguard" => "vanguardia",
                "pyromancer" => "piromante",
                "silent" => "silencioso",
                "primal" => "primigenio",
                "mighty" => "poderoso",
                "acolyte" => "acólito",
                "assassin" => "asesino",
                "wildheart" => "corazón salvaje",
                "guardian" => "guardián",
                "sorcerer" => "hechicero",
                "infiltrator" => "infiltrador",
                "verdant" => "verdoso",
                "deception" => "engaño",
                "valor" => "valor",
                "focus" => "enfoque",
                "earth" => "tierra",
                "constitution" => "constitución",
                "health" => "salud",
                "mana" => "maná",
                "rejuvenation" => "rejuvenecimiento",
                _ => word.as_str(),
            },
            Language::Dutch => match word.as_str() {
                "novice" => "novice",
                "apprentice" => "leerling",
                "adept" => "adept",
                "expert" => "expert",
                "master" => "meester",
                "bronze" => "brons",
                "iron" => "ijzer",
                "steel" => "staal",
                "mithril" => "mithril",
                "adamant" => "adamant",
                "rawhide" => "ruw leer",
                "leather" => "leer",
                "cloth" => "stof",
                "quilted" => "gewatteerd",
                "enchanted" => "betoverd",
                "copper" => "koper",
                "pewter" => "tin",
                "silver" => "zilver",
                "gold" => "goud",
                "platinum" => "platina",
                "minor" => "kleine",
                "standard" => "standaard",
                "major" => "grote",
                "helm" | "helmet" => "helm",
                "tunic" => "tuniek",
                "hauberk" => "bolder",
                "robes" => "gewaad",
                "garb" => "dracht",
                "boots" | "treads" | "greaves" | "soles" | "shoes" => "laarzen",
                "axe" => "bijl",
                "bow" => "boog",
                "spear" => "speer",
                "dagger" => "dolk",
                "staff" => "staf",
                "wand" => "stafje",
                "tome" => "boek",
                "shield" => "schild",
                "handwraps" | "gauntlets" | "gloves" | "vambraces" => "handschoenen",
                "necklace" => "halsketting",
                "potion" => "drank",
                "elixir" => "elixer",
                "vial" => "flacon",
                "blend" => "mengsel",
                "vanguard" => "voorhoede",
                "pyromancer" => "vuurmagiër",
                "silent" => "stille",
                "primal" => "oerkracht",
                "mighty" => "machtige",
                "acolyte" => "acoliet",
                "assassin" => "sluipmoordenaar",
                "wildheart" => "wildhart",
                "guardian" => "beschermer",
                "sorcerer" => "tovenaar",
                "infiltrator" => "infiltrant",
                "verdant" => "groene",
                "deception" => "misleiding",
                "valor" => "moed",
                "focus" => "focus",
                "earth" => "aarde",
                "constitution" => "constitutie",
                "health" => "gezondheid",
                "mana" => "mana",
                "rejuvenation" => "verjonging",
                _ => word.as_str(),
            },
            _ => word.as_str(),
        };
        trans_words.push(trans.to_string());
    }
    
    capitalize_words(&trans_words.join(" "))
}

fn name_with_level(name: String, level: u8, localization: &Localization, lang: Language) -> String {
    let trans = translate_game_term(&name, localization, lang);
    format!("{} (Lv. {})", trans, level)
}

fn ability_detail_line(
    stats: &crate::core::catalog::GeneratedAbility,
    localization: &Localization,
    lang: Language,
) -> String {
    let mut parts = vec![format!(
        "{}: {} | {}: {}",
        localization.get("ability_type", lang),
        localization.get(&stats.magic_type.to_lowercase(), lang),
        localization.get("mana", lang),
        stats.mana_cost
    )];

    if stats.cooldown > 0 {
        parts.push(format!(
            "{}: {}s",
            localization.get("cooldown", lang),
            stats.cooldown
        ));
    }

    parts.join(" | ")
}

/// Format the bonus characteristics of a weapon, e.g. "+6 attack | +10 crit | 1.2 as".
fn weapon_stat_lines(
    weapon: &crate::core::catalog::GeneratedEquipment,
    player: &Player,
    localization: &Localization,
    lang: Language,
) -> Vec<String> {
    let mut parts = Vec::new();
    let mut push = |val: i32, key: &str| {
        if val != 0 {
            let sign = if val > 0 {
                "+"
            } else {
                ""
            };
            parts.push(format!("{}{} {}", sign, val, localization.get(key, lang)));
        }
    };
    push(weapon.attack, "attack");
    push(weapon.armor, "armor");
    push(weapon.crit, "crit");
    push(weapon.initiative, "initiative");
    if weapon.attack_speed > 0.0 {
        parts.push(format!("{:.1} as", player.weapon_attack_speed(&weapon.name)));
    }
    if parts.is_empty() {
        vec![]
    } else {
        vec![parts.join(" | ")]
    }
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
    value_for: impl Fn(&crate::core::catalog::GeneratedEquipment) -> i32,
) -> Vec<String> {
    player
        .equipped_equipment()
        .into_iter()
        .filter_map(|weapon| {
            let value = value_for(&weapon);
            (value != 0).then(|| signed_line(capitalize_words(&weapon.name.to_string()), value))
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
            let mut lines = vec![signed_line(localization.get("base", lang), 5)];
            lines.push(signed_line(
                localization.get("strength", lang),
                player.strength() as i32 - 10,
            ));
            lines.extend(weapon_bonus_lines(player, |weapon| {
                weapon.attack
            }));
            lines
        },
        PlayingStat::Armor => {
            let mut lines = vec![signed_line(
                localization.get("constitution", lang),
                player.constitution() as i32 / 4,
            )];
            lines.extend(weapon_bonus_lines(player, |weapon| {
                weapon.armor
            }));
            lines
        },
        PlayingStat::Initiative => {
            let mut lines = vec![signed_line(
                localization.get("dexterity", lang),
                player.dexterity() as i32 / 2,
            )];
            lines.extend(weapon_bonus_lines(player, |weapon| {
                weapon.initiative
            }));
            if matches!(player.class, Class::Rogue) {
                lines.push(signed_line(localization.get("rogue", lang), 2));
            }
            lines
        },
        _ => vec![],
    }
}

fn spawn_pet_stat_box(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    label_key: &str,
    image_key: &str,
    value: i32,
) {
    parent
        .spawn((
            Node {
                width: percent(32.),
                aspect_ratio: Some(1.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(1.),
                border: UiRect::all(Val::Px(2.)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(PLACEHOLDER_COLOR),
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
                ImageNode {
                    image: assets.image(image_key),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(1., 1., 1., 0.3),
                    ..default()
                },
            ));
            parent.spawn((
                add_text(localization.get(label_key, lang), "medium", 1.6, assets),
                TextColor(BUTTON_TEXT_COLOR),
            ));
            parent.spawn((
                add_text(value.to_string(), "bold", 3.0, assets),
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_pet_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    player: &Player,
    windows: &Query<&Window>,
) {
    let Some(pet) = player.pet else { return };
    let pet_type_name = capitalize_words(&pet.to_lowername());
    let title = format!("{} ({})", player.pet_name, pet_type_name);
    let desc = localization
        .get_opt(&format!("{}_desc", pet.to_lowername()), lang)
        .unwrap_or_else(|| format!("A loyal {} companion.", pet_type_name.to_lowercase()));
    let stats = pet.stats();

    let tooltip_width = 320.0_f32;
    let tooltip_height = 180.0_f32;

    let (left, top) = if let Ok(window) = windows.single() {
        if let Some(cursor) = window.cursor_position() {
            place_tooltip(cursor, tooltip_width, tooltip_height, window.width(), window.height())
        } else {
            (20., 20.)
        }
    } else {
        (20., 20.)
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::all(Val::Px(10.)),
                border: UiRect::all(Val::Px(2.)),
                width: Val::Px(tooltip_width),
                height: Val::Auto,
                max_width: Val::Px(tooltip_width),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(10, 18, 45, 245)),
            BorderColor::all(BUTTON_BORDER_COLOR),
            GlobalZIndex(200),
            TooltipNode {
                width: tooltip_width,
                height: tooltip_height,
            },
        ))
        .with_children(|parent| {
            parent.spawn((add_text(title, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));
            parent.spawn((
                add_text(desc, "medium", 1.6, assets),
                TextColor(Color::WHITE),
            ));
            // Stat boxes row
            parent
                .spawn(Node {
                    width: percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::top(Val::Px(4.)),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_pet_stat_box(parent, assets, localization, lang, "attack", "attack_icon", stats.attack);
                    spawn_pet_stat_box(parent, assets, localization, lang, "armor", "armor_icon", stats.armor);
                    spawn_pet_stat_box(parent, assets, localization, lang, "initiative", "initiative_icon", stats.initiative);
                });
        });
}

fn spawn_action_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    action_name: String,
    ap_cost: u32,
    desc: String,
    windows: &Query<&Window>,
) {
    let wrapped: Vec<String> = wrap_tooltip_line(&desc, 60);
    let desc_max = wrapped.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    // Title width: action name + " (  N)" — approximate as name len + 6 chars
    let title_chars = (action_name.chars().count() + 8) as f32;
    let max_chars = title_chars.max(desc_max as f32);

    let (window_width, window_height, cursor) = if let Ok(window) = windows.single() {
        (window.width(), window.height(), window.cursor_position())
    } else {
        (1600., 900., None)
    };
    let tooltip_width = (max_chars * 9.5 + 32.).clamp(200., (window_width - 24.).max(200.));
    let line_count = 1 + wrapped.len().max(1);
    let tooltip_height = (line_count as f32 * 24. + 36.).clamp(64., (window_height - 24.).max(64.));
    let (left, top) = place_tooltip(
        cursor.unwrap_or(Vec2::new(100., 100.)),
        tooltip_width,
        tooltip_height,
        window_width,
        window_height,
    );

    let icon_size = Val::Px(16.);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::all(Val::Px(10.)),
                border: UiRect::all(Val::Px(2.)),
                width: Val::Px(tooltip_width),
                height: Val::Auto,
                max_width: Val::Px(tooltip_width),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(10, 18, 45, 245)),
            BorderColor::all(BUTTON_BORDER_COLOR),
            GlobalZIndex(200),
            TooltipNode {
                width: tooltip_width,
                height: tooltip_height,
            },
        ))
        .with_children(|parent| {
            // Title row: "ActionName  ([ap icon] N)"
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        add_text(action_name, "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                    parent.spawn((
                        add_text("  ".to_string(), "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                    parent.spawn((
                        Node { width: icon_size, height: icon_size, ..default() },
                        ImageNode::new(assets.image("ap")).with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(format!(" {}", ap_cost), "bold", 1.9, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                });
            if !wrapped.is_empty() {
                parent.spawn((
                    add_text(wrapped.join("\n"), "medium", 1.6, assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
}

fn spawn_tooltip(
    commands: &mut Commands,
    assets: &WorldAssets,
    title: String,
    lines: Vec<String>,
    windows: &Query<&Window>,
) {
    let wrapped_lines: Vec<String> =
        lines.into_iter().flat_map(|line| wrap_tooltip_line(&line, 60)).collect();
    let max_chars = std::iter::once(title.as_str())
        .chain(wrapped_lines.iter().map(String::as_str))
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(12) as f32;
    let line_count = 1 + wrapped_lines.len().max(1);

    let (window_width, window_height, cursor) = if let Ok(window) = windows.single() {
        (window.width(), window.height(), window.cursor_position())
    } else {
        (1600., 900., None)
    };
    let tooltip_width = (max_chars * 9.5 + 32.).clamp(190., (window_width - 24.).max(190.));
    let tooltip_height = (line_count as f32 * 24. + 24.).clamp(64., (window_height - 24.).max(64.));
    let (left, top) = place_tooltip(
        cursor.unwrap_or(Vec2::new(100., 100.)),
        tooltip_width,
        tooltip_height,
        window_width,
        window_height,
    );

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::all(Val::Px(10.)),
                border: UiRect::all(Val::Px(2.)),
                width: Val::Px(tooltip_width),
                height: Val::Auto,
                max_width: Val::Px(tooltip_width),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(10, 18, 45, 245)),
            BorderColor::all(BUTTON_BORDER_COLOR),
            GlobalZIndex(200),
            TooltipNode {
                width: tooltip_width,
                height: tooltip_height,
            },
        ))
        .with_children(|parent| {
            parent.spawn((add_text(title, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));
            if !wrapped_lines.is_empty() {
                parent.spawn((
                    add_text(wrapped_lines.join("\n"), "medium", 1.6, assets),
                    TextColor(Color::WHITE),
                ));
            }
        });
}

fn wrap_tooltip_line(line: &str, max_chars: usize) -> Vec<String> {
    if line.chars().count() <= max_chars {
        return vec![line.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split_whitespace() {
        let next_len = current.chars().count()
            + if current.is_empty() {
                0
            } else {
                1
            }
            + word.chars().count();
        if next_len > max_chars && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn place_tooltip(
    cursor: Vec2,
    width: f32,
    height: f32,
    window_width: f32,
    window_height: f32,
) -> (f32, f32) {
    let margin = 12.;
    let mut left = cursor.x + margin;
    if left + width + margin > window_width {
        left = cursor.x - width - margin;
    }
    let mut top = cursor.y + margin;
    if top + height + margin > window_height {
        top = cursor.y - height - margin;
    }
    (
        left.clamp(margin, (window_width - width - margin).max(margin)),
        top.clamp(margin, (window_height - height - margin).max(margin)),
    )
}

/// A bordered placeholder box (used wherever an item/ability image will go later).
fn spawn_placeholder(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, image_key: &str, size: Val) {
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
        ImageNode::new(assets.image(image_key)).with_mode(NodeImageMode::Stretch),
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
) {
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
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
        ))
        .with_children(|parent| {
            spawn_placeholder(parent, assets, image_key, ICON_ITEM);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    let mut name_cmd = parent
                        .spawn((add_text(name, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));
                    if let Some(key) = name_key {
                        name_cmd.insert(LocalizedText(key));
                    }

                    for line in lines {
                        parent.spawn((
                            add_text(line, "medium", 1.6, assets),
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
                    image: assets.image(image_key),
                    image_mode: NodeImageMode::Stretch,
                    color: Color::srgba(1., 1., 1., 0.3),
                    ..default()
                },
            ));
            parent.spawn((
                add_text(localization.get(label_key, lang), "medium", 2.2, assets),
                TextColor(BUTTON_TEXT_COLOR),
                LocalizedText(label_key.to_string()),
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
    mut player: ResMut<Player>,
    existing_screen_q: Query<Entity, With<PlayingCmp>>,
) {
    if existing_screen_q.iter().next().is_some() {
        return;
    }

    if player.pet.is_some() && player.pet_health.is_none() {
        player.pet_health = Some(player.pet.unwrap().stats().health as f32);
    }

    let lang = settings.language;

    let (mut root_node, pickable) = add_root_node(true);
    root_node.justify_content = JustifyContent::FlexStart;
    root_node.padding = UiRect::all(Val::Px(0.));
    root_node.row_gap = Val::Px(4.);

    commands
        .spawn((
            root_node,
            pickable,
            ImageNode {
                image: assets.image("bg3"),
                image_mode: NodeImageMode::Stretch,
                color: Color::srgba(0.40, 0.40, 0.40, 1.0),
                ..default()
            },
            PlayingCmp,
        ))
        .with_children(|parent| {
            // Character name, top centered with banner background.
            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::Center,
                        width: Val::Vh(50.0),
                        height: Val::Vh(7.11),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect {
                            top: Val::Vh(2.67),
                            bottom: Val::Vh(1.78),
                            ..default()
                        },
                        ..default()
                    },
                    ImageNode::new(assets.image("banner")).with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
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
                    spawn_playing_action_button(parent, "rest", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "study", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "work", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "craft", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "shop", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "train", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "hunt", &assets, &localization, lang);
                    spawn_playing_action_button(parent, "quest", &assets, &localization, lang);
                });

            // Toast container: stacks notifications top-right
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
        });
}

/// Column 1: Character portrait image with equipment slot overlays and pet.
fn spawn_image_column(parent: &mut ChildSpawnerCommands, assets: &WorldAssets, player: &Player) {
    parent
        .spawn((
            Node {
                width: percent(33.5),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(6.)),
                ..default()
            },
        ))
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
                                parent.spawn((
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
                                EquipSlot::Armor,
                                EquipSlot::WeaponLH,
                                EquipSlot::WeaponRH,
                                EquipSlot::Gloves,
                                EquipSlot::Boots,
                            ] {
                                parent.spawn((
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
                        let pet = player.pet.unwrap();
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
                                ImageNode::new(assets.image(pet.to_lowername()))
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
                                    localization.get("characteristics", lang),
                                    "bold",
                                    2.2,
                                    assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("characteristics".to_string()),
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
                                        ("race", PlayingStat::CharRace),
                                        ("class", PlayingStat::CharClass),
                                        ("sex", PlayingStat::CharSex),
                                        ("age", PlayingStat::CharAge),
                                        ("height", PlayingStat::CharHeight),
                                        ("weight", PlayingStat::CharWeight),
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
                                add_text(localization.get("attributes", lang), "bold", 2.2, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                                LocalizedText("attributes".to_string()),
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
                                                        localization
                                                            .get(&attr.to_lowername(), lang),
                                                        "medium",
                                                        1.8,
                                                        assets,
                                                    ),
                                                    TextColor(Color::WHITE),
                                                    LocalizedText(attr.to_lowername()),
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
                        "attack",
                        "attack_icon",
                        PlayingStat::Attack,
                    );
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        "armor",
                        "armor_icon",
                        PlayingStat::Armor,
                    );
                    spawn_combat_stat(
                        parent,
                        assets,
                        localization,
                        lang,
                        "initiative",
                        "initiative_icon",
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
    slot_q: Query<(&Interaction, &EquipSlot), Changed<Interaction>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if level_up.active {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    let lang = settings.language;

    for (interaction, slot) in &slot_q {
        // Despawn any existing tooltip on any change
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }

        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let equipped_key = match slot {
                EquipSlot::Helmet => player.helmet.as_deref(),
                EquipSlot::Accessory => player.accessory.as_deref(),
                EquipSlot::Accessory2 => player.accessory2.as_deref(),
                EquipSlot::WeaponLH => player.weapon_lh.as_deref().or(player.weapon_2h.as_deref()),
                EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
                EquipSlot::Armor => player.armor.as_deref(),
                EquipSlot::Boots => player.boots.as_deref(),
                EquipSlot::Gloves => player.gloves.as_deref(),
            };

            if let Some(key) = equipped_key {
                if let Some(weapon) = crate::core::catalog::get_equipment(key) {
                    let name = name_with_level(weapon.name.to_string(), weapon.level, &localization, lang);
                    let stat_lines = weapon_stat_lines(&weapon, &player, &localization, lang);

                    spawn_tooltip(&mut commands, &assets, name, stat_lines, &windows);
                }
            }
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
    info_q: Query<(&Interaction, &InfoTooltip), Changed<Interaction>>,
    tooltip_q: Query<Entity, With<TooltipNode>>,
    windows: Query<&Window>,
) {
    if level_up.active {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    let lang = settings.language;

    for (interaction, tooltip) in &info_q {
        for entity in tooltip_q.iter() {
            commands.entity(entity).try_despawn();
        }

        if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
            continue;
        }

        if matches!(tooltip, InfoTooltip::Pet) {
            spawn_pet_tooltip(&mut commands, &assets, &localization, lang, &player, &windows);
            continue;
        }

        if let InfoTooltip::Action(act) = tooltip {
            let ap_cost = match *act {
                "rest" | "study" | "work" => 1u32,
                "shop" => 0,
                "craft" | "train" | "hunt" => 2,
                "quest" => 3,
                _ => 1,
            };
            let action_name = localization.get(act, lang);
            let desc_key = format!("{}_desc", act);
            let desc = match *act {
                "work" => {
                    let charisma = player.charisma() as i32;
                    let level = player.level as i32;
                    let base = charisma * level;
                    let min_gold = (base * 4 / 5).max(1);
                    let max_gold = base * 6 / 5;
                    let base_text = localization.get_opt("work_desc_base", lang)
                        .unwrap_or_else(|| "Earn gold by working. Scales with Level and Charisma.".to_string());
                    let format_str = localization.get_opt("work_desc_format", lang)
                        .unwrap_or_else(|| "{} Earns: {min}-{max} Gold.".to_string());
                    format_str.replace("{}", &base_text)
                        .replace("{min}", &min_gold.to_string())
                        .replace("{max}", &max_gold.to_string())
                },
                "rest" => {
                    let wisdom = player.wisdom() as i32;
                    let level = player.level as i32;
                    let base = wisdom * level;
                    let min_r = (base * 4 / 5).max(1);
                    let max_r = base * 6 / 5;
                    let wisdom_bonus = (player.wisdom() as f32 - 10.).max(0.) * 0.005;
                    let max_pct = ((0.05 + wisdom_bonus).min(0.20) * 100.) as u32;
                    let base_text = localization.get_opt("rest_desc_base", lang)
                        .unwrap_or_else(|| "Rest to recover health and mana. Scales with Level and Wisdom.".to_string());
                    let format_str = localization.get_opt("rest_desc_format", lang)
                        .unwrap_or_else(|| "{} Recovers: {min}-{max} HP/MP. {pct}% chance of +max HP/MP.".to_string());
                    format_str.replace("{}", &base_text)
                        .replace("{min}", &min_r.to_string())
                        .replace("{max}", &max_r.to_string())
                        .replace("{pct}", &max_pct.to_string())
                },
                "study" => {
                    let int_bonus = (player.intelligence() as f32 - 10.).max(0.) * 0.025;
                    let perk_pct = ((0.333 + int_bonus).min(0.65) * 100.) as u32;
                    let abil_pct = ((0.200 + int_bonus).min(0.45) * 100.) as u32;
                    let base_text = localization.get_opt("study_desc_base", lang)
                        .unwrap_or_else(|| "Study tomes and scrolls. Scales with Intelligence.".to_string());
                    let format_str = localization.get_opt("study_desc_format", lang)
                        .unwrap_or_else(|| "{} Perk: {perk}%, Ability: {abil}%, +1 Wisdom: 5%.".to_string());
                    format_str.replace("{}", &base_text)
                        .replace("{perk}", &perk_pct.to_string())
                        .replace("{abil}", &abil_pct.to_string())
                },
                _ => localization.get_opt(&desc_key, lang)
                    .unwrap_or_else(|| match *act {
                        "hunt" => "Go hunting in the wild to earn gold.".to_string(),
                        "shop" => "Buy a random consumable item.".to_string(),
                        "quest" => "Embark on an adventure to earn gold and find new equipment.".to_string(),
                        "train" => "Train hard to increase a random attribute.".to_string(),
                        "craft" => "Craft a piece of equipment suitable for your level.".to_string(),
                        _ => "Perform an action.".to_string(),
                    }),
            };
            spawn_action_tooltip(&mut commands, &assets, action_name, ap_cost, desc, &windows);
            continue;
        }

        let (title, lines) = match tooltip {
            InfoTooltip::Gold => {
                (localization.get("gold", lang), vec![localization.get("gold_desc", lang)])
            },
            InfoTooltip::ActionPoints => {
                (localization.get("active_points", lang), vec![localization.get("active_points_desc", lang)])
            },
            InfoTooltip::Combat(stat) => {
                let title_key = match stat {
                    PlayingStat::Attack => "attack",
                    PlayingStat::Armor => "armor",
                    PlayingStat::Initiative => "initiative",
                    _ => "",
                };
                (
                    localization.get(title_key, lang),
                    combat_breakdown(*stat, &player, &localization, lang),
                )
            },
            InfoTooltip::Action(_) | InfoTooltip::Pet => unreachable!(),
        };

        spawn_tooltip(&mut commands, &assets, title, lines, &windows);
    }
}

/// Moves the tooltip to follow the mouse cursor.
pub fn tooltip_follow_cursor_system(
    mut tooltip_q: Query<(&mut Node, &TooltipNode)>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        if let Some(cursor) = window.cursor_position() {
            for (mut node, tooltip) in &mut tooltip_q {
                let (left, top) = place_tooltip(
                    cursor,
                    tooltip.width,
                    tooltip.height,
                    window.width(),
                    window.height(),
                );
                node.left = Val::Px(left);
                node.top = Val::Px(top);
            }
        }
    }
}

fn handle_tab_click(
    ev: On<Pointer<Click>>,
    btn_q: Query<&RightTabBtn>,
    mut right_tab: ResMut<RightTab>,
) {
    if let Ok(btn) = btn_q.get(ev.entity) {
        *right_tab = btn.0;
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
            },
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
                            for (tab, key, fallback) in [
                                (RightTab::Equipment, "equipment_tab", "Equipment"),
                                (RightTab::Abilities, "abilities_tab", "Abilities"),
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
                                        Pickable::default(),
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
    pub equip_wrap_q: Query<'w, 's, (&'static mut Node, &'static mut Visibility), (With<EquipmentListWrapper>, Without<AbilitiesListWrapper>, Without<PerksListWrapper>, Without<EquipSlot>, Without<RightTabBtn>)>,
    pub abil_wrap_q: Query<'w, 's, (&'static mut Node, &'static mut Visibility), (With<AbilitiesListWrapper>, Without<EquipmentListWrapper>, Without<PerksListWrapper>, Without<EquipSlot>, Without<RightTabBtn>)>,
    pub perk_wrap_q: Query<'w, 's, (&'static mut Node, &'static mut Visibility), (With<PerksListWrapper>, Without<EquipmentListWrapper>, Without<AbilitiesListWrapper>, Without<EquipSlot>, Without<RightTabBtn>)>,
    pub tab_btn_q: Query<'w, 's, (Entity, &'static RightTabBtn, &'static mut BackgroundColor, &'static mut BorderColor, &'static mut Node), (With<RightTabBtn>, Without<EquipmentListWrapper>, Without<AbilitiesListWrapper>, Without<PerksListWrapper>, Without<EquipSlot>)>,
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

    // Update the equipment image-slots on the portrait.
    for (slot, mut image) in &mut queries.slot_q {
        let equipped_key = match slot {
            EquipSlot::Helmet => player.helmet.as_deref(),
            EquipSlot::Accessory => player.accessory.as_deref(),
            EquipSlot::Accessory2 => player.accessory2.as_deref(),
            EquipSlot::WeaponLH => player.weapon_lh.as_deref().or(player.weapon_2h.as_deref()),
            EquipSlot::WeaponRH => player.weapon_rh.as_deref(),
            EquipSlot::Armor => player.armor.as_deref(),
            EquipSlot::Boots => player.boots.as_deref(),
            EquipSlot::Gloves => player.gloves.as_deref(),
        };
        image.image = match equipped_key {
            Some(key) => assets.image(key),
            None => assets.image("stone"),
        };
    }

    // Show only filled slots; hide WeaponRH when a 2H weapon is also equipped
    for (slot, mut vis) in &mut queries.slot_vis_q {
        let visible = match slot {
            EquipSlot::Helmet => player.helmet.is_some(),
            EquipSlot::Accessory => player.accessory.is_some(),
            EquipSlot::Accessory2 => player.accessory2.is_some(),
            EquipSlot::WeaponLH => player.weapon_lh.is_some() || player.weapon_2h.is_some(),
            EquipSlot::WeaponRH => player.weapon_rh.is_some() && player.weapon_2h.is_none(),
            EquipSlot::Armor => player.armor.is_some(),
            EquipSlot::Boots => player.boots.is_some(),
            EquipSlot::Gloves => player.gloves.is_some(),
        };
        *vis = if visible { Visibility::Inherited } else { Visibility::Hidden };
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
                (&player.weapon_2h, "two_hand_weapon"),
                (&player.armor, "armor"),
                (&player.boots, "boots"),
                (&player.gloves, "gloves"),
            ];

            // Collect equipped items and sort by level then name
            let mut equipped_items: Vec<crate::core::catalog::GeneratedEquipment> = equipped_slots
                .iter()
                .filter_map(|(slot_val, _)| slot_val.as_deref().and_then(crate::core::catalog::get_equipment))
                .collect();
            equipped_items.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for weapon in &equipped_items {
                empty = false;
                spawn_equipment_card(
                    parent,
                    &assets,
                    &weapon.name,
                    name_with_level(weapon.name.to_string(), weapon.level, &localization, lang),
                    weapon_stat_lines(&weapon, &player, &localization, lang),
                    EquipmentCard {
                        key: weapon.name.to_string(),
                        is_equipped: true,
                    },
                );
            }

            // Inventory items (unequipped), sorted by level then name
            let mut inventory_items: Vec<crate::core::catalog::GeneratedEquipment> = player
                .inventory
                .iter()
                .filter_map(|key| crate::core::catalog::get_equipment(key))
                .collect();
            inventory_items.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for weapon in &inventory_items {
                empty = false;
                spawn_equipment_card(
                    parent,
                    &assets,
                    &weapon.name,
                    name_with_level(weapon.name.to_string(), weapon.level, &localization, lang),
                    weapon_stat_lines(&weapon, &player, &localization, lang),
                    EquipmentCard {
                        key: weapon.name.to_string(),
                        is_equipped: false,
                    },
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
    if let Ok(entity) = queries.abil_q.single() {
        clear(&mut commands, entity, &queries.children_q);
        commands.entity(entity).with_children(|parent| {
            if player.abilities.is_empty() {
                parent.spawn((
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            let mut sorted_abilities: Vec<_> = player
                .abilities
                .iter()
                .filter_map(|key| crate::core::catalog::get_ability(key))
                .collect();
            sorted_abilities.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for ability in &sorted_abilities {
                let desc = localization
                    .get_opt(&format!("{}_desc", ability.name), lang)
                    .unwrap_or_else(|| {
                        format!(
                            "A powerful {} ability that consumes {} mana.",
                            ability.magic_type.to_lowercase(),
                            ability.mana_cost
                        )
                    });
                spawn_card(
                    parent,
                    &assets,
                    &ability.name,
                    name_with_level(ability.name.to_string(), ability.level, &localization, lang),
                    None,
                    vec![
                        ability_detail_line(&ability, &localization, lang),
                        desc,
                    ],
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
                    add_text(localization.get("none", lang), "medium", 1.6, &assets),
                    TextColor(Color::WHITE),
                ));
            }
            let mut sorted_perks: Vec<_> = player
                .perks
                .iter()
                .filter_map(|key| crate::core::catalog::get_perk(key))
                .collect();
            sorted_perks.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
            for perk in &sorted_perks {
                let desc = localization
                    .get_opt(&format!("{}_desc", perk.name), lang)
                    .unwrap_or_else(|| {
                        format!(
                            "An impressive passive perk that empowers your {} capabilities.",
                            perk.theme
                        )
                    });
                spawn_card(
                    parent,
                    &assets,
                    &perk.name,
                    name_with_level(perk.name.to_string(), perk.level, &localization, lang),
                    None,
                    vec![desc],
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
    mut hbar_q: Query<&mut Node, (With<HealthBarFill>, Without<ManaBarFill>, Without<PetHealthBarFill>)>,
    mut mbar_q: Query<&mut Node, (With<ManaBarFill>, Without<HealthBarFill>, Without<PetHealthBarFill>)>,
    mut pet_hbar_q: Query<&mut Node, (With<PetHealthBarFill>, Without<HealthBarFill>, Without<ManaBarFill>)>,
) {
    let lang = settings.language;

    for (mut text, stat) in &mut text_q {
        text.0 = match stat.0 {
            PlayingStat::ClassLine => class_line(&player, &localization, lang),
            PlayingStat::CharRace => localization.get(&player.race.to_lowername(), lang),
            PlayingStat::CharClass => localized_class_name(&player, &localization, lang),
            PlayingStat::CharSex => match player.sex {
                crate::core::player::Sex::Male => localization.get("male", lang),
                crate::core::player::Sex::Female => localization.get("female", lang),
            },
            PlayingStat::CharAge => {
                format!("{} {}", player.actual_age(), localization.get("years", lang))
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
            PlayingStat::Money => format!("{}", player.gold),
            PlayingStat::Attack => format!("{}", player.attack_damage()),
            PlayingStat::Armor => format!("{}", player.armor_value()),
            PlayingStat::Initiative => format!("{}", player.initiative()),
            PlayingStat::ActionPoints => format!("{}", player.ap),
            PlayingStat::PetHealth => {
                if let Some(pet) = player.pet {
                    let pet_max = pet.stats().health as f32;
                    let pet_current = player.pet_health.unwrap_or(pet_max);
                    format!(
                        "{} / {} {}",
                        pet_current.max(0.) as i32,
                        pet_max as i32,
                        localization.get("health", lang)
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
        let ratio = (player.health / player.max_health()).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Ok(mut node) = mbar_q.single_mut() {
        let ratio = (player.mana / player.max_mana()).clamp(0., 1.) * 100.;
        node.width = percent(ratio);
    }
    if let Some(pet) = player.pet {
        if let Ok(mut node) = pet_hbar_q.single_mut() {
            let pet_max = pet.stats().health as f32;
            let pet_current = player.pet_health.unwrap_or(pet_max);
            let ratio = (pet_current / pet_max).clamp(0., 1.) * 100.;
            node.width = percent(ratio);
        }
    }
}

pub fn highlight_border<E: std::fmt::Debug + Clone + Reflect>(
    color: Color,
    thickness: Val,
) -> impl Fn(On<Pointer<E>>, Query<(&mut BorderColor, &mut Node)>) {
    move |ev, mut q| {
        if let Ok((mut border_color, mut node)) = q.get_mut(ev.entity) {
            border_color.top = color;
            border_color.right = color;
            border_color.bottom = color;
            border_color.left = color;
            node.border = UiRect::all(thickness);
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
                    ImageNode::new(assets.image(format!("action_{}", action)))
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
                LocalizedText(action.to_string()),
            ));
        });
}

#[derive(Component)]
pub struct EquipmentCard {
    pub key: String,
    pub is_equipped: bool,
}

pub fn handle_playing_action_clicks(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    action_btn_q: Query<&ActionButton>,
    toast_container_q: Query<Entity, With<ToastContainer>>,
) {
    use rand::RngExt;

    if let Ok(action) = action_btn_q.get(event.entity) {
        let cost_gold = match action.0 {
            "craft" => 15,
            "shop" => 30,
            _ => 0,
        };

        if player.gold < cost_gold {
            play_audio_msg.write(PlayAudioMsg::new("error"));
            return;
        }

        // Play action sound (work/study/rest use their own sound, others use generic button)
        if matches!(action.0, "work" | "study" | "rest") {
            play_audio_msg.write(PlayAudioMsg::new(action.0));
        } else {
            play_audio_msg.write(PlayAudioMsg::new("button"));
        }
        player.gold -= cost_gold;

        // Helper to spawn a toast into the stacking container
        let spawn_toast = |commands: &mut Commands,
                           assets: &WorldAssets,
                           msg: String,
                           bg: Color,
                           border: Color,
                           text_color: Color,
                           container_q: &Query<Entity, With<ToastContainer>>| {
            if let Ok(container) = container_q.single() {
                let toast_text = msg.clone();
                commands.entity(container).with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(14.), Val::Px(9.)),
                                border: UiRect::all(Val::Px(2.)),
                                border_radius: BorderRadius::all(Val::Px(8.)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border),
                            GoldToast { timer: 3.5 },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(toast_text, "bold", 2.2, assets),
                                TextColor(text_color),
                            ));
                        });
                });
            }
        };

        // Handle the specific action
        let ap_cost = match action.0 {
            "hunt" => {
                let gold_earned = rand::rng().random_range(10..=20);
                player.gold += gold_earned;
                2
            },
            "work" => {
                let charisma = player.charisma() as i32;
                let level = player.level as i32;
                let base = charisma * level;
                let min_gold = (base * 4 / 5).max(1) as u32;
                let max_gold = (base * 6 / 5).max(2) as u32;
                let gold_earned = rand::rng().random_range(min_gold..=max_gold);
                player.gold += gold_earned;
                spawn_toast(
                    &mut commands, &assets,
                    format!("+ {} gold earned!", gold_earned),
                    Color::srgba(0.18, 0.13, 0.02, 0.93),
                    Color::srgb(0.85, 0.65, 0.15),
                    Color::srgb(1.0, 0.88, 0.30),
                    &toast_container_q,
                );
                1
            },
            "shop" => {
                let lvl = player.level;
                let items: Vec<&crate::core::catalog::GeneratedEquipment> = crate::core::catalog::GENERATED_EQUIPMENT
                    .iter()
                    .filter(|eq| eq.kind == "consumable" && eq.level <= lvl)
                    .collect();
                use rand::seq::IndexedRandom;
                if let Some(item) = items.choose(&mut rand::rng()) {
                    let name = item.name.to_string();
                    reward_equipment(&mut player, name);
                }
                0
            },
            "quest" => {
                let gold_earned = rand::rng().random_range(20..=40);
                if rand::rng().random_bool(0.5) {
                    let items: Vec<&crate::core::catalog::GeneratedEquipment> = crate::core::catalog::GENERATED_EQUIPMENT
                        .iter()
                        .filter(|eq| eq.level <= player.level)
                        .collect();
                    use rand::seq::IndexedRandom;
                    if let Some(item) = items.choose(&mut rand::rng()) {
                        reward_equipment(&mut player, item.name.to_string());
                    }
                }
                player.gold += gold_earned;
                3
            },
            "train" => {
                let attr_idx = rand::rng().random_range(0..6);
                match attr_idx {
                    0 => player.strength += 1,
                    1 => player.dexterity += 1,
                    2 => player.constitution += 1,
                    3 => player.intelligence += 1,
                    4 => player.wisdom += 1,
                    _ => player.charisma += 1,
                };
                2
            },
            "craft" => {
                let items: Vec<&crate::core::catalog::GeneratedEquipment> = crate::core::catalog::GENERATED_EQUIPMENT
                    .iter()
                    .filter(|eq| eq.level == player.level)
                    .collect();
                use rand::seq::IndexedRandom;
                if let Some(item) = items.choose(&mut rand::rng()) {
                    reward_equipment(&mut player, item.name.to_string());
                }
                2
            },
            "rest" => {
                let wisdom = player.wisdom() as i32;
                let level = player.level as i32;
                let base = wisdom * level;
                let min_recover = (base * 4 / 5).max(1) as f32;
                let max_recover = (base * 6 / 5).max(2) as f32;
                let recover_amount = rand::rng().random_range(min_recover..=max_recover);

                let max_hp = player.max_health().floor();
                let max_mp = player.max_mana().floor();
                let health_before = player.health;
                let mana_before = player.mana;
                player.health = (player.health + recover_amount).min(max_hp);
                player.mana = (player.mana + recover_amount).min(max_mp);
                let health_gained = (player.health - health_before).round() as i32;
                let mana_gained = (player.mana - mana_before).round() as i32;

                let mut pet_gained = 0;
                if player.pet.is_some() {
                    let pet = player.pet.unwrap();
                    let pet_max_hp = pet.stats().health as f32;
                    let pet_health_before = player.pet_health.unwrap_or(pet_max_hp);
                    let new_pet_health = (pet_health_before + recover_amount).min(pet_max_hp);
                    player.pet_health = Some(new_pet_health);
                    pet_gained = (new_pet_health - pet_health_before).round() as i32;
                }

                // Small chance of permanently increasing max health / max mana
                let wisdom_bonus = (player.wisdom() as f32 - 10.).max(0.) * 0.005;
                let max_chance = (0.05 + wisdom_bonus).min(0.20) as f64;
                let mut bonus_lines = Vec::new();
                if pet_gained > 0 {
                    bonus_lines.push(format!("{} +{} health!", player.pet_name, pet_gained));
                }
                if rand::rng().random_bool(max_chance) {
                    let gain = rand::rng().random_range(2.0_f32..=5.0_f32).round();
                    player.bonus_max_health += gain;
                    player.health = (player.health + gain).min(player.max_health().floor());
                    bonus_lines.push(format!("+{} max health!", gain as i32));
                }
                if rand::rng().random_bool(max_chance) {
                    let gain = rand::rng().random_range(2.0_f32..=5.0_f32).round();
                    player.bonus_max_mana += gain;
                    player.mana = (player.mana + gain).min(player.max_mana().floor());
                    bonus_lines.push(format!("+{} max mana!", gain as i32));
                }

                let mut toast_parts = vec![
                    format!("Recovered {} health, {} mana.", health_gained, mana_gained),
                ];
                toast_parts.extend(bonus_lines);
                let toast_msg = toast_parts.join("  ");
                spawn_toast(
                    &mut commands, &assets,
                    toast_msg,
                    Color::srgba(0.08, 0.16, 0.12, 0.93),
                    Color::srgb(0.25, 0.75, 0.50),
                    Color::srgb(0.60, 1.0, 0.75),
                    &toast_container_q,
                );
                1
            },
            "study" => {
                use rand::seq::IndexedRandom;
                let int_bonus = (player.intelligence() as f32 - 10.).max(0.) * 0.025;
                let perk_chance = (0.333 + int_bonus).min(0.65) as f64;
                let ability_chance = (0.200 + int_bonus).min(0.45) as f64;
                let wisdom_chance = 0.05_f64;
                let class_hint = player.class.to_lowername();

                // Weighted level selection: 50% current, 25% lower, 25% higher
                let level_offset: i8 = match rand::rng().random_range(0u8..4) {
                    0 => 1,
                    1 | 2 => 0,
                    _ => -1,
                };
                let target_level = (player.level as i8 + level_offset).clamp(1, 20) as u8;

                let mut toast_msg = "Nothing new learned.".to_string();

                // Roll for perk first (higher chance)
                if rand::rng().random_bool(perk_chance) {
                    let candidates: Vec<&crate::core::catalog::GeneratedPerk> = crate::core::catalog::GENERATED_PERKS
                        .iter()
                        .filter(|pk| {
                            (pk.level == target_level || pk.level == player.level)
                                && pk.class_hint == class_hint
                                && !player.perks.contains(&pk.name.to_string())
                        })
                        .collect();
                    if let Some(perk) = candidates.choose(&mut rand::rng()) {
                        let name = capitalize_words(&perk.name.to_string());
                        player.perks.push(perk.name.to_string());
                        toast_msg = format!("Learned perk: {}!", name);
                    }
                // Otherwise roll for ability (lower chance)
                } else if rand::rng().random_bool(ability_chance) {
                    let candidates: Vec<&crate::core::catalog::GeneratedAbility> = crate::core::catalog::GENERATED_ABILITIES
                        .iter()
                        .filter(|ab| {
                            (ab.level == target_level || ab.level == player.level)
                                && ab.class_hint == class_hint
                                && !player.abilities.contains(&ab.name.to_string())
                        })
                        .collect();
                    if let Some(ability) = candidates.choose(&mut rand::rng()) {
                        let name = capitalize_words(&ability.name.to_string());
                        player.abilities.push(ability.name.to_string());
                        toast_msg = format!("Learned ability: {}!", name);
                    }
                }

                // Rare wisdom bonus (independent)
                if rand::rng().random_bool(wisdom_chance) {
                    player.wisdom += 1;
                    if toast_msg == "Nothing new learned." {
                        toast_msg = "+1 Wisdom!".to_string();
                    } else {
                        toast_msg = format!("{} +1 Wisdom!", toast_msg);
                    }
                }

                spawn_toast(
                    &mut commands, &assets,
                    toast_msg,
                    Color::srgba(0.08, 0.10, 0.20, 0.93),
                    Color::srgb(0.35, 0.55, 0.90),
                    Color::srgb(0.75, 0.90, 1.0),
                    &toast_container_q,
                );
                1
            },
            _ => 0,
        };

        // Deduct action points
        if player.ap <= ap_cost {
            let old_max_health = player.max_health();
            let old_max_mana = player.max_mana();

            player.level += 1;
            player.ap = 10 + (player.level as u32) * 2;
            // Base stat gains (+1 to all per level)
            player.strength += 1;
            player.dexterity += 1;
            player.constitution += 1;
            player.intelligence += 1;
            player.wisdom += 1;
            player.charisma += 1;
            // Bonus health/mana increase
            player.bonus_max_health += 10.;
            player.bonus_max_mana += 10.;

            let health_diff = player.max_health() - old_max_health;
            let mana_diff = player.max_mana() - old_max_mana;

            player.health = (player.health + health_diff).min(player.max_health().floor());
            player.mana = (player.mana + mana_diff).min(player.max_mana().floor());

            // Generate ability and perk choices for the new level
            let class_hint = player.class.to_lowername();
            let new_level = player.level;

            let mut ability_choices = Vec::new();
            let mut ability_pool: Vec<_> = crate::core::catalog::GENERATED_ABILITIES
                .iter()
                .filter(|ab| {
                    ab.level == new_level
                        && ab.class_hint == class_hint
                        && !player.abilities.contains(&ab.name.to_string())
                })
                .collect();
            for _ in 0..3 {
                if ability_pool.is_empty() { break; }
                let idx = rand::rng().random_range(0..ability_pool.len());
                ability_choices.push(ability_pool[idx].name.to_string());
                ability_pool.remove(idx);
            }

            let mut perk_choices = Vec::new();
            let mut perk_pool: Vec<_> = crate::core::catalog::GENERATED_PERKS
                .iter()
                .filter(|pk| {
                    pk.level == new_level
                        && pk.class_hint == class_hint
                        && !player.perks.contains(&pk.name.to_string())
                })
                .collect();
            for _ in 0..3 {
                if perk_pool.is_empty() { break; }
                let idx = rand::rng().random_range(0..perk_pool.len());
                perk_choices.push(perk_pool[idx].name.to_string());
                perk_pool.remove(idx);
            }

            let ability_chosen = if ability_choices.is_empty() { Some(0) } else { None };
            let perk_chosen = if perk_choices.is_empty() { Some(0) } else { None };

            *level_up = LevelUpPending {
                active: true,
                new_level,
                points_remaining: 2,
                attr_gains: [0; 6],
                ability_choices,
                perk_choices,
                ability_chosen,
                perk_chosen,
            };

            play_audio_msg.write(PlayAudioMsg::new("victory"));
        } else {
            player.ap -= ap_cost;
        }
    }
}

fn attr_to_idx(attr: Attribute) -> usize {
    match attr {
        Attribute::Strength => 0,
        Attribute::Dexterity => 1,
        Attribute::Constitution => 2,
        Attribute::Intelligence => 3,
        Attribute::Wisdom => 4,
        Attribute::Charisma => 5,
    }
}

pub fn handle_attr_plus_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&LevelUpAttrPlusBtn>,
    mut level_up: ResMut<LevelUpPending>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        let idx = attr_to_idx(btn.0);
        if level_up.points_remaining > 0 && level_up.attr_gains[idx] < 2 {
            level_up.attr_gains[idx] += 1;
            level_up.points_remaining -= 1;
        }
    }
}

pub fn handle_attr_minus_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&LevelUpAttrMinusBtn>,
    mut level_up: ResMut<LevelUpPending>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        let idx = attr_to_idx(btn.0);
        if level_up.attr_gains[idx] > 0 {
            level_up.attr_gains[idx] -= 1;
            level_up.points_remaining += 1;
        }
    }
}

pub fn handle_ability_choice_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&LevelUpAbilityChoiceBtn>,
    mut level_up: ResMut<LevelUpPending>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        level_up.ability_chosen = Some(btn.0);
    }
}

pub fn handle_perk_choice_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&LevelUpPerkChoiceBtn>,
    mut level_up: ResMut<LevelUpPending>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        level_up.perk_chosen = Some(btn.0);
    }
}

pub fn handle_level_up_confirm(
    _event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut level_up: ResMut<LevelUpPending>,
) {
    let ability_ok = level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
    let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
    if level_up.points_remaining != 0 || !ability_ok || !perk_ok {
        return;
    }

    player.strength += level_up.attr_gains[0] as u8;
    player.dexterity += level_up.attr_gains[1] as u8;
    player.constitution += level_up.attr_gains[2] as u8;
    player.intelligence += level_up.attr_gains[3] as u8;
    player.wisdom += level_up.attr_gains[4] as u8;
    player.charisma += level_up.attr_gains[5] as u8;

    if let Some(idx) = level_up.ability_chosen {
        if let Some(name) = level_up.ability_choices.get(idx) {
            player.abilities.push(name.clone());
        }
    }
    if let Some(idx) = level_up.perk_chosen {
        if let Some(name) = level_up.perk_choices.get(idx) {
            player.perks.push(name.clone());
        }
    }

    level_up.active = false;
    level_up.attr_gains = [0; 6];
    level_up.ability_chosen = None;
    level_up.perk_chosen = None;
}

pub fn manage_level_up_overlay(
    level_up: Res<LevelUpPending>,
    overlay_q: Query<Entity, With<LevelUpOverlayCmp>>,
    player: Res<Player>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    settings: Res<Settings>,
    localization: Res<Localization>,
) {
    let overlay_exists = !overlay_q.is_empty();

    if !level_up.active && overlay_exists {
        for entity in overlay_q.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let lang = settings.language;

    if level_up.active && !overlay_exists {
        spawn_level_up_overlay(&mut commands, &assets, &level_up, &player, &localization, lang);
    } else if level_up.active && overlay_exists && level_up.is_changed() {
        for entity in overlay_q.iter() {
            commands.entity(entity).despawn();
        }
        spawn_level_up_overlay(&mut commands, &assets, &level_up, &player, &localization, lang);
    }
}

fn spawn_level_up_overlay(
    commands: &mut Commands,
    assets: &WorldAssets,
    level_up: &LevelUpPending,
    player: &Player,
    localization: &Localization,
    lang: Language,
) {
    const GOLD: Color = Color::srgb(1.0, 0.85, 0.2);
    const SELECTED_BORDER: Color = Color::srgb(1.0, 0.85, 0.2);
    const UNSELECTED_BORDER: Color = BUTTON_BORDER_COLOR;

    let ability_ok = level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
    let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
    let confirm_ready = level_up.points_remaining == 0 && ability_ok && perk_ok;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(100.),
                height: Val::Vh(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.85)),
            GlobalZIndex(500),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
            LevelUpOverlayCmp,
        ))
        .with_children(|parent| {
            // Main Panel
            parent
                .spawn((
                    Node {
                        width: Val::Vw(74.),
                        height: Val::Vh(84.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::axes(Val::Px(32.), Val::Px(20.)),
                        ..default()
                    },
                    ImageNode::new(assets.image("banner_large")).with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    // Header / Title
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(6.)),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                add_text(
                                    format!("{} {}", localization.get("level", lang), level_up.new_level),
                                    "bold",
                                    3.0,
                                    assets,
                                ),
                                TextColor(BUTTON_TEXT_COLOR),
                            ));
                            parent.spawn((
                                add_text(
                                    localization.get("level_up_subtitle", lang).to_string(),
                                    "medium",
                                    1.3,
                                    assets,
                                ),
                                TextColor(Color::srgba(1., 1., 1., 0.7)),
                            ));
                        });

                    // Two-Column Grid Area
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Stretch,
                            height: percent(70.),
                            column_gap: Val::Px(24.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // --- LEFT COLUMN: Attributes ---
                            parent
                                .spawn(Node {
                                    width: percent(46.),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(4.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(localization.get("assign_points", lang).to_string(), "bold", 1.8, assets),
                                        TextColor(BUTTON_TEXT_COLOR),
                                    ));

                                    let pts_color = if level_up.points_remaining > 0 { GOLD } else { Color::WHITE };
                                    parent.spawn((
                                        add_text(
                                            format!("{}: {}", localization.get("points remaining", lang), level_up.points_remaining),
                                            "bold",
                                            1.5,
                                            assets,
                                        ),
                                        TextColor(pts_color),
                                    ));

                                    let attrs = [
                                        (Attribute::Strength, localization.get("strength", lang), player.strength, 0),
                                        (Attribute::Dexterity, localization.get("dexterity", lang), player.dexterity, 1),
                                        (Attribute::Constitution, localization.get("constitution", lang), player.constitution, 2),
                                        (Attribute::Intelligence, localization.get("intelligence", lang), player.intelligence, 3),
                                        (Attribute::Wisdom, localization.get("wisdom", lang), player.wisdom, 4),
                                        (Attribute::Charisma, localization.get("charisma", lang), player.charisma, 5),
                                    ];

                                    for (attr, name, base_val, idx) in attrs {
                                        let gain = level_up.attr_gains[idx];
                                        let can_plus = level_up.points_remaining > 0 && gain < 2;
                                        let can_minus = gain > 0;

                                        parent
                                            .spawn((
                                                Node {
                                                    flex_direction: FlexDirection::Row,
                                                    align_items: AlignItems::Center,
                                                    justify_content: JustifyContent::SpaceBetween,
                                                    padding: UiRect::axes(Val::Px(10.), Val::Px(4.)),
                                                    border: UiRect::all(Val::Px(1.)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgba(0.015, 0.025, 0.06, 0.65)),
                                                BorderColor::all(BUTTON_BORDER_COLOR),
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    add_text(name.to_string(), "bold", 1.5, assets),
                                                    TextColor(BUTTON_TEXT_COLOR),
                                                ));

                                                // Right side of attribute row (values and buttons)
                                                parent
                                                    .spawn(Node {
                                                        flex_direction: FlexDirection::Row,
                                                        align_items: AlignItems::Center,
                                                        column_gap: Val::Px(12.),
                                                        ..default()
                                                    })
                                                    .with_children(|parent| {
                                                        parent.spawn((
                                                            add_text(format!("{}", base_val), "medium", 1.5, assets),
                                                            TextColor(Color::WHITE),
                                                        ));

                                                        let gain_color = if gain > 0 { Color::srgb(0.3, 1.0, 0.3) } else { Color::srgba(1., 1., 1., 0.3) };
                                                        parent.spawn((
                                                            add_text(format!("+{}", gain), "bold", 1.5, assets),
                                                            TextColor(gain_color),
                                                            Node {
                                                                width: Val::Px(24.),
                                                                ..default()
                                                            },
                                                        ));

                                                        // Minus Button
                                                        let minus_col = if can_minus { NORMAL_BUTTON_COLOR } else { Color::srgba(0.05, 0.09, 0.22, 0.3) };
                                                        parent
                                                            .spawn((
                                                                Node {
                                                                    width: Val::Px(24.),
                                                                    height: Val::Px(24.),
                                                                    align_items: AlignItems::Center,
                                                                    justify_content: JustifyContent::Center,
                                                                    border: UiRect::all(Val::Px(1.)),
                                                                    ..default()
                                                                },
                                                                BackgroundColor(minus_col),
                                                                BorderColor::all(BUTTON_BORDER_COLOR),
                                                                Button,
                                                                Interaction::default(),
                                                                Pickable::default(),
                                                                LevelUpAttrMinusBtn(attr),
                                                            ))
                                                            .observe(handle_attr_minus_click)
                                                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                                                            .with_children(|parent| {
                                                                parent.spawn((
                                                                    add_text("-", "bold", 1.6, assets),
                                                                    TextColor(if can_minus { Color::WHITE } else { Color::srgba(1., 1., 1., 0.2) }),
                                                                ));
                                                            });

                                                        // Plus Button
                                                        let plus_col = if can_plus { NORMAL_BUTTON_COLOR } else { Color::srgba(0.05, 0.09, 0.22, 0.3) };
                                                        parent
                                                            .spawn((
                                                                Node {
                                                                    width: Val::Px(24.),
                                                                    height: Val::Px(24.),
                                                                    align_items: AlignItems::Center,
                                                                    justify_content: JustifyContent::Center,
                                                                    border: UiRect::all(Val::Px(1.)),
                                                                    ..default()
                                                                },
                                                                BackgroundColor(plus_col),
                                                                BorderColor::all(BUTTON_BORDER_COLOR),
                                                                Button,
                                                                Interaction::default(),
                                                                Pickable::default(),
                                                                LevelUpAttrPlusBtn(attr),
                                                            ))
                                                            .observe(handle_attr_plus_click)
                                                            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                                            .observe(cursor::<Out>(SystemCursorIcon::Default))
                                                            .with_children(|parent| {
                                                                parent.spawn((
                                                                    add_text("+", "bold", 1.6, assets),
                                                                    TextColor(if can_plus { Color::WHITE } else { Color::srgba(1., 1., 1., 0.2) }),
                                                                ));
                                                            });
                                                    });
                                            });
                                    }
                                });

                            // --- RIGHT COLUMN: Abilities & Perks ---
                            parent
                                .spawn(Node {
                                    width: percent(50.),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.),
                                    justify_content: JustifyContent::FlexStart,
                                    ..default()
                                })
                                .with_children(|parent| {
                                    // --- Abilities Section ---
                                    if !level_up.ability_choices.is_empty() {
                                        parent
                                            .spawn(Node {
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(8.),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    add_text(localization.get("choose_ability", lang).to_string(), "bold", 1.8, assets),
                                                    TextColor(BUTTON_TEXT_COLOR),
                                                ));

                                                parent
                                                    .spawn(Node {
                                                        flex_direction: FlexDirection::Row,
                                                        column_gap: Val::Px(6.),
                                                        justify_content: JustifyContent::SpaceBetween,
                                                        ..default()
                                                    })
                                                    .with_children(|parent| {
                                                        for (i, name) in level_up.ability_choices.iter().enumerate() {
                                                            let is_selected = level_up.ability_chosen == Some(i);
                                                            let border_col = if is_selected { SELECTED_BORDER } else { UNSELECTED_BORDER };
                                                            let border_thickness = if is_selected { 3. } else { 1. };
                                                            let ab_name = name.clone();
                                                            parent
                                                                .spawn((
                                                                    Node {
                                                                        flex_direction: FlexDirection::Column,
                                                                        align_items: AlignItems::Center,
                                                                        justify_content: JustifyContent::Center,
                                                                        padding: UiRect::all(Val::Px(4.)),
                                                                        border: UiRect::all(Val::Px(border_thickness)),
                                                                        row_gap: Val::Px(2.),
                                                                        width: percent(32.),
                                                                        height: Val::Px(95.),
                                                                        ..default()
                                                                    },
                                                                    BackgroundColor(if is_selected {
                                                                        Color::srgba(0.20, 0.16, 0.04, 0.95)
                                                                    } else {
                                                                        PLACEHOLDER_COLOR
                                                                    }),
                                                                    BorderColor::all(border_col),
                                                                    Button,
                                                                    Interaction::default(),
                                                                    Pickable::default(),
                                                                    LevelUpAbilityChoiceBtn(i),
                                                                ))
                                                                .observe(handle_ability_choice_click)
                                                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                                                .with_children(|parent| {
                                                                    parent.spawn((
                                                                        Node {
                                                                            width: Val::Px(28.),
                                                                            height: Val::Px(28.),
                                                                            flex_shrink: 0.,
                                                                            ..default()
                                                                        },
                                                                        ImageNode::new(assets.image(ab_name.as_str()))
                                                                            .with_mode(NodeImageMode::Stretch),
                                                                    ));
                                                                    parent.spawn((
                                                                        add_text(translate_game_term(name, localization, lang), "bold", 1.2, assets),
                                                                        TextColor(BUTTON_TEXT_COLOR),
                                                                        Node {
                                                                            align_self: AlignSelf::Center,
                                                                            ..default()
                                                                        },
                                                                    ));
                                                                    if let Some(ab) = crate::core::catalog::get_ability(name.as_str()) {
                                                                        parent.spawn((
                                                                            add_text(format!("Lv. {}", ab.level), "medium", 1.1, assets),
                                                                            TextColor(Color::WHITE),
                                                                        ));
                                                                    }
                                                                });
                                                        }
                                                    });
                                            });
                                    }

                                    // --- Perks Section ---
                                    if !level_up.perk_choices.is_empty() {
                                        parent
                                            .spawn(Node {
                                                flex_direction: FlexDirection::Column,
                                                row_gap: Val::Px(8.),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    add_text(localization.get("choose_perk", lang).to_string(), "bold", 1.8, assets),
                                                    TextColor(BUTTON_TEXT_COLOR),
                                                ));

                                                parent
                                                    .spawn(Node {
                                                        flex_direction: FlexDirection::Row,
                                                        column_gap: Val::Px(6.),
                                                        justify_content: JustifyContent::SpaceBetween,
                                                        ..default()
                                                    })
                                                    .with_children(|parent| {
                                                        for (i, name) in level_up.perk_choices.iter().enumerate() {
                                                            let is_selected = level_up.perk_chosen == Some(i);
                                                            let border_col = if is_selected { SELECTED_BORDER } else { UNSELECTED_BORDER };
                                                            let border_thickness = if is_selected { 3. } else { 1. };
                                                            let pk_name = name.clone();
                                                            parent
                                                                .spawn((
                                                                    Node {
                                                                        flex_direction: FlexDirection::Column,
                                                                        align_items: AlignItems::Center,
                                                                        justify_content: JustifyContent::Center,
                                                                        padding: UiRect::all(Val::Px(4.)),
                                                                        border: UiRect::all(Val::Px(border_thickness)),
                                                                        row_gap: Val::Px(2.),
                                                                        width: percent(32.),
                                                                        height: Val::Px(95.),
                                                                        ..default()
                                                                    },
                                                                    BackgroundColor(if is_selected {
                                                                        Color::srgba(0.20, 0.16, 0.04, 0.95)
                                                                    } else {
                                                                        PLACEHOLDER_COLOR
                                                                    }),
                                                                    BorderColor::all(border_col),
                                                                    Button,
                                                                    Interaction::default(),
                                                                    Pickable::default(),
                                                                    LevelUpPerkChoiceBtn(i),
                                                                ))
                                                                .observe(handle_perk_choice_click)
                                                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                                                .with_children(|parent| {
                                                                    parent.spawn((
                                                                        Node {
                                                                            width: Val::Px(28.),
                                                                            height: Val::Px(28.),
                                                                            flex_shrink: 0.,
                                                                            ..default()
                                                                        },
                                                                        ImageNode::new(assets.image(pk_name.as_str()))
                                                                            .with_mode(NodeImageMode::Stretch),
                                                                    ));
                                                                    parent.spawn((
                                                                        add_text(translate_game_term(name, localization, lang), "bold", 1.2, assets),
                                                                        TextColor(BUTTON_TEXT_COLOR),
                                                                        Node {
                                                                            align_self: AlignSelf::Center,
                                                                            ..default()
                                                                        },
                                                                    ));
                                                                    if let Some(pk) = crate::core::catalog::get_perk(name.as_str()) {
                                                                        parent.spawn((
                                                                            add_text(format!("Lv. {}", pk.level), "medium", 1.1, assets),
                                                                            TextColor(Color::WHITE),
                                                                        ));
                                                                    }
                                                                });
                                                        }
                                                    });
                                            });
                                    }
                                });
                        });

                    // --- Bottom Footer Area with Confirm Button ---
                    let confirm_bg = if confirm_ready { GOLD } else { Color::srgba(0.08, 0.12, 0.22, 0.5) };
                    let confirm_txt = if confirm_ready { Color::BLACK } else { Color::srgba(1.0, 0.85, 0.2, 0.3) };
                    let confirm_label = if confirm_ready {
                        localization.get("confirm_level_up", lang)
                    } else {
                        localization.get("complete_selections", lang)
                    };

                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.),
                            margin: UiRect::top(Val::Px(8.)),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    Node {
                                        align_self: AlignSelf::Center,
                                        padding: UiRect::axes(Val::Px(36.), Val::Px(8.)),
                                        border: UiRect::all(Val::Px(2.)),
                                        ..default()
                                    },
                                    BackgroundColor(confirm_bg),
                                    BorderColor::all(if confirm_ready { GOLD } else { BUTTON_BORDER_COLOR }),
                                    Button,
                                    Interaction::default(),
                                    Pickable::default(),
                                    LevelUpConfirmBtn,
                                ))
                                .observe(handle_level_up_confirm)
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(confirm_label.to_string(), "bold", 1.8, assets),
                                        TextColor(confirm_txt),
                                    ));
                                });

                            if confirm_ready {
                                parent.spawn((
                                    add_text(localization.get("press_enter_confirm", lang).to_string(), "medium", 1.2, assets),
                                    TextColor(GOLD),
                                ));
                            }
                        });
                });
        });
}

pub fn equip_item(player: &mut Player, key: &str) -> Option<&'static str> {
    if let Some(weapon) = crate::core::catalog::get_equipment(key) {
        // Remove from inventory first
        if let Some(pos) = player.inventory.iter().position(|k| k == key) {
            player.inventory.remove(pos);
        }

        match weapon.kind {
            "consumable" => {
                let name = weapon.name.to_lowercase();
                if name.contains("health") {
                    player.health = player.max_health().floor();
                } else if name.contains("mana") {
                    player.mana = player.max_mana().floor();
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
                    player.health = player.max_health().floor();
                    player.mana = player.max_mana().floor();
                } else if name.contains("antidote") {
                    player.health = player.max_health().floor();
                }
                return Some("button");
            },
            "helmet" => {
                if let Some(old) = player.helmet.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "armor" => {
                if let Some(old) = player.armor.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "boots" => {
                if let Some(old) = player.boots.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "accessory" => {
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
            "gloves" => {
                if let Some(old) = player.gloves.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            "one_hand_weapon" => {
                if let Some(old_2h) = player.weapon_2h.take() {
                    player.inventory.push(old_2h);
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
            },
            "two_hand_weapon" => {
                if let Some(old_lh) = player.weapon_lh.take() {
                    player.inventory.push(old_lh);
                }
                if let Some(old_rh) = player.weapon_rh.take() {
                    player.inventory.push(old_rh);
                }
                if let Some(old_2h) = player.weapon_2h.replace(key.to_string()) {
                    player.inventory.push(old_2h);
                }
            },
            "offhand" => {
                if let Some(old_2h) = player.weapon_2h.take() {
                    player.inventory.push(old_2h);
                }
                if let Some(old) = player.weapon_rh.replace(key.to_string()) {
                    player.inventory.push(old);
                }
            },
            _ => {}
        }
    }
    None
}

pub fn unequip_item(player: &mut Player, key: &str) {
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
    } else if player.weapon_2h.as_deref() == Some(key) {
        player.weapon_2h = None;
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
}

pub fn unequip_slot(player: &mut Player, slot: EquipSlot) -> bool {
    let key_opt = match slot {
        EquipSlot::Helmet => player.helmet.take(),
        EquipSlot::Accessory => player.accessory.take(),
        EquipSlot::Accessory2 => player.accessory2.take(),
        EquipSlot::WeaponLH => {
            player.weapon_lh.take().or(player.weapon_2h.take())
        },
        EquipSlot::WeaponRH => player.weapon_rh.take(),
        EquipSlot::Armor => player.armor.take(),
        EquipSlot::Boots => player.boots.take(),
        EquipSlot::Gloves => player.gloves.take(),
    };
    if let Some(key) = key_opt {
        player.inventory.push(key);
        true
    } else {
        false
    }
}

pub fn reward_equipment(player: &mut Player, key: String) {
    if let Some(weapon) = crate::core::catalog::get_equipment(&key) {
        let is_empty = match weapon.kind {
            "helmet" => player.helmet.is_none(),
            "armor" => player.armor.is_none(),
            "boots" => player.boots.is_none(),
            "gloves" => player.gloves.is_none(),
            "accessory" => player.accessory.is_none() || player.accessory2.is_none(),
            "one_hand_weapon" => player.weapon_2h.is_none() && (player.weapon_lh.is_none() || player.weapon_rh.is_none()),
            "two_hand_weapon" => player.weapon_lh.is_none() && player.weapon_rh.is_none() && player.weapon_2h.is_none(),
            "offhand" => player.weapon_2h.is_none() && player.weapon_rh.is_none(),
            _ => false,
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
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    card_q: Query<&EquipmentCard>,
) {
    if let Ok(card) = card_q.get(event.entity) {
        if card.is_equipped {
            unequip_item(&mut player, &card.key);
            play_audio_msg.write(PlayAudioMsg::new("click"));
        } else {
            // Check consumable restrictions
            if let Some(eq) = crate::core::catalog::get_equipment(&card.key) {
                if eq.kind == "consumable" {
                    let name = eq.name.to_lowercase();
                    let blocked = if name.contains("rejuvenation") {
                        player.health >= player.max_health().floor()
                            && player.mana >= player.max_mana().floor()
                    } else if name.contains("health") || name.contains("antidote") {
                        player.health >= player.max_health().floor()
                    } else if name.contains("mana") {
                        player.mana >= player.max_mana().floor()
                    } else {
                        false
                    };
                    if blocked {
                        play_audio_msg.write(PlayAudioMsg::new("error"));
                        return;
                    }
                }
            }
            let sound = equip_item(&mut player, &card.key).unwrap_or("click");
            play_audio_msg.write(PlayAudioMsg::new(sound));
        }
    }
}

pub fn tick_gold_toasts(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut GoldToast)>,
) {
    for (entity, mut toast) in &mut q {
        toast.timer -= time.delta_secs();
        if toast.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn handle_equipment_slot_click(
    event: On<Pointer<Click>>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    slot_q: Query<&EquipSlot>,
) {
    if let Ok(slot) = slot_q.get(event.entity) {
        if unequip_slot(&mut player, *slot) {
            play_audio_msg.write(PlayAudioMsg::new("click"));
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
                ..default()
            },
            BackgroundColor(BAR_BG_COLOR),
            BorderColor::all(BUTTON_BORDER_COLOR),
            Button,
            Interaction::default(),
            Pickable::default(),
            card,
        ))
        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
        .observe(recolor::<Out>(BAR_BG_COLOR))
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .observe(handle_equipment_card_click)
        .with_children(|parent| {
            if is_equipped {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(4.),
                        top: Val::Px(4.),
                        width: ICON_BADGE,
                        height: ICON_BADGE,
                        ..default()
                    },
                    ImageNode::new(assets.image("equipped"))
                        .with_mode(NodeImageMode::Stretch),
                ));
            }
            spawn_placeholder(parent, assets, image_key, ICON_ITEM);

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((add_text(name, "bold", 1.9, assets), TextColor(BUTTON_TEXT_COLOR)));

                    for line in lines {
                        parent.spawn((
                            add_text(line, "medium", 1.6, assets),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}
