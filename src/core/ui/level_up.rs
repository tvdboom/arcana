use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::{get_ability, get_perk};
use crate::core::constants::*;
use crate::core::localization::Localization;
use crate::core::menu::buttons::DisabledButton;
use crate::core::menu::utils::{add_text, spawn_rich_text_row};
use crate::core::player::{Attribute, Player};
use crate::core::settings::Language;
use crate::core::ui::playing::RightColumnTooltip;
use crate::core::utils::cursor;
use crate::utils::{capitalize_words, NameFromEnum};

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

#[derive(Message)]
pub struct ApplyLevelUpMsg;

#[derive(Component)]
pub struct LevelUpOverlayCmp;

#[derive(Component)]
pub struct LevelUpAttrPlusBtn(pub Attribute);

#[derive(Component)]
pub struct LevelUpAttrMinusBtn(pub Attribute);

#[derive(Component)]
pub struct LevelUpAbilityChoiceBtn(pub usize);

#[derive(Component)]
pub struct LevelUpPerkChoiceBtn(pub usize);

#[derive(Component)]
pub struct LevelUpConfirmBtn;

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
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        let idx = attr_to_idx(btn.0);
        if level_up.points_remaining > 0 && level_up.attr_gains[idx] < 2 {
            level_up.attr_gains[idx] += 1;
            level_up.points_remaining -= 1;
            play_audio_msg.write(PlayAudioMsg::new("button"));
        }
    }
}

pub fn handle_attr_minus_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&LevelUpAttrMinusBtn>,
    mut level_up: ResMut<LevelUpPending>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        let idx = attr_to_idx(btn.0);
        if level_up.attr_gains[idx] > 0 {
            level_up.attr_gains[idx] -= 1;
            level_up.points_remaining += 1;
            play_audio_msg.write(PlayAudioMsg::new("button"));
        }
    }
}

pub fn handle_ability_choice_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&LevelUpAbilityChoiceBtn>,
    mut level_up: ResMut<LevelUpPending>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        level_up.ability_chosen = Some(btn.0);
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn handle_perk_choice_click(
    event: On<Pointer<Click>>,
    btn_q: Query<&LevelUpPerkChoiceBtn>,
    mut level_up: ResMut<LevelUpPending>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if let Ok(btn) = btn_q.get(event.entity) {
        level_up.perk_chosen = Some(btn.0);
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
}

pub fn handle_level_up_confirm(
    _event: On<Pointer<Click>>,
    mut apply_level_up_msg: MessageWriter<ApplyLevelUpMsg>,
) {
    apply_level_up_msg.write(ApplyLevelUpMsg);
}

pub fn apply_level_up_system(
    mut apply_level_up_msg: MessageReader<ApplyLevelUpMsg>,
    mut player: ResMut<Player>,
    mut level_up: ResMut<LevelUpPending>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    if !apply_level_up_msg.is_empty() {
        let ability_ok = level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
        let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
        if level_up.points_remaining == 0 && ability_ok && perk_ok {
            let old_hp = player.max_health();
            let old_mp = player.max_mana();

            play_audio_msg.write(PlayAudioMsg::new("button"));

            player.strength += level_up.attr_gains[0] as u32;
            player.dexterity += level_up.attr_gains[1] as u32;
            player.constitution += level_up.attr_gains[2] as u32;
            player.intelligence += level_up.attr_gains[3] as u32;
            player.wisdom += level_up.attr_gains[4] as u32;
            player.charisma += level_up.attr_gains[5] as u32;

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

            player.update_health_mana(old_hp, old_mp);

            level_up.active = false;
            level_up.attr_gains = [0; 6];
            level_up.ability_chosen = None;
            level_up.perk_chosen = None;
        }

        apply_level_up_msg.clear();
    }
}

pub fn handle_confirm_over(
    event: On<Pointer<Over>>,
    level_up: Res<LevelUpPending>,
    mut btn_q: Query<&mut BackgroundColor>,
    children_q: Query<&Children>,
    mut text_color_q: Query<&mut TextColor>,
) {
    let ability_ok = level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
    let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
    let confirm_ready = level_up.points_remaining == 0 && ability_ok && perk_ok;

    if confirm_ready {
        if let Ok(mut bg) = btn_q.get_mut(event.entity) {
            bg.0 = BUTTON_TEXT_COLOR;
        }
        if let Ok(children) = children_q.get(event.entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = text_color_q.get_mut(child) {
                    txt_col.0 = Color::BLACK;
                }
            }
        }
    }
}

