use std::collections::hash_map::Entry;
use std::collections::{HashMap};

use euclid::{vec2, Vector2D};
use eyre::{ContextCompat, Result};
use hecs::{Entity, EntityRef};
use tracing::{debug, info, trace, warn};

use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::Carrier;
use crate::entity::component::{PositionComponent, PhysicsComponent};
use crate::entity::prototype::EntityPrototype;
use crate::network::Token;
use crate::ty::WS;
use crate::{packet, EntityWorld, ServerNetwork};

packet!(Player(ServerBoundPlayerPacket, ClientBoundPlayerPacket));

pub enum ServerBoundPlayerPacket {
    SetDir(u32, Vector2D<f32, WS>),
    Join(),
}

pub enum ClientBoundPlayerPacket {
    RespondPos(u32, Option<Vector2D<f32, WS>>),
    Joined(Entity)
}

pub(crate) struct PlayerSystem {
    players: HashMap<Token, Option<Entity>>,
    response_requests: Vec<(u32, Token)>,
    joined: Vec<(Token, Entity)>,
    player_entity: Id<EntityPrototype>,
}

impl PlayerSystem {
    pub fn new(carrier: &Carrier) -> Result<PlayerSystem> {
        info!("Initializing");
        Ok(PlayerSystem {
            players: Default::default(),
            response_requests: vec![],
            joined: Default::default(),
            player_entity: carrier
                .entity
                .identifier_to_id(&Identifier::new("player"))
                .wrap_err("Could not find Player entity")?,
        })
    }

    fn get_player_entity<'a, 'e>(
        &'a mut self,
        token: Token,
        entity_world: &'e EntityWorld,
    ) -> Option<EntityRef<'e>> {
        match self.players.entry(token) {
            Entry::Occupied(mut occupied) => {
                if let Some(entity) = *occupied.get() {
                    if let Some(entity) = entity_world.storage.get(entity) {
                        return Some(entity);
                    } else {
                        warn!("Player entity got yeeted");
                        (*occupied.get_mut()) = None;
                    }
                }
            }
            Entry::Vacant(_) => {}
        }
        None
    }

    pub fn tick(
        &mut self,
        networking: &mut ServerNetwork,
        entity_world: &EntityWorld,
    ) -> Result<()> {
        for (token, entity) in self.joined.drain(..) {
            debug!("Sent joined packet");
            networking.send(token, ClientBoundPlayerPacket::Joined(entity))?;
        }

        let responses: Vec<_> = self.response_requests.drain(..).collect();
        for (tick, token) in responses {
            networking.send(
                token,
                ClientBoundPlayerPacket::RespondPos(
                    tick,
                    self.get_player_entity(token, entity_world)
                        .map(|entity| {
                            entity.get::<PositionComponent>().expect("where pos").pos
                        }),
                ),
            )?;
        }

        Ok(())
    }

    pub fn packet(
        &mut self,
        token: Token,
        packet: ServerBoundPlayerPacket,
        entity: &mut EntityWorld,
    ) {
        match packet {
            ServerBoundPlayerPacket::SetDir(tick, speed) => {
                if let Some(player) = self.get_player_entity(token, entity) {
                    player
                        .get_mut::<PhysicsComponent>()
                        .expect("Player does not have velocity")
                        .vel = speed;
                }  else {
                    trace!("player entity not here");
                }
                self.response_requests.push((tick, token));
            }
            ServerBoundPlayerPacket::Join() => {
                info!("Player {:?} joined", token);
                let entity = entity.storage.push(self.player_entity);
                self.players.insert(token, Some(entity));
                self.joined.push((token, entity));
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
