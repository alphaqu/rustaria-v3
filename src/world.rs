use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoroshiro64Star;
use crate::{Api, Chunk, ChunkPos, ChunkStorage, draw_debug, EntityWorld, packet, ServerNetwork};
use crate::api::prototype::FactoryPrototype;
use crate::api::registry::MappedRegistry;
use crate::chunk::block::{BlockPrototype, BlockSpreader};
use crate::chunk::layer::BlockLayerPrototype;
use crate::debug::{DebugCategory, DebugRendererImpl};
use crate::network::Token;
use crate::ty::block_pos::BlockPos;
use crate::ty::direction::Direction;
use crate::ty::id::Id;
use crate::ty::Offset;
use eyre::Result;
use tracing::{debug, trace};


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
	pub rand: Xoroshiro64Star,

	// Spread
	baked_spreaders: MappedRegistry<BlockLayerPrototype, MappedRegistry<BlockPrototype, Option<BakedBlockSpreader>>>,
	active_spreads: HashMap<(BlockPos, Id<BlockLayerPrototype>), Id<BlockPrototype>>,
}

impl World {
	pub fn new(api: &Api, chunk: ChunkStorage) -> Result<World> {
		Ok(World {
			chunk,
			entity: EntityWorld::new(api)?,
			rand: Xoroshiro64Star::seed_from_u64(69420),
			baked_spreaders: api.carrier.block_layer.map(|_, layer| {
				layer.registry.map(|_, prototype| {
					prototype.spread.as_ref().map(|value| {
						BakedBlockSpreader::new(layer, value)
					})
				})
			}),
			active_spreads: Default::default()
		})
	}

	pub fn tick(&mut self, api: &Api, debug: &mut impl DebugRendererImpl) {
		// Spread
		let mut remove = Vec::new();
		let mut spread = Vec::new();
		for ((pos, layer_id), block_id) in &self.active_spreads {
			if let Some(spreader) = self.baked_spreaders.get(*layer_id).get(*block_id) {
				let result = spreader.tick(*pos, *layer_id, &mut self.chunk, &mut self.rand);
				if let Some(result) = result.spread {
					draw_debug!(debug, DebugCategory::TileSpread, result.0, 0xfcfcfa, 10.0, 1.0);
					spread.push((result.0, *layer_id, result.1));
				}

				if !result.keep {
					draw_debug!(debug, DebugCategory::TileSpread, *pos, 0xbf5570, 1.0, 1.0);
					remove.push((*pos, *layer_id));
				} else {
					draw_debug!(debug, DebugCategory::TileSpread, *pos, 0x5b595c);
				}
			}
		}

		for pos in remove {
			self.active_spreads.remove(&pos);
		}

		for (pos, layer_id, block_id) in spread {
			self.place_block(api, pos, layer_id, block_id)
		}

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

			self.active_spreads.remove(&(pos, layer_id));
			if self.baked_spreaders.get(layer_id).get(block_id).is_some() {
				self.active_spreads.insert((pos, layer_id), block_id);
			}
		}
	}
}

pub struct BakedBlockSpreader {
	chance: f32,
	convert_table: HashMap<Id<BlockPrototype>, Id<BlockPrototype>>,
}

impl BakedBlockSpreader {
	pub fn new(layer: &BlockLayerPrototype, block: &BlockSpreader) -> BakedBlockSpreader {
		let mut out = HashMap::new();
		for (from, to) in &block.convert_table {
			if let Some(from_id) = layer.registry.identifier_to_id(from) {
				if let Some(to_id) = layer.registry.identifier_to_id(to) {
					out.insert(from_id, to_id);
				}
			}
		}

		BakedBlockSpreader {
			chance: block.chance,
			convert_table: out
		}
	}

	pub fn tick(&self, pos: BlockPos, layer_id: Id<BlockLayerPrototype>, chunks: &mut ChunkStorage, rand: &mut Xoroshiro64Star) -> SpreadResult {
		if self.chance >= rand.gen_range(0.0..1.0) as f32 {
			let mut spread = None;
			let mut keep = false;
			for dir in Direction::values() {
				if let Some(pos) = pos.checked_offset(dir.offset()) {
					if let Some(chunk) = chunks.get_mut(pos.chunk) {
						let layer = chunk.layers.get_mut(layer_id);
						let id = layer[pos.entry].id;
						if let Some(next_id) = self.convert_table.get(&id) {
							if spread.is_some() {
								keep = true;
								break;
							}

							spread = Some((pos, *next_id));
						}
					}
				}
			}

			// we could not spread in the 4 directions
			SpreadResult {
				keep: keep,
				spread: spread
			}
		} else {
			SpreadResult {
				keep: true,
				spread: None
			}
		}

	}
}

pub struct SpreadResult {
	keep: bool,
	spread: Option<(BlockPos, Id<BlockPrototype>)>
}