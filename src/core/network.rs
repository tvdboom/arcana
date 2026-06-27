//! Peer-to-peer duel networking (host/client) built on `bevy_renet`.
//!
//! This module is compiled only for native targets; WebAssembly builds cannot
//! open UDP sockets and therefore never see any of the duel networking.
//!
//! The host acts as the authoritative server: it owns the combat simulation,
//! resolves the betting handshake and streams snapshots to the joining client.

use std::net::{IpAddr, UdpSocket};
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_renet::netcode::*;
use bevy_renet::renet::{ConnectionConfig, DefaultChannel, ServerEvent};
use bevy_renet::*;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};

use crate::core::actions::gain_xp;
use crate::core::audio::PlayAudioMsg;
use crate::core::catalog::equipment::Equipment;
use crate::core::classes::Class;
use crate::core::combat::mechanics::{
    consumable_card_order, enemy_cast_ability, enemy_use_consumable, step_combat, try_cast_ability,
    try_use_consumable, CombatCard, CombatFx, CombatSpeed, CombatState, CombatStatus, Fighter,
    FxSide, ABILITY_HOTKEYS, CONSUMABLE_HOTKEYS,
};
use crate::core::monsters::{ActiveMonster, Monster, MonsterKind};
use crate::core::player::Player;
use crate::core::states::GameState;
use crate::core::ui::creation::SelectionItem;
use crate::core::ui::level_up::LevelUpPending;

const PROTOCOL_ID: u64 = 0xA2C4_0DEF_0001; // arbitrary but stable
const DUEL_PORT: u16 = 5001;

pub type ClientId = u64;

// ---------------------------------------------------------------------------
// Roles, phases and the shared duel state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuelRole {
    Host,
    Client,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DuelPhase {
    /// Lobby is open but no peer has connected yet.
    Connecting,
    /// Both peers connected; players are placing their wagers.
    Betting,
    /// Combat is running.
    Combat,
    /// Combat is over; result is shown.
    Result,
}

/// Maximum number of items that can be wagered by a single player.
pub const MAX_BET_ITEMS: usize = 3;

/// Live state of an ongoing duel. Present only while a duel session exists.
#[derive(Resource)]
pub struct DuelState {
    pub role: DuelRole,
    pub phase: DuelPhase,
    /// Full profile of the opponent (used for portrait, name and combat stats).
    pub opponent: Option<Player>,
    pub my_gold_bet: u32,
    pub my_item_bet: Vec<String>,
    pub opp_gold_bet: u32,
    pub opp_item_bet: Vec<String>,
    pub my_accept: bool,
    pub opp_accept: bool,
    /// Who currently holds the pause (only they may resume).
    pub pause_owner: Option<DuelRole>,
    /// Set once combat is decided.
    pub i_won: bool,
    /// True once end-of-combat rewards have been applied locally.
    pub resolved: bool,
}

impl DuelState {
    fn new(role: DuelRole) -> Self {
        Self {
            role,
            phase: DuelPhase::Connecting,
            opponent: None,
            my_gold_bet: 0,
            my_item_bet: Vec::new(),
            opp_gold_bet: 0,
            opp_item_bet: Vec::new(),
            my_accept: false,
            opp_accept: false,
            pause_owner: None,
            i_won: false,
            resolved: false,
        }
    }

    pub fn is_host(&self) -> bool {
        self.role == DuelRole::Host
    }
}

/// Marker resource that lives only while a *networked* combat is active. It is
/// referenced by the always-compiled combat systems so they can step aside and
/// let the duel systems drive the fight instead.
pub use crate::core::combat::mechanics::DuelActive;

// ---------------------------------------------------------------------------
// Combat snapshot streamed from host to client
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FighterSnapshot {
    pub health: f32,
    pub max_health: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub alive: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct DuelSnapshot {
    /// Host player fighter.
    pub host: FighterSnapshot,
    /// Client player fighter (the host's "enemy").
    pub client: FighterSnapshot,
    pub over: bool,
    pub host_won: bool,
    /// Floating-text events to show over each side this frame.
    pub fx_host: Vec<String>,
    pub fx_client: Vec<String>,
}

// ---------------------------------------------------------------------------
// Wire messages
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    /// Host shares its character profile right after a client connects.
    Profile(Player),
    /// Authoritative view of the betting handshake.
    Lobby {
        host_gold: u32,
        host_items: Vec<String>,
        host_accept: bool,
        client_gold: u32,
        client_items: Vec<String>,
        client_accept: bool,
    },
    /// Both players accepted: enter combat.
    StartCombat,
    /// Authoritative combat state (sent every tick).
    Snapshot(DuelSnapshot),
    /// Combat finished. Tells the client what it won (or lost).
    Result {
        client_won: bool,
        gold_won: u32,
        items_won: Vec<String>,
        xp_won: u32,
    },
    /// Pause state changed.
    Pause {
        paused: bool,
        owner_is_host: bool,
    },
}

