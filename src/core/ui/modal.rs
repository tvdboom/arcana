use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};

use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::*;
use crate::core::localization::Localization;
use crate::core::menu::utils::{add_root_node, add_text};
use crate::core::player::Player;
use crate::core::settings::Language;
use crate::core::ui::button::spawn_action_button;
use crate::core::ui::playing::{unequip_item, EquipSlot};
use crate::core::utils::cursor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalAction {
    SellItem {
        key: String,
        price: u32,
        is_equipped: bool,
        slot: Option<EquipSlot>,
    },
    RemovePerk {
        perk_name: String,
    },
}

#[derive(Resource, Default)]
pub struct ActiveModal {
    pub active: bool,
    pub action: Option<ModalAction>,
}

#[derive(Component)]
pub struct ModalOverlay;

pub fn spawn_modal(
    commands: &mut Commands,
    assets: &WorldAssets,
    localization: &Localization,
    lang: Language,
    action: ModalAction,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
) {
    // Play button sound on open
    play_audio_msg.write(PlayAudioMsg::new("button"));

    // Set resource state
    commands.insert_resource(ActiveModal {
        active: true,
        action: Some(action.clone()),
    });

    // Translate texts
    let (title_key, text_key) = match action {
        ModalAction::SellItem {
            ..
        } => ("modal.sell_item_title", "modal.sell_item_text"),
        ModalAction::RemovePerk {
            ..
        } => ("modal.remove_perk_title", "modal.remove_perk_text"),
    };

    let title_text = localization.get(title_key, lang);
    let body_text = localization.get(text_key, lang);
    let ok_text = localization.get("general.ok", lang);
    let cancel_text = localization.get("general.cancel", lang);

    let (root_node, _) = add_root_node(false);
    commands
        .spawn((
            root_node,
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            GlobalZIndex(2000),
            ModalOverlay,
            Button,
            Interaction::default(),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ))
        .observe(handle_modal_overlay_click)
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Vh(85.0),
                        height: Val::Vh(40.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Vh(5.5),
                        padding: UiRect {
                            left: Val::Vh(6.0),
                            right: Val::Vh(6.0),
                            top: Val::Vh(6.0),
                            bottom: Val::Vh(6.0),
                        },
                        ..default()
                    },
                    ImageNode::new(assets.image("banner_large")).with_mode(NodeImageMode::Stretch),
                    Interaction::default(),
                    Pickable::default(),
                ))
                .observe(|mut ev: On<Pointer<Click>>| {
                    ev.propagate(false);
                })
                .with_children(|parent| {
                    // Top Column for Title and Description (keeps them stacked closely)
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Vh(1.5),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Title
                            parent.spawn((
                                add_text(title_text, "bold", 3.0, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                            ));

                            // Description
                            parent.spawn((
                                add_text(body_text, "medium", 2.2, assets),
                                TextColor(Color::WHITE),
                                Node {
                                    max_width: Val::Vh(72.0),
                                    ..default()
                                },
                            ));
                        });

                    // Buttons Row
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Vh(4.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            // OK button
                            spawn_action_button(parent, assets, ok_text)
                                .observe(handle_modal_ok_click)
                                .observe(cursor::<Release>(SystemCursorIcon::Default));

                            // Cancel Button
                            spawn_action_button(parent, assets, cancel_text)
                                .observe(handle_modal_cancel_click)
                                .observe(cursor::<Release>(SystemCursorIcon::Default));
                        });
                });
        });
}

fn confirm_modal_action(
    commands: &mut Commands,
    active_modal: &mut ActiveModal,
    player: &mut Player,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    overlay_q: &Query<Entity, With<ModalOverlay>>,
    window_entity: Option<Entity>,
) {
    if let Some(action) = active_modal.action.take() {
        match action {
            ModalAction::SellItem {
                key,
                price,
                is_equipped,
                slot,
            } => {
                if is_equipped {
                    if let Some(s) = slot {
                        match s {
                            EquipSlot::Helmet => player.helmet = None,
                            EquipSlot::Accessory => player.accessory = None,
                            EquipSlot::Accessory2 => player.accessory2 = None,
                            EquipSlot::WeaponLH => player.weapon_lh = None,
                            EquipSlot::WeaponRH => player.weapon_rh = None,
                            EquipSlot::Chestplate => player.armor = None,
                            EquipSlot::Boots => player.boots = None,
                            EquipSlot::Gloves => player.gloves = None,
                        }
                    } else {
                        unequip_item(player, &key);
                    }
                }
                if let Some(pos) = player.inventory.iter().position(|k| k == &key) {
                    player.inventory.remove(pos);
                }
                player.gold += price;
                play_audio_msg.write(PlayAudioMsg::new("sell"));
                if let Some(win_e) = window_entity {
                    commands.entity(win_e).insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
            ModalAction::RemovePerk {
                perk_name,
            } => {
                if let Some(pos) = player.perks.iter().position(|p| p == &perk_name) {
                    player.perks.remove(pos);
                }
                play_audio_msg.write(PlayAudioMsg::new("poof"));
            },
        }
    }

    active_modal.active = false;
    for entity in overlay_q.iter() {
        commands.entity(entity).despawn();
    }
}

fn cancel_modal_action(
    commands: &mut Commands,
    active_modal: &mut ActiveModal,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    overlay_q: &Query<Entity, With<ModalOverlay>>,
) {
    play_audio_msg.write(PlayAudioMsg::new("button"));
    active_modal.active = false;
    active_modal.action = None;
    for entity in overlay_q.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn handle_modal_ok_click(
    _event: On<Pointer<Click>>,
    mut commands: Commands,
    overlay_q: Query<Entity, With<ModalOverlay>>,
    mut active_modal: ResMut<ActiveModal>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    window_e: Single<Entity, With<Window>>,
) {
    confirm_modal_action(
        &mut commands,
        &mut active_modal,
        &mut player,
        &mut play_audio_msg,
        &overlay_q,
        Some(*window_e),
    );
}

pub fn handle_modal_cancel_click(
    _event: On<Pointer<Click>>,
    mut commands: Commands,
    overlay_q: Query<Entity, With<ModalOverlay>>,
    mut active_modal: ResMut<ActiveModal>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    cancel_modal_action(&mut commands, &mut active_modal, &mut play_audio_msg, &overlay_q);
}

pub fn handle_modal_overlay_click(
    _event: On<Pointer<Click>>,
    mut commands: Commands,
    overlay_q: Query<Entity, With<ModalOverlay>>,
    mut active_modal: ResMut<ActiveModal>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    cancel_modal_action(&mut commands, &mut active_modal, &mut play_audio_msg, &overlay_q);
}

pub fn modal_input_system(
    mut commands: Commands,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    overlay_q: Query<Entity, With<ModalOverlay>>,
    mut active_modal: ResMut<ActiveModal>,
    mut player: ResMut<Player>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    window_e: Single<Entity, With<Window>>,
) {
    if !active_modal.active {
        return;
    }

    if keyboard.just_released(KeyCode::Enter) {
        keyboard.reset(KeyCode::Enter);
        confirm_modal_action(
            &mut commands,
            &mut active_modal,
            &mut player,
            &mut play_audio_msg,
            &overlay_q,
            Some(*window_e),
        );
    } else if keyboard.just_released(KeyCode::Escape) {
        keyboard.reset(KeyCode::Escape);
        cancel_modal_action(&mut commands, &mut active_modal, &mut play_audio_msg, &overlay_q);
    }
}
