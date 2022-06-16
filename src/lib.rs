#![allow(clippy::new_without_default)]

use eyre::{Context, Result};
use tracing::info;

use ty::chunk_pos::ChunkPos;
use crate::api::Api;

use crate::chunk::storage::ChunkStorage;
use crate::chunk::Chunk;
use crate::debug::DummyRenderer;
use crate::entity::EntityWorld;
use crate::network::packet::{ClientBoundPacket, ServerBoundPacket};
use crate::network::ServerNetwork;
use crate::player::PlayerSystem;
use crate::world::World;

pub mod api;
pub mod chunk;
pub mod debug;
pub mod entity;
pub mod network;
pub mod player;
pub mod ty;
pub mod util;
pub mod world;

pub const TPS: usize = 60;

pub struct Server {
    network: ServerNetwork,
    player: PlayerSystem,
    world: World,
}

impl Server {
    pub fn new(api: &Api, network: ServerNetwork, world: World) -> Result<Server> {
        info!("Launching integrated server.");
        Ok(Server {
            network,
            player: PlayerSystem::new(api)?,
            world
        })
    }

    pub fn tick(&mut self, api: &Api) -> Result<()> {
        for (token, packet) in self.network.poll() {
            match packet {

                ServerBoundPacket::Player(packet) => {
                    self.player.packet(token, packet, &mut self.world);
                }
                ServerBoundPacket::World(packet) => {
                    self.world.packet(api, token, packet, &mut self.network)?;
                }
            }
        }

        self.world.tick(api, &mut DummyRenderer);
        self.player
            .tick(&mut self.network, &self.world)
            .wrap_err("Ticking player system.")?;
        Ok(())
    }
}