impl ServerMessage {
    pub fn channel(&self) -> DefaultChannel {
        match self {
            ServerMessage::Snapshot {
                ..
            } => DefaultChannel::Unreliable,
            _ => DefaultChannel::ReliableOrdered,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    /// Client shares its character profile.
    Profile(Player),
    /// Client updates its wager.
    SetBet {
        gold: u32,
        items: Vec<String>,
    },
    /// Client toggles its accept flag.
    Accept(bool),
    /// Combat input: cast the ability with the given key.
    CastAbility(String),
    /// Combat input: use the consumable with the given key.
    UseConsumable(String),
    /// Combat input: request pause / resume (ownership enforced by host).
    Pause(bool),
}

impl ClientMessage {
    pub fn channel(&self) -> DefaultChannel {
        DefaultChannel::ReliableOrdered
    }
}

// ---------------------------------------------------------------------------
// Messages used internally to request a send
// ---------------------------------------------------------------------------

#[derive(Message)]
pub struct ServerSendMsg {
    pub message: ServerMessage,
    pub client: Option<ClientId>,
}

impl ServerSendMsg {
    pub fn new(message: ServerMessage, client: Option<ClientId>) -> Self {
        Self {
            message,
            client,
        }
    }
}

#[derive(Message)]
pub struct ClientSendMsg {
    pub message: ClientMessage,
}

impl ClientSendMsg {
    pub fn new(message: ClientMessage) -> Self {
        Self {
            message,
        }
    }
}

// ---------------------------------------------------------------------------
// IP helpers and transport setup
// ---------------------------------------------------------------------------

#[derive(Resource, Deref, DerefMut)]
pub struct Ip(pub String);

impl Default for Ip {
    fn default() -> Self {
        Self(local_ip().to_string())
    }
}

/// Best-effort discovery of the machine's LAN IP address.
pub fn local_ip() -> IpAddr {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return "127.0.0.1".parse().unwrap(),
    };

    if socket.connect("8.8.8.8:80").is_ok() {
        socket.local_addr().ok().map(|addr| addr.ip()).unwrap_or("127.0.0.1".parse().unwrap())
    } else {
        "127.0.0.1".parse().unwrap()
    }
}

/// True when `ip` is syntactically a valid IPv4/IPv6 address.
pub fn is_valid_ip(ip: &str) -> bool {
    ip.trim().parse::<IpAddr>().is_ok()
}

pub fn new_renet_client(ip: &str) -> (RenetClient, NetcodeClientTransport) {
    let server_addr = format!("{ip}:{DUEL_PORT}").parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());

    (client, transport)
}

pub fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = format!("0.0.0.0:{DUEL_PORT}").parse().unwrap();
    let socket = UdpSocket::bind(public_addr).expect("Duel port already in use.");
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 1,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(ConnectionConfig::default());

    (server, transport)
}

// ---------------------------------------------------------------------------
// Generic send systems (mirrors the reference design)
// ---------------------------------------------------------------------------

pub fn server_send_message(
    mut server_send_msg: MessageReader<ServerSendMsg>,
    mut server: ResMut<RenetServer>,
) {
    for msg in server_send_msg.read() {
        let bytes = encode_to_vec(&msg.message, standard()).unwrap();
        if let Some(client_id) = msg.client {
            server.send_message(client_id, msg.message.channel(), bytes);
        } else {
            server.broadcast_message(msg.message.channel(), bytes);
        }
    }
}

pub fn client_send_message(
    mut client_send_msg: MessageReader<ClientSendMsg>,
    mut client: ResMut<RenetClient>,
) {
    for msg in client_send_msg.read() {
        let bytes = encode_to_vec(&msg.message, standard()).unwrap();
        client.send_message(msg.message.channel(), bytes);
    }
}

