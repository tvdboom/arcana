use crate::core::assets::WorldAssets;
use crate::core::localization::Localization;
use crate::core::menu::utils::add_text;
use crate::core::player::Player;
use crate::core::settings::{Language, Settings};
use crate::core::ui::utils::*;
use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

// ---------------------------------------------------------------------------
// WebAssembly stub: networking (and therefore duels) is not available on web.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub fn setup_duel_ui(
    mut commands: Commands,
    assets: Res<WorldAssets>,
    columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
    mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
) {
    for mut node in &mut columns_2_3_q {
        node.display = Display::None;
    }
    if let Some(container_entity) = columns_container_q.iter().next() {
        let panel_entity = spawn_panel_base(&mut commands, &assets, container_entity, "bg_duel");
        commands.entity(panel_entity).with_children(|parent| {
            parent.spawn((
                Node {
                    width: percent(100.),
                    height: percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(
                    add_text("Duels are not available on the web build.", "medium", 2.2, &assets),
                    TextColor(Color::WHITE),
                )],
            ));
        });
    }
}

// ---------------------------------------------------------------------------
// Native duel lobby UI
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use crate::core::audio::PlayAudioMsg;
    use crate::core::constants::*;
    use crate::core::network::{
        broadcast_lobby, is_valid_ip, local_ip, portrait_key, start_client, start_host,
        ClientMessage, ClientSendMsg, DuelPhase, DuelState, Ip, ServerSendMsg, MAX_BET_ITEMS,
    };

    /// Marker on the container whose children are rebuilt when the lobby changes.
    #[derive(Component)]
    pub struct DuelLobbyContent;

    /// The single host/connect button.
    #[derive(Component)]
    pub struct DuelActionBtn;

    /// A gold wager adjustment button carrying its delta.
    #[derive(Component)]
    pub struct DuelGoldBtn(pub i32);

    /// A toggle button for wagering one inventory item.
    #[derive(Component)]
    pub struct DuelItemBtn(pub String);

    /// The accept-wager toggle button.
    #[derive(Component)]
    pub struct DuelAcceptBtn;

    pub fn setup_duel_ui(
        mut commands: Commands,
        assets: Res<WorldAssets>,
        columns_container_q: Query<Entity, With<PlayScreenColumnsContainer>>,
        mut columns_2_3_q: Query<&mut Node, (With<PlayScreenColumns2And3>, Without<PanelCmp>)>,
    ) {
        for mut node in &mut columns_2_3_q {
            node.display = Display::None;
        }

        if let Some(container_entity) = columns_container_q.iter().next() {
            let panel_entity =
                spawn_panel_base(&mut commands, &assets, container_entity, "bg_duel");
            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn((
                    Node {
                        width: percent(100.),
                        height: percent(100.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        row_gap: Val::Px(14.),
                        padding: UiRect::all(percent(5.)),
                        ..default()
                    },
                    DuelLobbyContent,
                ));
            });
        }
    }

    /// Lets the player edit the IP address while in the connection phase.
    pub fn duel_ip_input(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut ip: ResMut<Ip>,
        duel: Option<Res<DuelState>>,
    ) {
        // The IP can only be edited before a session is created.
        if duel.is_some() {
            return;
        }

        for key in keyboard.get_just_pressed() {
            match key {
                KeyCode::Backspace => {
                    ip.pop();
                },
                KeyCode::Period | KeyCode::NumpadDecimal => ip.push('.'),
                KeyCode::Digit0 | KeyCode::Numpad0 => ip.push('0'),
                KeyCode::Digit1 | KeyCode::Numpad1 => ip.push('1'),
                KeyCode::Digit2 | KeyCode::Numpad2 => ip.push('2'),
                KeyCode::Digit3 | KeyCode::Numpad3 => ip.push('3'),
                KeyCode::Digit4 | KeyCode::Numpad4 => ip.push('4'),
                KeyCode::Digit5 | KeyCode::Numpad5 => ip.push('5'),
                KeyCode::Digit6 | KeyCode::Numpad6 => ip.push('6'),
                KeyCode::Digit7 | KeyCode::Numpad7 => ip.push('7'),
                KeyCode::Digit8 | KeyCode::Numpad8 => ip.push('8'),
                KeyCode::Digit9 | KeyCode::Numpad9 => ip.push('9'),
                _ => {},
            }
        }
    }

    /// Rebuilds the lobby content whenever a relevant value changes.
    #[allow(clippy::too_many_arguments)]
    pub fn refresh_duel_lobby(
        mut commands: Commands,
        assets: Res<WorldAssets>,
        localization: Res<Localization>,
        settings: Res<Settings>,
        player: Res<Player>,
        ip: Res<Ip>,
        duel: Option<Res<DuelState>>,
        content_q: Query<Entity, With<DuelLobbyContent>>,
        children_q: Query<&Children>,
        mut signature: Local<String>,
        mut local_ip_cache: Local<Option<String>>,
    ) {
        let Some(content) = content_q.iter().next() else {
            return;
        };

        let my_ip = local_ip_cache.get_or_insert_with(|| local_ip().to_string()).clone();

        let lang = settings.language;
        let sig = match &duel {
            None => format!("none|{my_ip}|{}", ip.as_str()),
            Some(d) => format!(
                "{:?}|{}|{}|{}|{}|{}|{}|{}|{}",
                d.phase,
                d.opponent.as_ref().map(|o| o.name.clone()).unwrap_or_default(),
                d.my_gold_bet,
                d.my_item_bet.join(","),
                d.opp_gold_bet,
                d.opp_item_bet.join(","),
                d.my_accept,
                d.opp_accept,
                player.gold,
            ),
        };

        if *signature == sig {
            return;
        }
        *signature = sig;

        despawn_descendants_manual(&mut commands, content, &children_q);

        match duel.as_deref() {
            None => build_connect_view(
                &mut commands,
                content,
                &assets,
                &localization,
                lang,
                ip.as_str(),
                &my_ip,
            ),
            Some(d) if d.phase == DuelPhase::Connecting => {
                build_waiting_view(&mut commands, content, &assets, &localization, lang)
            },
            Some(d) => build_betting_view(
                &mut commands,
                content,
                &assets,
                &localization,
                lang,
                &player,
                d,
            ),
        }
    }

    fn build_connect_view(
        commands: &mut Commands,
        content: Entity,
        assets: &WorldAssets,
        localization: &Localization,
        lang: Language,
        ip: &str,
        my_ip: &str,
    ) {
        let valid = is_valid_ip(ip);
        let is_host = valid && ip.trim() == my_ip;
        let label = localization.get(if is_host { "duel.host" } else { "duel.connect" }, lang);

        commands.entity(content).with_children(|parent| {
            parent.spawn((
                add_text(localization.get("duel.enter_ip", lang), "bold", 2.6, assets),
                TextColor(BUTTON_TEXT_COLOR),
            ));

            // IP textbox.
            parent.spawn((
                Node {
                    min_width: Val::Px(240.),
                    padding: UiRect::axes(Val::Px(18.), Val::Px(10.)),
                    border: UiRect::all(Val::Px(1.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON_COLOR),
                BorderColor::all(BUTTON_BORDER_COLOR),
                children![(
                    add_text(format!("{ip}_"), "medium", 2.4, assets),
                    TextColor(Color::WHITE),
                )],
            ));

            // Host / connect button (greyed out when the IP is invalid).
            let (bg, border) = if valid {
                (NORMAL_BUTTON_COLOR, BUTTON_BORDER_COLOR)
            } else {
                (DISABLED_BUTTON_COLOR, DISABLED_BORDER_COLOR)
            };
            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::Center,
                        padding: UiRect::axes(Val::Px(32.), Val::Px(10.)),
                        border: UiRect::all(Val::Px(1.)),
                        margin: UiRect::top(Val::Px(8.)),
                        ..default()
                    },
                    BackgroundColor(bg),
                    BorderColor::all(border),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    DuelActionBtn,
                    children![(
                        add_text(label, "bold", 2.0, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    )],
                ))
                .observe(on_action_click);
        });
    }

    fn build_waiting_view(
        commands: &mut Commands,
        content: Entity,
        assets: &WorldAssets,
        localization: &Localization,
        lang: Language,
    ) {
        commands.entity(content).with_children(|parent| {
            parent.spawn((
                add_text(localization.get("duel.waiting", lang), "medium", 2.4, assets),
                TextColor(Color::WHITE),
            ));
        });
    }

    fn build_betting_view(
        commands: &mut Commands,
        content: Entity,
        assets: &WorldAssets,
        localization: &Localization,
        lang: Language,
        player: &Player,
        duel: &DuelState,
    ) {
        let opponent_name =
            duel.opponent.as_ref().map(|o| o.name.clone()).unwrap_or_default();
        let opponent_img =
            duel.opponent.as_ref().map(portrait_key).unwrap_or_else(|| "unknown".to_string());

        commands.entity(content).with_children(|parent| {
            // Opponent portrait + name.
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Node { width: Val::Px(110.), height: Val::Px(110.), ..default() },
                        ImageNode::new(assets.image(&opponent_img))
                            .with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(opponent_name, "bold", 2.6, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                    parent.spawn((
                        add_text(
                            format!(
                                "{}: {} - {}",
                                localization.get("duel.their_wager", lang),
                                duel.opp_gold_bet,
                                duel.opp_item_bet.len(),
                            ),
                            "medium",
                            1.8,
                            assets,
                        ),
                        TextColor(Color::WHITE),
                    ));
                    if duel.opp_accept {
                        parent.spawn((
                            add_text(localization.get("duel.accepted", lang), "bold", 1.8, assets),
                            TextColor(Color::srgb_u8(120, 200, 120)),
                        ));
                    }
                });

            // Gold wager row.
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_step_button(parent, assets, "-100", DuelGoldBtn(-100));
                    spawn_step_button(parent, assets, "-10", DuelGoldBtn(-10));
                    parent.spawn((
                        Node { width: Val::Px(28.), height: Val::Px(28.), ..default() },
                        ImageNode::new(assets.image("gold")).with_mode(NodeImageMode::Stretch),
                    ));
                    parent.spawn((
                        add_text(duel.my_gold_bet.to_string(), "bold", 2.4, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    ));
                    spawn_step_button(parent, assets, "+10", DuelGoldBtn(10));
                    spawn_step_button(parent, assets, "+100", DuelGoldBtn(100));
                });

            // Item wager (clickable inventory, max MAX_BET_ITEMS).
            parent.spawn((
                add_text(
                    format!(
                        "{} ({}/{})",
                        localization.get("duel.wager_items", lang),
                        duel.my_item_bet.len(),
                        MAX_BET_ITEMS,
                    ),
                    "bold",
                    2.0,
                    assets,
                ),
                TextColor(BUTTON_TEXT_COLOR),
            ));
            parent
                .spawn(Node {
                    width: percent(90.),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(6.),
                    row_gap: Val::Px(6.),
                    ..default()
                })
                .with_children(|parent| {
                    for key in &player.inventory {
                        let selected = duel.my_item_bet.contains(key);
                        let border = if selected {
                            Color::srgb_u8(120, 200, 120)
                        } else {
                            BUTTON_BORDER_COLOR
                        };
                        parent
                            .spawn((
                                Node {
                                    width: Val::Px(48.),
                                    height: Val::Px(48.),
                                    border: UiRect::all(Val::Px(2.)),
                                    ..default()
                                },
                                BorderColor::all(border),
                                ImageNode::new(assets.image(format!("build_{key}")))
                                    .with_mode(NodeImageMode::Stretch),
                                Button,
                                Interaction::default(),
                                Pickable::default(),
                                DuelItemBtn(key.clone()),
                            ))
                            .observe(on_item_click);
                    }
                });

            // Accept button.
            let accept_label = if duel.my_accept {
                localization.get("duel.cancel", lang)
            } else {
                localization.get("duel.accept", lang)
            };
            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::Center,
                        padding: UiRect::axes(Val::Px(32.), Val::Px(10.)),
                        border: UiRect::all(Val::Px(1.)),
                        margin: UiRect::top(Val::Px(8.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    DuelAcceptBtn,
                    children![(
                        add_text(accept_label, "bold", 2.0, assets),
                        TextColor(BUTTON_TEXT_COLOR),
                    )],
                ))
                .observe(on_accept_click);
        });
    }

    fn spawn_step_button(
        parent: &mut ChildSpawnerCommands,
        assets: &WorldAssets,
        label: &str,
        marker: DuelGoldBtn,
    ) {
        parent
            .spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.), Val::Px(6.)),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON_COLOR),
                BorderColor::all(BUTTON_BORDER_COLOR),
                Button,
                Interaction::default(),
                Pickable::default(),
                marker,
                children![(add_text(label, "bold", 1.6, assets), TextColor(BUTTON_TEXT_COLOR))],
            ))
            .observe(on_gold_click);
    }

    // -----------------------------------------------------------------------
    // Click handling
    // -----------------------------------------------------------------------

    fn on_action_click(
        _event: On<Pointer<Click>>,
        mut commands: Commands,
        ip: Res<Ip>,
        duel: Option<Res<DuelState>>,
        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    ) {
        if duel.is_some() {
            return;
        }
        let ip_str = ip.trim().to_string();
        if !is_valid_ip(&ip_str) {
            return;
        }
        play_audio_msg.write(PlayAudioMsg::new("button"));
        if ip_str == local_ip().to_string() {
            start_host(&mut commands);
        } else {
            start_client(&mut commands, &ip_str);
        }
    }

    fn on_gold_click(
        event: On<Pointer<Click>>,
        btn_q: Query<&DuelGoldBtn>,
        player: Res<Player>,
        duel: Option<ResMut<DuelState>>,
        play_audio_msg: MessageWriter<PlayAudioMsg>,
        server_send: MessageWriter<ServerSendMsg>,
        client_send: MessageWriter<ClientSendMsg>,
    ) {
        let Ok(btn) = btn_q.get(event.entity) else {
            return;
        };
        let delta = btn.0;
        let Some(mut duel) = duel else {
            return;
        };
        if duel.phase != DuelPhase::Betting {
            return;
        }
        let new = (duel.my_gold_bet as i32 + delta).clamp(0, player.gold as i32) as u32;
        if new == duel.my_gold_bet {
            return;
        }
        duel.my_gold_bet = new;
        push_my_bet(&mut duel, play_audio_msg, server_send, client_send);
    }

    fn on_item_click(
        event: On<Pointer<Click>>,
        btn_q: Query<&DuelItemBtn>,
        duel: Option<ResMut<DuelState>>,
        play_audio_msg: MessageWriter<PlayAudioMsg>,
        server_send: MessageWriter<ServerSendMsg>,
        client_send: MessageWriter<ClientSendMsg>,
    ) {
        let Ok(btn) = btn_q.get(event.entity) else {
            return;
        };
        let key = btn.0.clone();
        let Some(mut duel) = duel else {
            return;
        };
        if duel.phase != DuelPhase::Betting {
            return;
        }
        if let Some(pos) = duel.my_item_bet.iter().position(|k| *k == key) {
            duel.my_item_bet.remove(pos);
        } else if duel.my_item_bet.len() < MAX_BET_ITEMS {
            duel.my_item_bet.push(key);
        } else {
            return;
        }
        push_my_bet(&mut duel, play_audio_msg, server_send, client_send);
    }

    fn on_accept_click(
        _event: On<Pointer<Click>>,
        duel: Option<ResMut<DuelState>>,
        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
        mut server_send: MessageWriter<ServerSendMsg>,
        mut client_send: MessageWriter<ClientSendMsg>,
    ) {
        let Some(mut duel) = duel else {
            return;
        };
        if duel.phase != DuelPhase::Betting {
            return;
        }
        duel.my_accept = !duel.my_accept;
        play_audio_msg.write(PlayAudioMsg::new("button"));
        if duel.is_host() {
            broadcast_lobby(&duel, &mut server_send);
        } else {
            client_send.write(ClientSendMsg::new(ClientMessage::Accept(duel.my_accept)));
        }
    }

    /// Propagate the local wager change to the peer (resetting accept flags).
    fn push_my_bet(
        duel: &mut DuelState,
        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
        mut server_send: MessageWriter<ServerSendMsg>,
        mut client_send: MessageWriter<ClientSendMsg>,
    ) {
        duel.my_accept = false;
        play_audio_msg.write(PlayAudioMsg::new("button"));
        if duel.is_host() {
            duel.opp_accept = false;
            broadcast_lobby(duel, &mut server_send);
        } else {
            client_send.write(ClientSendMsg::new(ClientMessage::SetBet {
                gold: duel.my_gold_bet,
                items: duel.my_item_bet.clone(),
            }));
        }
    }
}
