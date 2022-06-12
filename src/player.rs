use crate::network::Token;
use crate::ty::WS;
use crate::{packet, ServerNetwork};
use euclid::Vector2D;
use eyre::Result;
use std::collections::HashMap;
use tracing::{info, trace};

packet!(Player(ServerBoundPlayerPacket, ClientBoundPlayerPacket));

pub enum ServerBoundPlayerPacket {
    SetDir(u32, Vector2D<f32, WS>),
    Join(),
}

pub enum ClientBoundPlayerPacket {
    RespondPos(u32, Option<Vector2D<f32, WS>>),
}

pub(crate) struct PlayerSystem {
    players: HashMap<Token, Player>,
    response_requests: Vec<(u32, Token)>,
}

impl PlayerSystem {
    pub fn new() -> Result<PlayerSystem> {
        info!("Initializing");
        Ok(PlayerSystem {
            players: Default::default(),
            response_requests: vec![]
        })
    }

    pub fn tick(&mut self, networking: &mut ServerNetwork) -> Result<()> {
        for player in self.players.values_mut() {
            player.tick(1.0);
        }

        for (tick, token) in self.response_requests.drain(..) {
            let response = self.players.get(&token).map(|player| player.pos);
            networking.send(token, ClientBoundPlayerPacket::RespondPos(tick, response))?;
        }

        Ok(())
    }

    pub fn packet(&mut self, token: Token, packet: ServerBoundPlayerPacket) {
	    match packet {
		    ServerBoundPlayerPacket::SetDir(tick, speed) => {
                if let Some(player) = self.players.get_mut(&token) {
                    player.velocity = speed;
                }
                self.response_requests.push((tick, token));
            }
            ServerBoundPlayerPacket::Join() => {
                info!("Player {:?} joined", token);
                self.players.insert(token, Player { pos: Default::default(), velocity: Default::default() });
            }
        }
    }
}

#[derive(Clone)]
pub struct Player {
    pub pos: Vector2D<f32, WS>,
    pub velocity: Vector2D<f32, WS>,
}

impl Player {
    pub fn tick(&mut self, delta: f32) {
        self.pos += self.velocity * delta;
    }
}