// ---------------------------------------------------------------------------
// Session lifecycle
// ---------------------------------------------------------------------------

/// Start hosting a duel. The local player becomes the authoritative server.
pub fn start_host(commands: &mut Commands) {
    let (server, transport) = new_renet_server();
    commands.insert_resource(server);
    commands.insert_resource(transport);
    commands.insert_resource(DuelState::new(DuelRole::Host));
}

/// Connect to a duel hosted at `ip`.
pub fn start_client(commands: &mut Commands, ip: &str) {
    let (client, transport) = new_renet_client(ip);
    commands.insert_resource(client);
    commands.insert_resource(transport);
    commands.insert_resource(DuelState::new(DuelRole::Client));
}

/// Tear down every resource belonging to a duel session.
pub fn teardown_duel(commands: &mut Commands) {
    commands.remove_resource::<RenetServer>();
    commands.remove_resource::<NetcodeServerTransport>();
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
    commands.remove_resource::<DuelState>();
    commands.remove_resource::<DuelActive>();
    commands.remove_resource::<CombatState>();
    commands.remove_resource::<ActiveMonster>();
}

/// Disconnect when leaving the duel lobby without entering combat.
pub fn leave_duel_lobby(mut commands: Commands, duel: Option<Res<DuelState>>) {
    if let Some(duel) = duel {
        if duel.phase != DuelPhase::Combat && duel.phase != DuelPhase::Result {
            teardown_duel(&mut commands);
        }
    }
}

/// Disconnect when leaving combat (back to the playing screen).
pub fn leave_duel_combat(mut commands: Commands, duel: Option<Res<DuelState>>) {
    if duel.is_some() {
        teardown_duel(&mut commands);
    }
}

// ---------------------------------------------------------------------------
// Connection events
// ---------------------------------------------------------------------------

pub fn on_server_event(
    event: On<RenetServerEvent>,
    player: Res<Player>,
    mut duel: Option<ResMut<DuelState>>,
    mut server_send: MessageWriter<ServerSendMsg>,
) {
    let Some(duel) = duel.as_mut() else {
        return;
    };
    match **event {
        ServerEvent::ClientConnected {
            client_id,
        } => {
            if duel.phase == DuelPhase::Connecting {
                duel.phase = DuelPhase::Betting;
            }
            server_send
                .write(ServerSendMsg::new(ServerMessage::Profile(player.clone()), Some(client_id)));
            broadcast_lobby(duel, &mut server_send);
        },
        ServerEvent::ClientDisconnected {
            ..
        } => {
            if duel.phase == DuelPhase::Betting || duel.phase == DuelPhase::Connecting {
                duel.opponent = None;
                duel.opp_accept = false;
                duel.opp_gold_bet = 0;
                duel.opp_item_bet.clear();
                duel.phase = DuelPhase::Connecting;
            }
        },
    }
}

/// The client shares its profile once the connection is established.
pub fn client_on_connect(
    client: Option<Res<RenetClient>>,
    duel: Option<Res<DuelState>>,
    player: Res<Player>,
    mut client_send: MessageWriter<ClientSendMsg>,
    mut sent: Local<bool>,
) {
    let Some(client) = client else {
        *sent = false;
        return;
    };
    if duel.is_none() {
        return;
    }
    if client.is_connected() {
        if !*sent {
            client_send.write(ClientSendMsg::new(ClientMessage::Profile(player.clone())));
            *sent = true;
        }
    } else {
        *sent = false;
    }
}

pub fn broadcast_lobby(duel: &DuelState, server_send: &mut MessageWriter<ServerSendMsg>) {
    server_send.write(ServerSendMsg::new(
        ServerMessage::Lobby {
            host_gold: duel.my_gold_bet,
            host_items: duel.my_item_bet.clone(),
            host_accept: duel.my_accept,
            client_gold: duel.opp_gold_bet,
            client_items: duel.opp_item_bet.clone(),
            client_accept: duel.opp_accept,
        },
        None,
    ));
}

// ---------------------------------------------------------------------------
// Lobby message handling (betting handshake)
// ---------------------------------------------------------------------------