pub fn handle_confirm_out(
    event: On<Pointer<Out>>,
    level_up: Res<LevelUpPending>,
    mut btn_q: Query<&mut BackgroundColor>,
    children_q: Query<&Children>,
    mut text_color_q: Query<&mut TextColor>,
) {
    let ability_ok = level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
    let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
    let confirm_ready = level_up.points_remaining == 0 && ability_ok && perk_ok;

    if let Ok(mut bg) = btn_q.get_mut(event.entity) {
        if confirm_ready {
            bg.0 = NORMAL_BUTTON_COLOR;
        } else {
            bg.0 = Color::srgba(0.05, 0.09, 0.22, 0.5);
        }
    }
    if let Ok(children) = children_q.get(event.entity) {
        for child in children.iter() {
            if let Ok(mut txt_col) = text_color_q.get_mut(child) {
                if confirm_ready {
                    txt_col.0 = BUTTON_TEXT_COLOR;
                } else {
                    txt_col.0 = Color::srgba(0.6, 0.55, 0.4, 0.5);
                }
            }
        }
    }
}

pub fn handle_confirm_press(
    event: On<Pointer<Press>>,
    level_up: Res<LevelUpPending>,
    mut btn_q: Query<&mut BackgroundColor>,
    children_q: Query<&Children>,
    mut text_color_q: Query<&mut TextColor>,
) {
    let ability_ok = level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
    let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
    let confirm_ready = level_up.points_remaining == 0 && ability_ok && perk_ok;

    if confirm_ready {
        if let Ok(mut bg) = btn_q.get_mut(event.entity) {
            bg.0 = Color::srgba_u8(30, 30, 50, 240);
        }
        if let Ok(children) = children_q.get(event.entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = text_color_q.get_mut(child) {
                    txt_col.0 = Color::srgba(1.0, 1.0, 1.0, 0.4);
                }
            }
        }
    }
}

pub fn handle_confirm_release(
    event: On<Pointer<Release>>,
    level_up: Res<LevelUpPending>,
    mut btn_q: Query<&mut BackgroundColor>,
    children_q: Query<&Children>,
    mut text_color_q: Query<&mut TextColor>,
) {
    let ability_ok = level_up.ability_choices.is_empty() || level_up.ability_chosen.is_some();
    let perk_ok = level_up.perk_choices.is_empty() || level_up.perk_chosen.is_some();
    let confirm_ready = level_up.points_remaining == 0 && ability_ok && perk_ok;

    if confirm_ready {
        if let Ok(mut bg) = btn_q.get_mut(event.entity) {
            bg.0 = BUTTON_TEXT_COLOR;
        }
        if let Ok(children) = children_q.get(event.entity) {
            for child in children.iter() {
                if let Ok(mut txt_col) = text_color_q.get_mut(child) {
                    txt_col.0 = Color::BLACK;
                }
            }
        }
    }
}

