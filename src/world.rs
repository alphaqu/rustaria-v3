use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoroshiro64Star;
use crate::{Api, Chunk, ChunkPos, ChunkStorage, draw_debug, EntityWorld, packet, ServerNetwork, TPS};
use crate::api::prototype::FactoryPrototype;
use crate::api::registry::MappedRegistry;
use chunk::block::{BlockPrototype, BlockSpreaderPrototype};
use chunk::layer::BlockLayerPrototype;
use crate::debug::{DebugCategory, DebugRendererImpl};
use crate::network::Token;
use crate::ty::block_pos::BlockPos;
use crate::ty::direction::Direction;
use crate::ty::id::Id;
use crate::ty::Offset;
use eyre::Result;
use crate::world::spread::Spreader;

pub mod chunk;
pub mod entity;
pub mod spread;


packet!(World(ServerBoundWorldPacket, ClientBoundWorldPacket));

pub enum ServerBoundWorldPacket {
	RequestChunk(ChunkPos),
	SetBlock(BlockPos, Id<BlockLayerPrototype>, Id<BlockPrototype>)
}

pub enum ClientBoundWorldPacket {
	Chunk(ChunkPos, Chunk),
	SetBlock(BlockPos, Id<BlockLayerPrototype>, Id<BlockPrototype>),
}

pub struct World {
	pub chunk: ChunkStorage,
	pub entity: EntityWorld,

	spreader: Spreader,
}

impl World {
	pub fn new(api: &Api, chunk: ChunkStorage) -> Result<World> {
		Ok(World {
			chunk,
			entity: EntityWorld::new(api)?,
			spreader: Spreader::new(api)
		})
	}

	pub fn tick(&mut self, api: &Api, debug: &mut impl DebugRendererImpl) {
		for (pos, layer_id,block_id) in self.spreader.tick(&mut self.chunk, debug) {
			self.place_block(api, pos, layer_id, block_id);
		};
		// Entity
		self.entity.tick(api, &self.chunk, debug);
	}

	pub fn packet(
		&mut self,
		api: &Api,
		token: Token,
		packet: ServerBoundWorldPacket,
		network: &mut ServerNetwork,
	) -> Result<()> {
		match packet {
			ServerBoundWorldPacket::RequestChunk(chunk_pos) => {
				if let Some(chunk) = self.chunk.get(chunk_pos) {
					network.send(token, ClientBoundWorldPacket::Chunk(chunk_pos, chunk.clone()))?;
				}
			}
			ServerBoundWorldPacket::SetBlock(pos, layer_id, block_id) => {
				self.place_block(api, pos, layer_id, block_id);
			}
		}
		Ok(())
	}

	pub fn place_block(&mut self, api: &Api, pos: BlockPos, layer_id: Id<BlockLayerPrototype>, block_id: Id<BlockPrototype>) {
		if let Some(chunk) = self.chunk.get_mut(pos.chunk) {
			// Layer
			let layer = chunk.layers.get_mut(layer_id);
			let prototype = api.carrier.block_layer.get(layer_id);

			// Block
			let block_prototype = prototype.registry.get(block_id);
			layer[pos.entry] = block_prototype.create(block_id);

			self.spreader.place_block(pos, layer_id, block_id);
		}
	}
}