pub fn server_lobby_recv(
    mut server: ResMut<RenetServer>,
    mut duel: Option<ResMut<DuelState>>,
    mut state: Option<ResMut<CombatState>>,
    mut server_send: MessageWriter<ServerSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    let Some(duel) = duel.as_mut() else {
        return;
    };
    for id in server.clients_id() {
        while let Some(bytes) = server.receive_message(id, DefaultChannel::ReliableOrdered) {
            let Ok((msg, _)) = decode_from_slice::<ClientMessage, _>(&bytes, standard()) else {
                continue;
            };
            match msg {
                ClientMessage::Profile(p) => {
                    duel.opponent = Some(p);
                    broadcast_lobby(duel, &mut server_send);
                },
                ClientMessage::SetBet {
                    gold,
                    items,
                } => {
                    duel.opp_gold_bet = gold;
                    duel.opp_item_bet = items;
                    duel.opp_accept = false;
                    duel.my_accept = false;
                    broadcast_lobby(duel, &mut server_send);
                },
                ClientMessage::Accept(a) => {
                    duel.opp_accept = a;
                    broadcast_lobby(duel, &mut server_send);
                },
                ClientMessage::CastAbility(key) => {
                    if let Some(s) = state.as_mut() {
                        enemy_cast_ability(s, &key, &mut play_audio_msg);
                    }
                },
                ClientMessage::UseConsumable(key) => {
                    if let Some(s) = state.as_mut() {
                        enemy_use_consumable(s, &key, &mut play_audio_msg);
                    }
                },
                ClientMessage::Pause(p) => {
                    if let Some(s) = state.as_mut() {
                        set_pause(duel, s, p, DuelRole::Client, &mut server_send);
                    }
                },
            }
        }
    }
}

pub fn client_lobby_recv(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut player: ResMut<Player>,
    mut duel: Option<ResMut<DuelState>>,
    mut state: Option<ResMut<CombatState>>,
    mut level_up: ResMut<LevelUpPending>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    let Some(duel) = duel.as_mut() else {
        return;
    };

    for channel in [DefaultChannel::ReliableOrdered, DefaultChannel::Unreliable] {
        let channel_id: u8 = channel.into();
        while let Some(bytes) = client.receive_message(channel_id) {
            let Ok((msg, _)) = decode_from_slice::<ServerMessage, _>(&bytes, standard()) else {
                continue;
            };
            match msg {
                ServerMessage::Profile(p) => {
                    duel.opponent = Some(p);
                    if duel.phase == DuelPhase::Connecting {
                        duel.phase = DuelPhase::Betting;
                    }
                },
                ServerMessage::Lobby {
                    host_gold,
                    host_items,
                    host_accept,
                    client_gold,
                    client_items,
                    client_accept,
                } => {
                    // From the client's perspective the host is the opponent.
                    duel.opp_gold_bet = host_gold;
                    duel.opp_item_bet = host_items;
                    duel.opp_accept = host_accept;
                    duel.my_gold_bet = client_gold;
                    duel.my_item_bet = client_items;
                    duel.my_accept = client_accept;
                    if duel.phase == DuelPhase::Connecting {
                        duel.phase = DuelPhase::Betting;
                    }
                },
                ServerMessage::StartCombat => {
                    if let Some(opponent) = duel.opponent.clone() {
                        duel.phase = DuelPhase::Combat;
                        duel.resolved = false;
                        start_duel_combat(&mut commands, &opponent, &mut next_game_state);
                    }
                },
                ServerMessage::Snapshot(snap) => {
                    if let Some(s) = state.as_mut() {
                        apply_snapshot(s, &snap);
                    }
                },
                ServerMessage::Result {
                    client_won,
                    gold_won,
                    items_won,
                    xp_won,
                } => {
                    if !duel.resolved {
                        duel.resolved = true;
                        duel.i_won = client_won;
                        duel.phase = DuelPhase::Result;
                        apply_local_rewards(
                            &mut player,
                            client_won,
                            gold_won,
                            &items_won,
                            xp_won,
                            duel.my_gold_bet,
                            &duel.my_item_bet,
                            &mut level_up,
                            &mut play_audio_msg,
                            &mut next_game_state,
                        );
                    }
                },
                ServerMessage::Pause {
                    paused,
                    owner_is_host,
                } => {
                    if let Some(s) = state.as_mut() {
                        s.paused = paused;
                    }
                    duel.pause_owner = if paused {
                        Some(if owner_is_host {
                            DuelRole::Host
                        } else {
                            DuelRole::Client
                        })
                    } else {
                        None
                    };
                },
            }
        }
    }
}