pub fn manage_level_up_overlay(
    level_up: Res<LevelUpPending>,
    game_state: Res<State<crate::core::states::GameState>>,
    mut overlay_q: Query<(&mut GlobalZIndex, &mut Pickable), With<LevelUpOverlayCmp>>,
    overlay_exists_q: Query<Entity, With<LevelUpOverlayCmp>>,
    player: Res<Player>,
    mut commands: Commands,
    assets: Res<WorldAssets>,
    settings: Res<crate::core::settings::Settings>,
    localization: Res<Localization>,
) {
    let overlay_exists = !overlay_exists_q.is_empty();
    let is_game_menu = *game_state.get() == crate::core::states::GameState::GameMenu;

    // Don't despawn when entering game menu, just adjust z-index and interactivity
    if !level_up.active && overlay_exists && !is_game_menu {
        for entity in overlay_exists_q.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // If overlay exists and we're in game menu, adjust z-index and make non-interactive
    if overlay_exists {
        for (mut z_index, mut pickable) in overlay_q.iter_mut() {
            if is_game_menu {
                *z_index = GlobalZIndex(100); // Below game menu
                pickable.should_block_lower = false;
                pickable.is_hoverable = false;
            } else {
                *z_index = GlobalZIndex(980); // Above everything else, but below tooltips (1000)
                pickable.should_block_lower = true;
                pickable.is_hoverable = true;
            }
        }
    }

    let lang = settings.language;

    if level_up.active && !overlay_exists {
        spawn_level_up_overlay(&mut commands, &assets, &level_up, &player, &localization, lang);
    } else if level_up.active && overlay_exists && level_up.is_changed() {
        for entity in overlay_exists_q.iter() {
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
            GlobalZIndex(980),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
            LevelUpOverlayCmp,
        ))
        .with_children(|parent| {
            // Main Panel - Made larger (Vw 88, Vh 96) with increased padding around panel to fit everything inside the background frame
            parent
                .spawn((
                    Node {
                        width: Val::Vw(88.),
                        height: Val::Vh(100.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect {
                            left: Val::Px(84.),
                            right: Val::Px(84.),
                            top: Val::Px(64.),
                            bottom: Val::Px(76.),
                        },
                        ..default()
                    },
                    ImageNode::new(assets.image("banner_large")).with_mode(NodeImageMode::Stretch),
                ))
                .with_children(|parent| {
                    // Main Content Area (not header/title)
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            flex_grow: 1.0,
                            row_gap: Val::Px(12.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Header / Title moved down inside content (added margin top to move it down significantly)
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    margin: UiRect {
                                        top: Val::Px(64.),
                                        bottom: Val::Px(16.),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        add_text(
                                            format!(
                                                "{} {}",
                                                localization.get("general.level", lang),
                                                level_up.new_level
                                            ),
                                            "bold",
                                            3.8,
                                            assets,
                                        ),
                                        TextColor(BUTTON_TEXT_COLOR),
                                    ));
                                });

                            // Two-Column Grid Area (centered to reduce space between columns and move attributes to the right)
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::FlexStart,
                                    flex_grow: 1.0,
                                    column_gap: Val::Px(32.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            width: percent(32.),
                                            flex_direction: FlexDirection::Column,
                                            row_gap: Val::Px(4.),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn((
                                                add_text(
                                                    localization
                                                        .get("general.assign_points", lang)
                                                        .to_string(),
                                                    "bold",
                                                    2.2,
                                                    assets,
                                                ),
                                                TextColor(BUTTON_TEXT_COLOR),
                                            ));

                                            let pts_color = if level_up.points_remaining > 0 {
                                                GOLD
                                            } else {
                                                Color::WHITE
                                            };
                                            parent.spawn((
                                                add_text(
                                                    format!(
                                                        "{}: {}",
                                                        localization
                                                            .get("general.points_remaining", lang),
                                                        level_up.points_remaining
                                                    ),
                                                    "bold",
                                                    1.8,
                                                    assets,
                                                ),
                                                TextColor(pts_color),
                                            ));

                                            let attrs = [
                                                (
                                                    Attribute::Strength,
                                                    localization.get(
                                                        &format!(
                                                            "attribute.{}",
                                                            Attribute::Strength.to_lowername()
                                                        ),
                                                        lang,
                                                    ),
                                                    player.strength(),
                                                    0,
                                                ),
                                                (
                                                    Attribute::Dexterity,
                                                    localization.get(
                                                        &format!(
                                                            "attribute.{}",
                                                            Attribute::Dexterity.to_lowername()
                                                        ),
                                                        lang,
                                                    ),
                                                    player.dexterity(),
                                                    1,
                                                ),
                                                (
                                                    Attribute::Constitution,
                                                    localization.get(
                                                        &format!(
                                                            "attribute.{}",
                                                            Attribute::Constitution.to_lowername()
                                                        ),
                                                        lang,
                                                    ),
                                                    player.constitution(),
                                                    2,
                                                ),
                                                (
                                                    Attribute::Intelligence,
                                                    localization.get(
                                                        &format!(
                                                            "attribute.{}",
                                                            Attribute::Intelligence.to_lowername()
                                                        ),
                                                        lang,
                                                    ),
                                                    player.intelligence(),
                                                    3,
                                                ),
                                                (
                                                    Attribute::Wisdom,
                                                    localization.get(
                                                        &format!(
                                                            "attribute.{}",
                                                            Attribute::Wisdom.to_lowername()
                                                        ),
                                                        lang,
                                                    ),
                                                    player.wisdom(),
                                                    4,
                                                ),
                                                (
                                                    Attribute::Charisma,
                                                    localization.get(
                                                        &format!(
                                                            "attribute.{}",
                                                            Attribute::Charisma.to_lowername()
                                                        ),
                                                        lang,
                                                    ),
                                                    player.charisma(),
                                                    5,
                                                ),
                                            ];

                                            for (attr, name, base_val, idx) in attrs {
                                                let gain = level_up.attr_gains[idx];
                                                let can_plus =
                                                    level_up.points_remaining > 0 && gain < 2;
                                                let can_minus = gain > 0;

                                                parent
                                                    .spawn((
                                                        Node {
                                                            flex_direction: FlexDirection::Row,
                                                            align_items: AlignItems::Center,
                                                            justify_content:
                                                                JustifyContent::SpaceBetween,
                                                            padding: UiRect::axes(
                                                                Val::Px(10.),
                                                                Val::Px(4.),
                                                            ),
                                                            border: UiRect::all(Val::Px(1.)),
                                                            ..default()
                                                        },
                                                        BackgroundColor(Color::srgba(
                                                            0.015, 0.025, 0.06, 0.65,
                                                        )),
                                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                                    ))
                                                    .with_children(|parent| {
                                                        parent.spawn((
                                                            add_text(
                                                                name.to_string(),
                                                                "bold",
                                                                1.8,
                                                                assets,
                                                            ),
                                                            TextColor(BUTTON_TEXT_COLOR),
                                                        ));

                                                        // Right side - reduced width modifier section
                                                        parent
                                                            .spawn(Node {
                                                                flex_direction: FlexDirection::Row,
                                                                align_items: AlignItems::Center,
                                                                column_gap: Val::Px(8.),
                                                                ..default()
                                                            })
                                                            .with_children(|parent| {
                                                                parent.spawn((
                                                                    add_text(
                                                                        format!("{}", base_val),
                                                                        "medium",
                                                                        1.8,
                                                                        assets,
                                                                    ),
                                                                    TextColor(Color::WHITE),
                                                                ));

                                                                let gain_color = if gain > 0 {
                                                                    Color::srgb(1.0, 0.85, 0.2)
                                                                // GOLD color
                                                                } else {
                                                                    Color::srgba(1., 1., 1., 0.3)
                                                                };
                                                                parent.spawn((
                                                                    add_text(
                                                                        format!("+{}", gain),
                                                                        "bold",
                                                                        1.8,
                                                                        assets,
                                                                    ),
                                                                    TextColor(gain_color),
                                                                    Node {
                                                                        width: Val::Px(24.),
                                                                        ..default()
                                                                    },
                                                                ));

                                                                // Minus Button
                                                                let minus_col = if can_minus {
                                                                    NORMAL_BUTTON_COLOR
                                                                } else {
                                                                    Color::srgba(
                                                                        0.05, 0.09, 0.22, 0.3,
                                                                    )
                                                                };
                                                                let mut m_btn = parent.spawn((
                                                                    Node {
                                                                        width: Val::Px(20.),
                                                                        height: Val::Px(20.),
                                                                        align_items:
                                                                            AlignItems::Center,
                                                                        justify_content:
                                                                            JustifyContent::Center,
                                                                        border: UiRect::all(
                                                                            Val::Px(1.),
                                                                        ),
                                                                        ..default()
                                                                    },
                                                                    BackgroundColor(minus_col),
                                                                    BorderColor::all(
                                                                        BUTTON_BORDER_COLOR,
                                                                    ),
                                                                    Button,
                                                                    Interaction::default(),
                                                                    Pickable::default(),
                                                                    LevelUpAttrMinusBtn(attr),
                                                                ));
                                                                if !can_minus {
                                                                    m_btn.insert(DisabledButton);
                                                                }
                                                                m_btn
                                                                    .observe(
                                                                        handle_attr_minus_click,
                                                                    )
                                                                    .observe(cursor::<Over>(
                                                                        SystemCursorIcon::Pointer,
                                                                    ))
                                                                    .observe(cursor::<Out>(
                                                                        SystemCursorIcon::Default,
                                                                    ))
                                                                    .with_children(|parent| {
                                                                        parent.spawn((
                                                                            add_text(
                                                                                "-", "bold", 1.4,
                                                                                assets,
                                                                            ),
                                                                            TextColor(
                                                                                if can_minus {
                                                                                    Color::WHITE
                                                                                } else {
                                                                                    Color::srgba(
                                                                                        1., 1., 1.,
                                                                                        0.2,
                                                                                    )
                                                                                },
                                                                            ),
                                                                        ));
                                                                    });

                                                                // Plus Button
                                                                let plus_col = if can_plus {
                                                                    NORMAL_BUTTON_COLOR
                                                                } else {
                                                                    Color::srgba(
                                                                        0.05, 0.09, 0.22, 0.3,
                                                                    )
                                                                };
                                                                let mut p_btn = parent.spawn((
                                                                    Node {
                                                                        width: Val::Px(20.),
                                                                        height: Val::Px(20.),
                                                                        align_items:
                                                                            AlignItems::Center,
                                                                        justify_content:
                                                                            JustifyContent::Center,
                                                                        border: UiRect::all(
                                                                            Val::Px(1.),
                                                                        ),
                                                                        ..default()
                                                                    },
                                                                    BackgroundColor(plus_col),
                                                                    BorderColor::all(
                                                                        BUTTON_BORDER_COLOR,
                                                                    ),
                                                                    Button,
                                                                    Interaction::default(),
                                                                    Pickable::default(),
                                                                    LevelUpAttrPlusBtn(attr),
                                                                ));
                                                                if !can_plus {
                                                                    p_btn.insert(DisabledButton);
                                                                }
                                                                p_btn
                                                                    .observe(handle_attr_plus_click)
                                                                    .observe(cursor::<Over>(
                                                                        SystemCursorIcon::Pointer,
                                                                    ))
                                                                    .observe(cursor::<Out>(
                                                                        SystemCursorIcon::Default,
                                                                    ))
                                                                    .with_children(|parent| {
                                                                        parent.spawn((
                                                                            add_text(
                                                                                "+", "bold", 1.4,
                                                                                assets,
                                                                            ),
                                                                            TextColor(
                                                                                if can_plus {
                                                                                    Color::WHITE
                                                                                } else {
                                                                                    Color::srgba(
                                                                                        1., 1., 1.,
                                                                                        0.2,
                                                                                    )
                                                                                },
                                                                            ),
                                                                        ));
                                                                    });
                                                            });
                                                    });
                                            }
                                        });

                                    // --- RIGHT COLUMN: Abilities & Perks (Reduced width to 42% so cards fit within the background) ---
                                    parent
                                        .spawn(Node {
                                            width: percent(42.),
                                            flex_direction: FlexDirection::Column,
                                            row_gap: Val::Px(4.),
                                            justify_content: JustifyContent::FlexStart,
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // --- Abilities Section (row_gap reduced from 8 to 4) ---
                                            if !level_up.ability_choices.is_empty() {
                                                parent
                                                    .spawn(Node {
                                                        flex_direction: FlexDirection::Column,
                                                        row_gap: Val::Px(4.),
                                                        ..default()
                                                    })
                                                    .with_children(|parent| {
                                                        parent.spawn((
                                                            add_text(
                                                                localization
                                                                    .get(
                                                                        "general.choose_ability",
                                                                        lang,
                                                                    )
                                                                    .to_string(),
                                                                "bold",
                                                                2.2,
                                                                assets,
                                                            ),
                                                            TextColor(BUTTON_TEXT_COLOR),
                                                        ));

                                                        // Abilities stacked vertically
                                                        for (i, name) in level_up
                                                            .ability_choices
                                                            .iter()
                                                            .enumerate()
                                                        {
                                                            let is_selected =
                                                                level_up.ability_chosen == Some(i);
                                                            spawn_choice_card(
                                                                parent,
                                                                assets,
                                                                localization,
                                                                lang,
                                                                name,
                                                                is_selected,
                                                                true, // is_ability
                                                                i,
                                                            );
                                                        }
                                                    });
                                            }

                                            // --- Perks Section (row_gap reduced from 8 to 4) ---
                                            if !level_up.perk_choices.is_empty() {
                                                parent
                                                    .spawn(Node {
                                                        flex_direction: FlexDirection::Column,
                                                        row_gap: Val::Px(4.),
                                                        ..default()
                                                    })
                                                    .with_children(|parent| {
                                                        parent.spawn((
                                                            add_text(
                                                                localization
                                                                    .get(
                                                                        "general.choose_perk",
                                                                        lang,
                                                                    )
                                                                    .to_string(),
                                                                "bold",
                                                                2.2,
                                                                assets,
                                                            ),
                                                            TextColor(BUTTON_TEXT_COLOR),
                                                        ));

                                                        // Perks stacked vertically
                                                        for (i, name) in
                                                            level_up.perk_choices.iter().enumerate()
                                                        {
                                                            let is_selected =
                                                                level_up.perk_chosen == Some(i);
                                                            spawn_choice_card(
                                                                parent,
                                                                assets,
                                                                localization,
                                                                lang,
                                                                name,
                                                                is_selected,
                                                                false, // is_perk
                                                                i,
                                                            );
                                                        }
                                                    });
                                            }
                                        });
                                });
                        });

                    // --- Bottom Footer Area with Confirm Button (fixed height to prevent shifting) ---
                    let confirm_label = if confirm_ready {
                        localization.get("general.confirm_level_up", lang)
                    } else {
                        localization.get("general.complete_selections", lang)
                    };

                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.),
                            height: Val::Px(70.), // Fixed height to prevent button shifting
                            justify_content: JustifyContent::Center,
                            margin: UiRect::bottom(Val::Px(48.)), // Move confirm button up more (from 32px to 48px)
                            ..default()
                        })
                        .with_children(|parent| {
                            // Tab button style for confirm button
                            let bg_color = if confirm_ready {
                                NORMAL_BUTTON_COLOR
                            } else {
                                Color::srgba(0.05, 0.09, 0.22, 0.5)
                            };

                            let mut c_btn = parent.spawn((
                                Node {
                                    align_self: AlignSelf::Center,
                                    padding: UiRect::axes(Val::Px(32.), Val::Px(10.)),
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                                BorderColor::all(BUTTON_BORDER_COLOR),
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                LevelUpConfirmBtn,
                            ));
                            if !confirm_ready {
                                c_btn.insert(DisabledButton);
                            }
                            c_btn
                                .observe(handle_level_up_confirm)
                                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                .observe(cursor::<Out>(SystemCursorIcon::Default))
                                .observe(handle_confirm_over)
                                .observe(handle_confirm_out)
                                .observe(handle_confirm_press)
                                .observe(handle_confirm_release)
                                .with_children(|parent| {
                                    let text_color = if confirm_ready {
                                        BUTTON_TEXT_COLOR
                                    } else {
                                        Color::srgba(0.6, 0.55, 0.4, 0.5)
                                    };
                                    parent.spawn((
                                        add_text(confirm_label.to_string(), "bold", 1.8, assets),
                                        TextColor(text_color),
                                    ));
                                });
                        });
                });
        });
}

