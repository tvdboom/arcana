use bevy::prelude::*;
use bevy_renet::{RenetClient, RenetServer};
use bevy_renet::renet::DefaultChannel;

use crate::core::audio::SoundEffect;
use crate::core::network::{ClientMessage, NetworkManager, ServerMessage};
use crate::core::player::Character;
use crate::core::states::AppState;
use crate::core::systems::combat_engine::CombatSession;

pub struct NetworkSyncPlugin;

impl Plugin for NetworkSyncPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkManager::default())
            .insert_resource(CombatSession::default())
            .add_systems(Update, (server_sync_system, client_sync_system));
    }
}

fn server_sync_system(
    mut server: Option<ResMut<RenetServer>>,
    mut net_manager: ResMut<NetworkManager>,
    mut combat_session: ResMut<CombatSession>,
    mut app_state: ResMut<NextState<AppState>>,
    character: Option<Res<Character>>,
    mut sfx_writer: MessageWriter<SoundEffect>,
) {
    let Some(ref mut server) = server else { return };

    // Read messages from clients
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
            if let Ok(client_msg) = postcard::from_bytes::<ClientMessage>(&message) {
                match client_msg {
                    ClientMessage::Introduce { character: opponent_character } => {
                        net_manager.opponent_character = Some(*opponent_character);
                        
                        // Send server's character back to the client
                        if let Some(my_char) = character.as_ref() {
                            let msg = ServerMessage::LobbySync { opponent: my_char.as_ref().clone() };
                            if let Ok(bytes) = postcard::to_stdvec(&msg) {
                                server.send_message(client_id, DefaultChannel::ReliableOrdered, bytes);
                            }
                        }
                    }
                    ClientMessage::UpdateWager { wager_gold, level_cap } => {
                        net_manager.wager_gold = wager_gold;
                        net_manager.level_cap = level_cap;
                        
                        // Sync wager details to client
                        let msg = ServerMessage::LockStakes { wager_gold, level_cap };
                        if let Ok(bytes) = postcard::to_stdvec(&msg) {
                            server.send_message(client_id, DefaultChannel::ReliableOrdered, bytes);
                        }
                    }
                    ClientMessage::Ready => {
                        net_manager.opponent_ready = true;
                        
                        // Notify host client
                        let msg = ServerMessage::OpponentReady;
                        if let Ok(bytes) = postcard::to_stdvec(&msg) {
                            server.send_message(client_id, DefaultChannel::ReliableOrdered, bytes);
                        }

                        // Start fight if both ready
                        if net_manager.my_ready && net_manager.opponent_ready {
                            if let Some(my_char) = character.as_ref() {
                                if let Some(ref enemy_char) = net_manager.opponent_character {
                                    // Scale characters down to level cap if needed
                                    let mut p1 = my_char.as_ref().clone();
                                    let mut p2 = enemy_char.clone();
                                    scale_to_cap(&mut p1, net_manager.level_cap);
                                    scale_to_cap(&mut p2, net_manager.level_cap);

                                    // Build session
                                    let mut session = CombatSession::init_hunt(&p1, &p2.name);
                                    session.opponent = Some(crate::core::systems::combat_engine::Combatant::from_character(&p2));
                                    session.opponent_pet = p2.pet.as_ref().map(crate::core::systems::combat_engine::Combatant::from_pet);
                                    session.is_pvp = true;

                                    *combat_session = session.clone();

                                    let msg = ServerMessage::StartCombat { session };
                                    if let Ok(bytes) = postcard::to_stdvec(&msg) {
                                        server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);
                                    }
                                    app_state.set(AppState::Combat);
                                }
                            }
                        }
                    }
                    ClientMessage::CastAbility { index } => {
                        if combat_session.active {
                            let _ = combat_session.cast_ability(false, index, &mut sfx_writer);
                        }
                    }
                    ClientMessage::UseConsumable { index } => {
                        if combat_session.active {
                            let _ = combat_session.use_consumable(false, index, &mut sfx_writer);
                        }
                    }
                }
            }
        }
    }

    // Host combat ticker
    if combat_session.active && combat_session.is_pvp && combat_session.victory_state.is_none() {
        // Ticks handled by main Bevy update loop in screen_combat
        // Broadcast updates to clients
        let msg = ServerMessage::CombatUpdate { session: combat_session.clone() };
        if let Ok(bytes) = postcard::to_stdvec(&msg) {
            server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);
        }
    }
}

fn client_sync_system(
    mut client: Option<ResMut<RenetClient>>,
    mut net_manager: ResMut<NetworkManager>,
    mut combat_session: ResMut<CombatSession>,
    mut app_state: ResMut<NextState<AppState>>,
    mut _sfx_writer: MessageWriter<SoundEffect>,
) {
    let Some(ref mut client) = client else { return };

    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        if let Ok(server_msg) = postcard::from_bytes::<ServerMessage>(&message) {
            match server_msg {
                ServerMessage::LobbySync { opponent } => {
                    net_manager.opponent_character = Some(opponent);
                }
                ServerMessage::LockStakes { wager_gold, level_cap } => {
                    net_manager.wager_gold = wager_gold;
                    net_manager.level_cap = level_cap;
                }
                ServerMessage::OpponentReady => {
                    net_manager.opponent_ready = true;
                }
                ServerMessage::StartCombat { session } => {
                    *combat_session = session;
                    app_state.set(AppState::Combat);
                }
                ServerMessage::CombatUpdate { session } => {
                    *combat_session = session;
                }
            }
        }
    }
}

fn scale_to_cap(character: &mut Character, cap: u32) {
    if character.level > cap {
        // Level down stats proportionally or cap them
        let ratio = cap as f32 / character.level as f32;
        character.spent_stats.strength = (character.spent_stats.strength as f32 * ratio) as u32;
        character.spent_stats.dexterity = (character.spent_stats.dexterity as f32 * ratio) as u32;
        character.spent_stats.intelligence = (character.spent_stats.intelligence as f32 * ratio) as u32;
        character.spent_stats.charisma = (character.spent_stats.charisma as f32 * ratio) as u32;
        character.level = cap;
    }
}