/// The host starts combat once both players have accepted their wagers.
pub fn host_check_start(
    mut commands: Commands,
    mut duel: Option<ResMut<DuelState>>,
    mut server_send: MessageWriter<ServerSendMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Some(duel) = duel.as_mut() else {
        return;
    };
    if duel.phase == DuelPhase::Betting
        && duel.my_accept
        && duel.opp_accept
        && duel.opponent.is_some()
    {
        let opponent = duel.opponent.clone().unwrap();
        duel.phase = DuelPhase::Combat;
        duel.resolved = false;
        server_send.write(ServerSendMsg::new(ServerMessage::StartCombat, None));
        start_duel_combat(&mut commands, &opponent, &mut next_game_state);
    }
}

// ---------------------------------------------------------------------------
// Combat helpers
// ---------------------------------------------------------------------------

/// Build the character image key for `player` (mirrors the playing screen).
pub fn portrait_key(player: &Player) -> String {
    match player.class {
        Class::Mage(ajah) => ajah.get_image_key(player),
        _ => player.class.get_image_key(player),
    }
}

/// Represent the opposing player as a `Monster` so the standard combat engine
/// and UI (which expect an `ActiveMonster`) can drive the duel unchanged.
fn player_to_monster(opponent: &Player) -> Monster {
    let mut effects = Vec::new();
    for eq in opponent.equipped_equipment() {
        if let Equipment::Weapon(w) = eq {
            effects.extend(w.effects.clone());
        }
    }
    Monster {
        name: opponent.name.clone(),
        image: portrait_key(opponent),
        level: opponent.level(),
        kind: MonsterKind::Creature,
        health: opponent.health(),
        max_health: opponent.max_health(),
        attack: opponent.attack(),
        defense: opponent.defense(),
        initiative: opponent.initiative(),
        attack_speed: opponent.attack_speed(),
        health_regen: opponent.health_regen(),
        modifiers: Vec::new(),
        effects,
    }
}

fn start_duel_combat(
    commands: &mut Commands,
    opponent: &Player,
    next_game_state: &mut NextState<GameState>,
) {
    commands.insert_resource(ActiveMonster {
        monster: player_to_monster(opponent),
    });
    commands.insert_resource(DuelActive);
    commands.insert_resource(CombatSpeed(1.0));
    next_game_state.set(GameState::Combat);
}

fn fighter_snap(f: &Fighter) -> FighterSnapshot {
    FighterSnapshot {
        health: f.health,
        max_health: f.max_health,
        mana: f.mana,
        max_mana: f.max_mana,
        alive: f.alive,
    }
}

fn apply_fighter(target: &mut Fighter, snap: &FighterSnapshot) {
    target.health = snap.health;
    target.max_health = snap.max_health;
    target.mana = snap.mana;
    target.max_mana = snap.max_mana;
    target.alive = snap.alive;
}

fn apply_snapshot(state: &mut CombatState, snap: &DuelSnapshot) {
    // Client perspective: local player == snap.client, enemy == snap.host.
    apply_fighter(&mut state.player, &snap.client);
    apply_fighter(&mut state.enemy, &snap.host);
    if snap.over {
        state.status = CombatStatus::Over;
        state.player_won = !snap.host_won;
    }
    for text in &snap.fx_client {
        state.fx.push(CombatFx {
            side: FxSide::Player,
            text: text.clone(),
            color: Color::WHITE,
        });
    }
    for text in &snap.fx_host {
        state.fx.push(CombatFx {
            side: FxSide::Enemy,
            text: text.clone(),
            color: Color::WHITE,
        });
    }
}

/// Apply a pause/resume request, honouring the rule that only the player who
/// paused may resume. Runs on the authoritative host.
fn set_pause(
    duel: &mut DuelState,
    state: &mut CombatState,
    paused: bool,
    requester: DuelRole,
    server_send: &mut MessageWriter<ServerSendMsg>,
) {
    if paused {
        if duel.pause_owner.is_none() {
            duel.pause_owner = Some(requester);
            state.paused = true;
        }
    } else if duel.pause_owner == Some(requester) {
        duel.pause_owner = None;
        state.paused = false;
    } else {
        return;
    }
    server_send.write(ServerSendMsg::new(
        ServerMessage::Pause {
            paused: state.paused,
            owner_is_host: duel.pause_owner == Some(DuelRole::Host),
        },
        None,
    ));
}

fn level_diff_xp(winner_level: u32, loser_level: u32) -> u32 {
    (2 + loser_level as i32 - winner_level as i32).max(0) as u32
}

