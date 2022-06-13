#![allow(clippy::new_without_default)]

use std::collections::HashMap;

use eyre::{Context, Result};
use tracing::info;

use ty::chunk_pos::ChunkPos;

use crate::api::Carrier;
use crate::chunk::Chunk;
use crate::debug::DummyRenderer;
use crate::entity::EntityWorld;
use crate::network::packet::{ClientBoundPacket, ServerBoundPacket};
use crate::network::ServerNetwork;
use crate::player::PlayerSystem;

pub mod chunk;
pub mod network;
pub mod ty;
pub mod api;
pub mod player;
pub mod entity;
pub mod util;
pub mod debug;

pub const TPS: usize = 144;

pub struct Server {
    chunks: HashMap<ChunkPos, Chunk>,
    network: ServerNetwork,
    entity: EntityWorld,
    player: PlayerSystem,
}

impl Server {
    pub fn new(carrier: &Carrier, network: ServerNetwork) -> Result<Server> {
        info!("Launching integrated server.");
        Ok(Server {
            chunks: Default::default(),
            network,
            entity: EntityWorld::new(carrier)?,
            player: PlayerSystem::new(carrier)?
        })
    }

    pub fn tick(&mut self) -> Result<()> {
        for (token, packet) in self.network.poll() {
            match packet {
                ServerBoundPacket::RequestChunk(pos) => if let Some(chunk) = self.get_chunk(pos) {
                    self.network.send(token, ClientBoundPacket::Chunk(pos, chunk.clone()))?;
                },
                ServerBoundPacket::Player(packet) => {
                    self.player.packet(token, packet, &mut self.entity);
                }
            }
        }

        self.entity.tick(&self.chunks, &mut DummyRenderer);
        self.player.tick(&mut self.network, &self.entity).wrap_err("Ticking player system.")?;
        Ok(())
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn put_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }
}