fn spawn_choice_card(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    name: &str,
    is_selected: bool,
    is_ability: bool,
    index: usize,
) {
    const SELECTED_BORDER: Color = Color::srgb(1.0, 0.85, 0.2);
    const UNSELECTED_BORDER: Color = BUTTON_BORDER_COLOR;

    let border_col = if is_selected {
        SELECTED_BORDER
    } else {
        UNSELECTED_BORDER
    };
    let border_thickness = if is_selected {
        2.
    } else {
        1.
    };

    let img_name = name.to_string();

    let key_name = if is_ability {
        format!("ability.{}", name.replace(" ", "_").to_lowercase())
    } else {
        format!("perk.{}", name.replace(" ", "_").to_lowercase())
    };
    let raw_title = localization.get_opt(&key_name, lang).unwrap_or_else(|| name.to_string());
    let title = capitalize_words(&raw_title);

    let mut entity_cmd = parent.spawn((
        Node {
            width: percent(100.),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            flex_shrink: 0.,
            column_gap: Val::Px(8.),
            padding: UiRect::all(Val::Px(6.)),
            margin: UiRect::bottom(Val::Px(4.)), // Reduced margin from 6 to 4 to pack boxes closer
            border: UiRect::all(Val::Px(border_thickness)),
            ..default()
        },
        BackgroundColor(if is_selected {
            Color::srgba(0.20, 0.16, 0.04, 0.95)
        } else {
            BAR_BG_COLOR
        }),
        BorderColor::all(border_col),
        Button,
        Interaction::default(),
        Pickable::default(),
    ));

    if is_ability {
        entity_cmd.insert(LevelUpAbilityChoiceBtn(index));
        entity_cmd.observe(handle_ability_choice_click);
        entity_cmd.insert(RightColumnTooltip::Ability(name.to_string()));
    } else {
        entity_cmd.insert(LevelUpPerkChoiceBtn(index));
        entity_cmd.observe(handle_perk_choice_click);
        entity_cmd.insert(RightColumnTooltip::Perk(name.to_string()));
    }

    entity_cmd
        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
        .observe(cursor::<Out>(SystemCursorIcon::Default))
        .with_children(|parent| {
            // Icon placeholder
            parent.spawn((
                Node {
                    width: ICON_ITEM,
                    height: ICON_ITEM,
                    flex_shrink: 0.,
                    border: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                BackgroundColor(PLACEHOLDER_COLOR),
                BorderColor::all(BUTTON_BORDER_COLOR),
                ImageNode::new(assets.image(format!("build_{}", img_name)))
                    .with_mode(NodeImageMode::Stretch),
            ));

            // Text content
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    // Name (same as playing tab)
                    parent.spawn((
                        add_text(&title, "bold", 2.3, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));

                    // Description
                    let desc_text = if is_ability {
                        get_ability(name)
                            .map(|ab| ab.description(lang, &localization))
                            .unwrap_or_default()
                    } else {
                        get_perk(name)
                            .map(|pk| pk.description(lang, &localization))
                            .unwrap_or_default()
                    };

                    spawn_rich_text_row(parent, assets, desc_text, 2.0, "medium", Color::WHITE);
                });
        });
}