#[allow(clippy::too_many_arguments)]
fn apply_local_rewards(
    player: &mut Player,
    won: bool,
    gold_won: u32,
    items_won: &[String],
    xp_won: u32,
    my_gold_bet: u32,
    my_item_bet: &[String],
    level_up: &mut LevelUpPending,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    next_game_state: &mut NextState<GameState>,
) {
    if won {
        player.gold = player.gold.saturating_add(gold_won);
        for it in items_won {
            player.add_inventory_item(it.clone());
        }
        gain_xp(player, xp_won, level_up, play_audio_msg, next_game_state, false);
        play_audio_msg.write(PlayAudioMsg::new("levelup").volume(-10.));
    } else {
        player.gold = player.gold.saturating_sub(my_gold_bet);
        for it in my_item_bet {
            if let Some(pos) = player.inventory.iter().position(|k| k == it) {
                player.inventory.remove(pos);
            }
        }
        play_audio_msg.write(PlayAudioMsg::new("defeat"));
    }
}

// ---------------------------------------------------------------------------
// Combat systems
// ---------------------------------------------------------------------------

const SNAPSHOT_FX_CAP: usize = 8;

pub fn duel_host_combat(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<CombatState>>,
    mut player: ResMut<Player>,
    mut duel: Option<ResMut<DuelState>>,
    mut server_send: MessageWriter<ServerSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut level_up: ResMut<LevelUpPending>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Some(duel) = duel.as_mut() else {
        return;
    };
    if !duel.is_host() {
        return;
    }
    let Some(state) = state.as_mut() else {
        return;
    };

    // Result screen: wait for the host to leave.
    if state.status == CombatStatus::Over {
        resolve_host_result(
            duel,
            state,
            &mut player,
            &mut level_up,
            &mut play_audio_msg,
            &mut server_send,
            &mut next_game_state,
            &keyboard,
        );
        return;
    }

    // Capture floating text generated this frame (casts + simulation) for the client.
    let fx_before = state.fx.len();

    // Pause (ownership enforced).
    if keyboard.just_pressed(KeyCode::Space) {
        let want = !state.paused;
        set_pause(duel, state, want, DuelRole::Host, &mut server_send);
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }

    // Host abilities / consumables (applied locally; the host is authoritative).
    if !state.paused {
        for (i, key) in ABILITY_HOTKEYS.iter().enumerate() {
            if keyboard.just_pressed(*key) {
                try_cast_ability(state, i, &mut play_audio_msg);
            }
        }
        let equipped = consumable_card_order(&player);
        for (i, hotkey) in CONSUMABLE_HOTKEYS.iter().enumerate() {
            if keyboard.just_pressed(*hotkey) {
                if let Some(key) = equipped.get(i) {
                    let key = key.clone();
                    try_use_consumable(state, &mut player, &key, &mut play_audio_msg);
                }
            }
        }
    }

    // Step the authoritative simulation at a fixed 1x speed.
    step_combat(state, time.delta_secs(), &mut play_audio_msg);

    // Persist the host's hit points.
    let new_hp = state.player.health.round() as u32;
    if player.health() != new_hp {
        player.set_health(new_hp);
    }
    let new_mp = state.player.mana.round() as u32;
    if player.mana() != new_mp {
        player.set_mana(new_mp);
    }

    // Forward this frame's floating text and fighter state to the client.
    let (fx_host, fx_client) = collect_fx(state, fx_before);
    let snap = DuelSnapshot {
        host: fighter_snap(&state.player),
        client: fighter_snap(&state.enemy),
        over: state.status == CombatStatus::Over,
        host_won: state.player_won,
        fx_host,
        fx_client,
    };
    server_send.write(ServerSendMsg::new(ServerMessage::Snapshot(snap), None));
}

