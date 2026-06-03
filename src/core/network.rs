use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_renet::renet::ConnectionConfig;
use bevy_renet::{RenetClient, RenetServer};
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication, ServerConfig as NetcodeServerConfig,
};
use serde::{Deserialize, Serialize};

use crate::core::player::Character;
use crate::core::systems::combat_engine::CombatSession;

pub const PROTOCOL_ID: u64 = 998877;
pub const PORT: u16 = 5000;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    LobbySync {
        opponent: Character,
    },
    LockStakes {
        wager_gold: u32,
        level_cap: u32,
    },
    StartCombat {
        session: CombatSession,
    },
    CombatUpdate {
        session: CombatSession,
    },
    OpponentReady,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Introduce {
        character: Box<Character>,
    },
    UpdateWager {
        wager_gold: u32,
        level_cap: u32,
    },
    Ready,
    CastAbility {
        index: usize,
    },
    UseConsumable {
        index: usize,
    },
}

#[derive(Resource)]
pub struct NetworkManager {
    pub is_host: bool,
    pub ip_address: String,
    pub opponent_character: Option<Character>,
    pub wager_gold: u32,
    pub level_cap: u32,
    pub my_ready: bool,
    pub opponent_ready: bool,
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self {
            is_host: false,
            ip_address: "127.0.0.1".to_string(),
            opponent_character: None,
            wager_gold: 0,
            level_cap: 1,
            my_ready: false,
            opponent_ready: false,
        }
    }
}

pub fn get_local_ip() -> String {
    let socket = UdpSocket::bind("0.0.0.0:0");
    if let Ok(s) = socket {
        if s.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = s.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

pub fn host_server(commands: &mut Commands) -> Result<(), String> {
    let public_addr = format!("0.0.0.0:{}", PORT).parse().unwrap();
    let socket = UdpSocket::bind(public_addr)
        .map_err(|e| format!("Failed to bind server socket: {}", e))?;
        
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
        
    let server_config = NetcodeServerConfig {
        current_time,
        max_clients: 2,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket)
        .map_err(|e| format!("Failed to create server transport: {}", e))?;
        
    let server = RenetServer::new(ConnectionConfig::default());

    commands.insert_resource(server);
    commands.insert_resource(transport);

    // Also connect host as its own client
    connect_client(commands, "127.0.0.1")?;
    Ok(())
}

pub fn connect_client(commands: &mut Commands, ip: &str) -> Result<(), String> {
    let server_addr = format!("{}:{}", ip, PORT)
        .parse()
        .map_err(|_| "Invalid IP address format.".to_string())?;
        
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind client socket: {}", e))?;
        
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
        
    let client_id = current_time.as_millis() as u64;
    
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .map_err(|e| format!("Failed to create client transport: {}", e))?;
        
    let client = RenetClient::new(ConnectionConfig::default());

    commands.insert_resource(client);
    commands.insert_resource(transport);
    Ok(())
}
