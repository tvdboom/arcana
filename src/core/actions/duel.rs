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
            broadcast_lobby, is_valid_ip, local_ip, portrait_key, start_client, start_host, teardown_duel,
            ClientMessage, ClientSendMsg, DuelPhase, DuelRole, DuelState, Ip, ServerSendMsg, MAX_BET_ITEMS,
    };

    /// Marker on the container whose children are rebuilt when the lobby changes.
    #[derive(Component)]
    pub struct DuelLobbyContent;

    /// The host button.
    #[derive(Component)]
    pub struct DuelHostBtn;

    /// The connect button.
    #[derive(Component)]
    pub struct DuelConnectBtn;

    /// The cancel hosting button.
    #[derive(Component)]
    pub struct DuelCancelHostBtn;

    /// A gold wager adjustment button carrying its delta.
    #[derive(Component)]
    pub struct DuelGoldBtn(pub i32);

    /// A toggle button for wagering one inventory item.
    #[derive(Component)]
    pub struct DuelItemBtn(pub usize);

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
                        padding: UiRect {
                            left: percent(5.),
                            right: percent(5.),
                            top: percent(14.),
                            bottom: percent(5.),
                        },
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

        const MAX_IP_LENGTH: usize = 15; // Maximum length for IPv4 (255.255.255.255)

        for key in keyboard.get_just_pressed() {
            match key {
                KeyCode::Backspace => {
                    ip.pop();
                },
                KeyCode::Space => {
                    ip.0 = local_ip().to_string();
                },
                KeyCode::Period | KeyCode::NumpadDecimal => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('.')
                    }
                },
                KeyCode::Digit0 | KeyCode::Numpad0 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('0')
                    }
                },
                KeyCode::Digit1 | KeyCode::Numpad1 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('1')
                    }
                },
                KeyCode::Digit2 | KeyCode::Numpad2 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('2')
                    }
                },
                KeyCode::Digit3 | KeyCode::Numpad3 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('3')
                    }
                },
                KeyCode::Digit4 | KeyCode::Numpad4 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('4')
                    }
                },
                KeyCode::Digit5 | KeyCode::Numpad5 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('5')
                    }
                },
                KeyCode::Digit6 | KeyCode::Numpad6 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('6')
                    }
                },
                KeyCode::Digit7 | KeyCode::Numpad7 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('7')
                    }
                },
                KeyCode::Digit8 | KeyCode::Numpad8 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('8')
                    }
                },
                KeyCode::Digit9 | KeyCode::Numpad9 => {
                    if ip.len() < MAX_IP_LENGTH {
                        ip.push('9')
                    }
                },
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
                "{:?}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                d.phase,
                d.opponent.as_ref().map(|o| o.name.clone()).unwrap_or_default(),
                d.my_gold_bet,
                d.my_item_bet.join(","),
                d.opp_gold_bet,
                d.opp_item_bet.join(","),
                d.my_accept,
                d.opp_accept,
                player.gold,
                player.inventory.join(","),
                player.equipped_consumables.join(","),
            ),
        };

        let content_empty =
            children_q.get(content).map(|children| children.is_empty()).unwrap_or(true);
        if *signature == sig && !content_empty {
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
                build_waiting_view(&mut commands, content, &assets, &localization, lang, d)
            },
            Some(d) if d.opponent.is_some() => {
                build_betting_view(&mut commands, content, &assets, &localization, lang, &player, d)
            },
            Some(_) => {
                build_waiting_view(&mut commands, content, &assets, &localization, lang, &duel.unwrap())
            },
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
        use crate::core::menu::utils::recolor;
        use crate::core::utils::cursor;
        use bevy::window::SystemCursorIcon;

        let valid = is_valid_ip(ip);
        let is_host_ip = valid && ip.trim() == my_ip;

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
                    add_text(ip.to_string(), "medium", 2.4, assets),
                    TextColor(Color::WHITE),
                )],
            ));

            // Host and Connect buttons container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::top(Val::Px(12.)),
                    ..default()
                })
                .with_children(|parent| {
                    // Host button (disabled when IP is not the same as local IP)
                    let host_disabled = !is_host_ip;
                    let (host_bg, host_border) = if is_host_ip {
                        (NORMAL_BUTTON_COLOR, BUTTON_BORDER_COLOR)
                    } else {
                        (DISABLED_BUTTON_COLOR, DISABLED_BORDER_COLOR)
                    };
                    let mut host_btn = parent.spawn((
                        Node {
                            align_self: AlignSelf::Center,
                            padding: UiRect::axes(Val::Px(32.), Val::Px(10.)),
                            border: UiRect::all(Val::Px(1.)),
                            ..default()
                        },
                        BackgroundColor(host_bg),
                        BorderColor::all(host_border),
                        Button,
                        Interaction::default(),
                        Pickable::default(),
                        DuelHostBtn,
                        children![(
                            add_text(
                                localization.get("duel.host", lang),
                                "bold",
                                2.0,
                                assets
                            ),
                            TextColor(BUTTON_TEXT_COLOR),
                        )],
                    ));
                    if host_disabled {
                        host_btn.insert(crate::core::menu::buttons::DisabledButton);
                    }
                    host_btn
                        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(on_host_click);

                    // Connect button (enabled for all valid IPs)
                    let connect_disabled = !valid;
                    let (connect_bg, connect_border) = if valid {
                        (NORMAL_BUTTON_COLOR, BUTTON_BORDER_COLOR)
                    } else {
                        (DISABLED_BUTTON_COLOR, DISABLED_BORDER_COLOR)
                    };
                    let mut connect_btn = parent.spawn((
                        Node {
                            align_self: AlignSelf::Center,
                            padding: UiRect::axes(Val::Px(32.), Val::Px(10.)),
                            border: UiRect::all(Val::Px(1.)),
                            ..default()
                        },
                        BackgroundColor(connect_bg),
                        BorderColor::all(connect_border),
                        Button,
                        Interaction::default(),
                        Pickable::default(),
                        DuelConnectBtn,
                        children![(
                            add_text(
                                localization.get("duel.connect", lang),
                                "bold",
                                2.0,
                                assets
                            ),
                            TextColor(BUTTON_TEXT_COLOR),
                        )],
                    ));
                    if connect_disabled {
                        connect_btn.insert(crate::core::menu::buttons::DisabledButton);
                    }
                    connect_btn
                        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(on_connect_click);
                });
        });
    }

    fn build_waiting_view(
        commands: &mut Commands,
        content: Entity,
        assets: &WorldAssets,
        localization: &Localization,
        lang: Language,
        duel: &DuelState,
    ) {
        use crate::core::menu::utils::recolor;
        use crate::core::utils::cursor;
        use bevy::window::SystemCursorIcon;

        commands.entity(content).with_children(|parent| {
            let waiting_text = if duel.role == DuelRole::Host {
                localization.get("duel.waiting", lang)
            } else {
                localization.get("duel.waiting_for_host", lang)
            };
            parent.spawn((
                add_text(waiting_text, "medium", 2.4, assets),
                TextColor(Color::WHITE),
            ));

            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::Center,
                        padding: UiRect::axes(Val::Px(32.), Val::Px(10.)),
                        border: UiRect::all(Val::Px(1.)),
                        margin: UiRect::top(Val::Px(12.)),
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON_COLOR),
                    BorderColor::all(BUTTON_BORDER_COLOR),
                    Button,
                    Interaction::default(),
                    Pickable::default(),
                    DuelCancelHostBtn,
                    children![(
                        add_text(
                            localization.get("duel.cancel", lang),
                            "bold",
                            2.0,
                            assets
                        ),
                        TextColor(BUTTON_TEXT_COLOR),
                    )],
                ))
                .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .observe(on_cancel_host_click);
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
        use crate::core::menu::utils::recolor;
        use crate::core::utils::cursor;
        use bevy::window::SystemCursorIcon;

        let opponent_name = duel.opponent.as_ref().map(|o| o.name.clone()).unwrap_or_default();
        let opponent_img =
            duel.opponent.as_ref().map(portrait_key).unwrap_or_else(|| "unknown".to_string());

        commands.entity(content).with_children(|parent| {
            // Main Row layout to split Left (Betting info & options) and Right (Opponent image)
            parent.spawn(Node {
                width: percent(100.),
                height: percent(100.),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(20.),
                ..default()
            }).with_children(|row_parent| {
                // LEFT SIDE: Betting info and options (wagers, Accept button, etc.)
                row_parent.spawn(Node {
                    flex_grow: 1.0,
                    flex_basis: percent(55.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    row_gap: Val::Px(12.),
                    ..default()
                }).with_children(|left_parent| {
                    // 1. Opponent's wager info
                    left_parent.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.),
                        ..default()
                    }).with_children(|opp_wager_parent| {
                        opp_wager_parent.spawn((
                            add_text(localization.get("duel.their_wager", lang), "bold", 2.0, assets),
                            TextColor(BUTTON_TEXT_COLOR),
                        ));
                        
                        // Opponent Gold
                        opp_wager_parent.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.),
                            ..default()
                        }).with_children(|gold_row| {
                            gold_row.spawn((
                                Node {
                                    width: Val::Px(24.),
                                    height: Val::Px(24.),
                                    ..default()
                                },
                                ImageNode::new(assets.image("gold")).with_mode(NodeImageMode::Stretch),
                            ));
                            gold_row.spawn((
                                add_text(duel.opp_gold_bet.to_string(), "bold", 2.0, assets),
                                TextColor(Color::WHITE),
                            ));
                        });

                        // Opponent Items
                        opp_wager_parent.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(6.),
                            margin: UiRect::top(Val::Px(4.)),
                            ..default()
                        }).with_children(|items_row| {
                            if duel.opp_item_bet.is_empty() {
                                items_row.spawn((
                                    add_text("No items", "medium", 1.6, assets),
                                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                                ));
                            } else {
                                for key in &duel.opp_item_bet {
                                    items_row.spawn((
                                        Node {
                                            width: Val::Px(40.),
                                            height: Val::Px(40.),
                                            border: UiRect::all(Val::Px(1.)),
                                            ..default()
                                        },
                                        BorderColor::all(BUTTON_BORDER_COLOR),
                                        ImageNode::new(assets.image(format!("build_{key}")))
                                            .with_mode(NodeImageMode::Stretch),
                                    ));
                                }
                            }
                        });

                        if duel.opp_accept {
                            opp_wager_parent.spawn((
                                add_text(localization.get("duel.accepted", lang), "bold", 1.8, assets),
                                TextColor(Color::srgb_u8(120, 200, 120)),
                            ));
                        }
                    });

                    // 2. My Gold Wager row
                    left_parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.),
                            ..default()
                        })
                        .with_children(|gold_btn_row| {
                            spawn_step_button(gold_btn_row, assets, "-100", DuelGoldBtn(-100));
                            spawn_step_button(gold_btn_row, assets, "-10", DuelGoldBtn(-10));
                            gold_btn_row.spawn((
                                Node {
                                    width: Val::Px(28.),
                                    height: Val::Px(28.),
                                    ..default()
                                },
                                ImageNode::new(assets.image("gold")).with_mode(NodeImageMode::Stretch),
                            ));
                            gold_btn_row.spawn((
                                add_text(duel.my_gold_bet.to_string(), "bold", 2.4, assets),
                                TextColor(BUTTON_TEXT_COLOR),
                            ));
                            spawn_step_button(gold_btn_row, assets, "+10", DuelGoldBtn(10));
                            spawn_step_button(gold_btn_row, assets, "+100", DuelGoldBtn(100));
                        });

                    // 3. Item wager (clickable unequipped items, max MAX_BET_ITEMS)
                    left_parent.spawn((
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

                    // Filter out equipped items from player's inventory
                    let mut unequipped_items = Vec::new();
                    let mut temp_equipped = player.equipped_consumables.clone();
                    for key in &player.inventory {
                        if let Some(pos) = temp_equipped.iter().position(|eq| eq == key) {
                            temp_equipped.remove(pos);
                        } else {
                            unequipped_items.push(key.clone());
                        }
                    }

                    // Build selected indices to highlight them correctly (avoiding duplicate highlighting bugs)
                    let mut temp_selected_indices = Vec::new();
                    for key in &duel.my_item_bet {
                        let mut found_idx = None;
                        for (idx, item_key) in unequipped_items.iter().enumerate() {
                            if item_key == key && !temp_selected_indices.contains(&idx) {
                                found_idx = Some(idx);
                                break;
                            }
                        }
                        if let Some(idx) = found_idx {
                            temp_selected_indices.push(idx);
                        }
                    }

                    left_parent
                        .spawn(Node {
                            width: percent(100.),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(6.),
                            row_gap: Val::Px(6.),
                            ..default()
                        })
                        .with_children(|items_container| {
                            for (i, key) in unequipped_items.iter().enumerate() {
                                let selected = temp_selected_indices.contains(&i);
                                let border = if selected {
                                    Color::srgb_u8(120, 200, 120)
                                } else {
                                    BUTTON_BORDER_COLOR
                                };
                                items_container
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
                                        DuelItemBtn(i),
                                    ))
                                    .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                                    .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                                    .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                                    .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                                    .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                                    .observe(cursor::<Out>(SystemCursorIcon::Default))
                                    .observe(on_item_click);
                            }
                        });

                    // 4. Accept button.
                    let accept_label = if duel.my_accept {
                        localization.get("duel.cancel", lang)
                    } else {
                        localization.get("duel.accept", lang)
                    };
                    left_parent
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
                        .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
                        .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
                        .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
                        .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
                        .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                        .observe(cursor::<Out>(SystemCursorIcon::Default))
                        .observe(on_accept_click);
                });

                // RIGHT SIDE: Large Opponent Portrait (height of panel minus padding, keeps 1.0 aspect ratio)
                row_parent.spawn((
                    Node {
                        height: percent(100.),
                        aspect_ratio: Some(1.0),
                        align_self: AlignSelf::Center,
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BorderColor::all(BUTTON_BORDER_COLOR),
                ))
                .with_children(|right_parent| {
                    right_parent.spawn((
                        Node {
                            width: percent(100.),
                            height: percent(100.),
                            ..default()
                        },
                        ImageNode::new(assets.image(&opponent_img))
                            .with_mode(NodeImageMode::Stretch),
                    ));

                    // Transparent overlay with name and level on top left (no darker background)
                    let opponent_level = duel.opponent.as_ref().map(|o| o.level()).unwrap_or(1);
                    right_parent.spawn(
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(10.),
                            top: Val::Px(10.),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(6.)),
                            ..default()
                        },
                    ).with_children(|overlay| {
                        overlay.spawn((
                            add_text(opponent_name, "bold", 2.2, assets),
                            TextColor(Color::WHITE),
                        ));
                        overlay.spawn((
                            add_text(format!("Lv. {}", opponent_level), "medium", 1.6, assets),
                            TextColor(Color::srgb_u8(240, 200, 80)),
                        ));
                    });
                });
            });
        });
    }

    fn spawn_step_button(
        parent: &mut ChildSpawnerCommands,
        assets: &WorldAssets,
        label: &str,
        marker: DuelGoldBtn,
    ) {
        use crate::core::menu::utils::recolor;
        use crate::core::utils::cursor;
        use bevy::window::SystemCursorIcon;

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
            .observe(recolor::<Over>(HOVERED_BUTTON_COLOR))
            .observe(recolor::<Out>(NORMAL_BUTTON_COLOR))
            .observe(recolor::<Press>(PRESSED_BUTTON_COLOR))
            .observe(recolor::<Release>(HOVERED_BUTTON_COLOR))
            .observe(cursor::<Over>(SystemCursorIcon::Pointer))
            .observe(cursor::<Out>(SystemCursorIcon::Default))
            .observe(on_gold_click);
    }

    // -----------------------------------------------------------------------
    // Click handling
    // -----------------------------------------------------------------------

    fn on_host_click(
        _event: On<Pointer<Click>>,
        mut commands: Commands,
        ip: Res<Ip>,
        duel: Option<Res<DuelState>>,
        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
        window_e: Single<Entity, With<Window>>,
    ) {
        if duel.is_some() {
            return;
        }
        let ip_str = ip.trim().to_string();
        if !is_valid_ip(&ip_str) {
            return;
        }
        if ip_str != local_ip().to_string() {
            return;
        }
        play_audio_msg.write(PlayAudioMsg::new("button"));
        start_host(&mut commands);
        commands.entity(*window_e).insert(bevy::window::CursorIcon::from(bevy::window::SystemCursorIcon::Default));
    }

    fn on_connect_click(
        _event: On<Pointer<Click>>,
        mut commands: Commands,
        ip: Res<Ip>,
        duel: Option<Res<DuelState>>,
        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
        window_e: Single<Entity, With<Window>>,
    ) {
        if duel.is_some() {
            return;
        }
        let ip_str = ip.trim().to_string();
        if !is_valid_ip(&ip_str) {
            return;
        }
        play_audio_msg.write(PlayAudioMsg::new("button"));
        start_client(&mut commands, &ip_str);
        commands.entity(*window_e).insert(bevy::window::CursorIcon::from(bevy::window::SystemCursorIcon::Default));
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
        player: Res<Player>,
        duel: Option<ResMut<DuelState>>,
        play_audio_msg: MessageWriter<PlayAudioMsg>,
        server_send: MessageWriter<ServerSendMsg>,
        client_send: MessageWriter<ClientSendMsg>,
    ) {
        let Ok(btn) = btn_q.get(event.entity) else {
            return;
        };
        let clicked_idx = btn.0;
        let Some(mut duel) = duel else {
            return;
        };
        if duel.phase != DuelPhase::Betting {
            return;
        }

        // Reconstruct unequipped_items list
        let mut unequipped_items = Vec::new();
        let mut temp_equipped = player.equipped_consumables.clone();
        for key in &player.inventory {
            if let Some(pos) = temp_equipped.iter().position(|eq| eq == key) {
                temp_equipped.remove(pos);
            } else {
                unequipped_items.push(key.clone());
            }
        }

        if clicked_idx >= unequipped_items.len() {
            return;
        }
        let clicked_key = &unequipped_items[clicked_idx];

        // Determine currently selected indices
        let mut temp_selected_indices = Vec::new();
        for key in &duel.my_item_bet {
            let mut found_idx = None;
            for (idx, item_key) in unequipped_items.iter().enumerate() {
                if item_key == key && !temp_selected_indices.contains(&idx) {
                    found_idx = Some(idx);
                    break;
                }
            }
            if let Some(idx) = found_idx {
                temp_selected_indices.push(idx);
            }
        }

        if temp_selected_indices.contains(&clicked_idx) {
            // Already selected, so remove one occurrence of this key
            if let Some(pos) = duel.my_item_bet.iter().position(|k| k == clicked_key) {
                duel.my_item_bet.remove(pos);
            }
        } else if duel.my_item_bet.len() < MAX_BET_ITEMS {
            // Add key
            duel.my_item_bet.push(clicked_key.clone());
        } else {
            return;
        }

        push_my_bet(&mut duel, play_audio_msg, server_send, client_send);
    }

    fn on_cancel_host_click(
        _event: On<Pointer<Click>>,
        mut commands: Commands,
        duel: Option<Res<DuelState>>,
        mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    ) {
        if duel.is_none() {
            return;
        }
        play_audio_msg.write(PlayAudioMsg::new("button"));
        teardown_duel(&mut commands);
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