#[allow(clippy::too_many_arguments)]
fn resolve_host_result(
    duel: &mut DuelState,
    state: &mut CombatState,
    player: &mut Player,
    level_up: &mut LevelUpPending,
    play_audio_msg: &mut MessageWriter<PlayAudioMsg>,
    server_send: &mut MessageWriter<ServerSendMsg>,
    next_game_state: &mut NextState<GameState>,
    keyboard: &ButtonInput<KeyCode>,
) {
    if !duel.resolved {
        duel.resolved = true;
        duel.phase = DuelPhase::Result;
        let host_won = state.player_won;
        duel.i_won = host_won;
        let host_level = player.level();
        let opp_level = duel.opponent.as_ref().map(|o| o.level()).unwrap_or(host_level);

        // One authoritative snapshot so the client also sees the final state.
        let snap = DuelSnapshot {
            host: fighter_snap(&state.player),
            client: fighter_snap(&state.enemy),
            over: true,
            host_won,
            fx_host: Vec::new(),
            fx_client: Vec::new(),
        };
        server_send.write(ServerSendMsg::new(ServerMessage::Snapshot(snap), None));

        if host_won {
            player.gold = player.gold.saturating_add(duel.opp_gold_bet);
            for it in duel.opp_item_bet.clone() {
                player.add_inventory_item(it);
            }
            let xp = level_diff_xp(host_level, opp_level);
            gain_xp(player, xp, level_up, play_audio_msg, next_game_state, false);
        } else {
            player.gold = player.gold.saturating_sub(duel.my_gold_bet);
            for it in &duel.my_item_bet {
                if let Some(pos) = player.inventory.iter().position(|k| k == it) {
                    player.inventory.remove(pos);
                }
            }
            play_audio_msg.write(PlayAudioMsg::new("defeat"));
        }

        // Tell the client what it won (the host's wager) or lost.
        server_send.write(ServerSendMsg::new(
            ServerMessage::Result {
                client_won: !host_won,
                gold_won: if host_won {
                    0
                } else {
                    duel.my_gold_bet
                },
                items_won: if host_won {
                    Vec::new()
                } else {
                    duel.my_item_bet.clone()
                },
                xp_won: if host_won {
                    0
                } else {
                    level_diff_xp(opp_level, host_level)
                },
            },
            None,
        ));
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        next_game_state.set(GameState::Playing);
    }
}

pub fn duel_client_combat(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: Option<ResMut<CombatState>>,
    mut player: ResMut<Player>,
    duel: Option<Res<DuelState>>,
    mut client_send: MessageWriter<ClientSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let Some(duel) = duel else {
        return;
    };
    if duel.is_host() {
        return;
    }
    let Some(state) = state.as_mut() else {
        return;
    };

    // Tick our own ability cooldowns locally for responsive UI feedback.
    if !state.paused && state.status != CombatStatus::Over {
        for slot in state.abilities.iter_mut() {
            if slot.remaining > 0.0 {
                slot.remaining = (slot.remaining - time.delta_secs()).max(0.0);
            }
        }
    }

    // Keep the local Player hit points in sync with the authoritative snapshot.
    let snap_hp = state.player.health.round() as u32;
    if player.health() != snap_hp {
        player.set_health(snap_hp);
    }

    if state.status == CombatStatus::Over {
        if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
            next_game_state.set(GameState::Playing);
        }
        return;
    }

    // Pause request (ownership enforced by host; we may only resume our own).
    if keyboard.just_pressed(KeyCode::Space) {
        let owner = duel.pause_owner;
        if !state.paused {
            client_send.write(ClientSendMsg::new(ClientMessage::Pause(true)));
        } else if owner == Some(DuelRole::Client) {
            client_send.write(ClientSendMsg::new(ClientMessage::Pause(false)));
        }
        play_audio_msg.write(PlayAudioMsg::new("button"));
    }
    if state.paused {
        return;
    }

    // Abilities: forward to the host and optimistically show the cooldown.
    for (i, key) in ABILITY_HOTKEYS.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            if let Some(Some(ability_key)) = player.active_abilities.get(i).cloned() {
                client_send
                    .write(ClientSendMsg::new(ClientMessage::CastAbility(ability_key.clone())));
                if let Some(slot) = state.abilities.get_mut(i) {
                    slot.remaining = slot.cooldown;
                }
                play_audio_msg.write(PlayAudioMsg::new("cast"));
            }
        }
    }

    // Consumables: forward to the host and consume one locally.
    let equipped = consumable_card_order(&player);
    for (i, hotkey) in CONSUMABLE_HOTKEYS.iter().enumerate() {
        if keyboard.just_pressed(*hotkey) {
            if let Some(key) = equipped.get(i).cloned() {
                client_send.write(ClientSendMsg::new(ClientMessage::UseConsumable(key.clone())));
                consume_one(&mut player, &key);
                play_audio_msg.write(PlayAudioMsg::new("drink"));
            }
        }
    }
}

