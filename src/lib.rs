#![allow(clippy::new_without_default)]

use std::collections::HashMap;

use eyre::{Context, Result};
use tracing::info;

use ty::chunk_pos::ChunkPos;

use crate::api::{Carrier, CarrierAccess};
use crate::chunk::storage::ChunkStorage;
use crate::chunk::Chunk;
use crate::debug::DummyRenderer;
use crate::entity::EntityWorld;
use crate::network::packet::{ClientBoundPacket, ServerBoundPacket};
use crate::network::ServerNetwork;
use crate::player::PlayerSystem;

pub mod api;
pub mod chunk;
pub mod debug;
pub mod entity;
pub mod network;
pub mod player;
pub mod ty;
pub mod util;

pub const TPS: usize = 144;

pub struct Server {
    chunks: ChunkStorage,
    network: ServerNetwork,
    entity: EntityWorld,
    player: PlayerSystem,
}

impl Server {
    pub fn new(carrier: &Carrier, network: ServerNetwork, storage: ChunkStorage) -> Result<Server> {
        info!("Launching integrated server.");
        Ok(Server {
            chunks: storage,
            network,
            entity: EntityWorld::new(carrier)?,
            player: PlayerSystem::new(carrier)?,
        })
    }

    pub fn tick(&mut self, carrier: &Carrier) -> Result<()> {
        for (token, packet) in self.network.poll() {
            match packet {
                ServerBoundPacket::RequestChunk(pos) => {
                    if let Some(chunk) = self.chunks.get(pos) {
                        self.network
                            .send(token, ClientBoundPacket::Chunk(pos, chunk.clone()))?;
                    }
                }
                ServerBoundPacket::SetTile(pos, id) => {
                    if let Some(chunk) = self.chunks.get_mut(pos.chunk) {
                        chunk.tile[pos.entry] = carrier.create(id);
                    }
                }
                ServerBoundPacket::Player(packet) => {
                    self.player.packet(token, packet, &mut self.entity);
                }
            }
        }

        self.entity.tick(&self.chunks, &mut DummyRenderer);
        self.player
            .tick(&mut self.network, &self.entity)
            .wrap_err("Ticking player system.")?;
        Ok(())
    }
}
