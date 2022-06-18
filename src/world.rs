use eyre::Result;
use rand::{Rng, SeedableRng};

use chunk::block::BlockPrototype;
use chunk::layer::BlockLayerPrototype;

use crate::{Api, Chunk, ChunkPos, ChunkStorage, EntityWorld, packet, ServerNetwork};
use crate::api::prototype::FactoryPrototype;
use crate::debug::DebugRendererImpl;
use crate::network::Token;
use crate::ty::block_pos::BlockPos;
use crate::ty::id::Id;
use crate::ty::Offset;
use crate::world::spread::Spreader;

pub mod chunk;
pub mod entity;
pub mod spread;


packet!(World(ServerBoundWorldPacket, ClientBoundWorldPacket));

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerBoundWorldPacket {
	RequestChunk(ChunkPos),
	SetBlock(BlockPos, Id<BlockLayerPrototype>, Id<BlockPrototype>)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientBoundWorldPacket {
	Chunk(ChunkPos, Chunk),
	SetBlock(BlockPos, Id<BlockLayerPrototype>, Id<BlockPrototype>),
}

pub struct World {
	pub chunks: ChunkStorage,
	pub entities: EntityWorld,

	spreader: Spreader,
}

impl World {
	pub fn new(api: &Api, chunk: ChunkStorage) -> Result<World> {
		Ok(World {
			chunks: chunk,
			entities: EntityWorld::new(api)?,
			spreader: Spreader::new()
		})
	}

	pub fn tick(&mut self, api: &Api, debug: &mut impl DebugRendererImpl) {
		for (pos, layer_id,block_id) in self.spreader.tick(api, &mut self.chunks, debug) {
			self.place_block(api, pos, layer_id, block_id);
		};
		// Entity
		self.entities.tick(api, &self.chunks, debug);
	}

	pub fn place_block(&mut self, api: &Api, pos: BlockPos, layer_id: Id<BlockLayerPrototype>, block_id: Id<BlockPrototype>) {
		if let Some(chunk) = self.chunks.get_mut(pos.chunk) {
			// Layer
			let layer = chunk.layers.get_mut(layer_id);
			let prototype = api.carrier.block_layer.get(layer_id);

			// Block
			let block_prototype = prototype.blocks.get(block_id);
			layer[pos.entry] = block_prototype.create(block_id);

			self.spreader.place_block(pos, layer_id, block_id, block_prototype);
		}
	}

	pub(crate) fn packet(
		&mut self,
		api: &Api,
		token: Token,
		packet: ServerBoundWorldPacket,
		network: &mut ServerNetwork,
	) -> Result<()> {
		match packet {
			ServerBoundWorldPacket::RequestChunk(chunk_pos) => {
				if let Some(chunk) = self.chunks.get(chunk_pos) {
					network.send(token, ClientBoundWorldPacket::Chunk(chunk_pos, chunk.clone()))?;
				}
			}
			ServerBoundWorldPacket::SetBlock(pos, layer_id, block_id) => {
				self.place_block(api, pos, layer_id, block_id);
			}
		}
		Ok(())
	}
}