/// Card clicks during a duel: host applies locally, client forwards to host.
pub fn duel_combat_card_click(
    event: On<Pointer<Click>>,
    card_q: Query<&CombatCard>,
    duel: Option<Res<DuelState>>,
    mut state: Option<ResMut<CombatState>>,
    mut player: ResMut<Player>,
    mut client_send: MessageWriter<ClientSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
) {
    let Some(duel) = duel else {
        return;
    };
    let Some(state) = state.as_mut() else {
        return;
    };
    let Ok(card) = card_q.get(event.entity) else {
        return;
    };
    if state.paused || state.status == CombatStatus::Over {
        return;
    }

    match card.clone() {
        CombatCard::Ability(index) => {
            if duel.is_host() {
                try_cast_ability(state, index, &mut play_audio_msg);
            } else if let Some(Some(ability_key)) = player.active_abilities.get(index).cloned() {
                client_send.write(ClientSendMsg::new(ClientMessage::CastAbility(ability_key)));
                if let Some(slot) = state.abilities.get_mut(index) {
                    slot.remaining = slot.cooldown;
                }
                play_audio_msg.write(PlayAudioMsg::new("cast"));
            }
        },
        CombatCard::Consumable(key) => {
            if duel.is_host() {
                try_use_consumable(state, &mut player, &key, &mut play_audio_msg);
            } else {
                client_send.write(ClientSendMsg::new(ClientMessage::UseConsumable(key.clone())));
                consume_one(&mut player, &key);
                play_audio_msg.write(PlayAudioMsg::new("drink"));
            }
        },
    }
}

fn consume_one(player: &mut Player, key: &str) {
    if let Some(pos) = player.inventory.iter().position(|k| k == key) {
        player.inventory.remove(pos);
    }
    if !player.inventory.iter().any(|k| k == key) {
        player.equipped_consumables.retain(|k| k != key);
    }
}

/// Collect floating-text events generated since `from` and split them by side.
fn collect_fx(state: &CombatState, from: usize) -> (Vec<String>, Vec<String>) {
    let mut fx_host = Vec::new();
    let mut fx_client = Vec::new();
    for fx in state.fx.iter().skip(from) {
        match fx.side {
            FxSide::Player => {
                if fx_host.len() < SNAPSHOT_FX_CAP {
                    fx_host.push(fx.text.clone());
                }
            },
            FxSide::Enemy => {
                if fx_client.len() < SNAPSHOT_FX_CAP {
                    fx_client.push(fx.text.clone());
                }
            },
        }
    }
    (fx_host, fx_client)
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RenetServerPlugin,
            RenetClientPlugin,
            NetcodeServerPlugin,
            NetcodeClientPlugin,
        ))
        .init_resource::<Ip>()
        .add_message::<ServerSendMsg>()
        .add_message::<ClientSendMsg>()
        .add_observer(on_server_event)
        .add_observer(duel_combat_card_click)
        .add_systems(
            Update,
            (
                // Lobby + connection handling.
                client_on_connect.run_if(resource_exists::<RenetClient>),
                server_lobby_recv.run_if(resource_exists::<RenetServer>),
                client_lobby_recv.run_if(resource_exists::<RenetClient>),
                host_check_start.run_if(resource_exists::<RenetServer>),
                // Authoritative / mirrored combat.
                duel_host_combat
                    .before(crate::core::combat::mechanics::update_combat_visuals)
                    .run_if(in_state(GameState::Combat).and_then(resource_exists::<DuelActive>)),
                duel_client_combat
                    .before(crate::core::combat::mechanics::update_combat_visuals)
                    .run_if(in_state(GameState::Combat).and_then(resource_exists::<DuelActive>)),
            ),
        )
        .add_systems(
            Update,
            (
                server_send_message.run_if(resource_exists::<RenetServer>),
                client_send_message.run_if(resource_exists::<RenetClient>),
            )
                .after(server_lobby_recv)
                .after(client_lobby_recv),
        )
        .add_systems(OnExit(GameState::Duel), leave_duel_lobby)
        .add_systems(OnExit(GameState::Combat), leave_duel_combat)
        .add_systems(
            Update,
            (
                crate::core::actions::duel::duel_ip_input,
                crate::core::actions::duel::refresh_duel_lobby,
            )
                .run_if(in_state(GameState::Duel)),
        );
    }
}
