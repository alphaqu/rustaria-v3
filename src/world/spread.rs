use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoroshiro64Star;
use crate::{Api, ChunkStorage, draw_debug, TPS};
use crate::api::registry::MappedRegistry;
use crate::debug::{DebugCategory, DebugRendererImpl};
use crate::ty::block_pos::BlockPos;
use crate::ty::direction::Direction;
use crate::ty::id::Id;
use crate::ty::Offset;
use crate::world::chunk::block::{BlockPrototype, BlockSpreaderPrototype};
use crate::world::chunk::layer::BlockLayerPrototype;

pub struct Spreader {
	rand: Xoroshiro64Star,
	baked_spreaders: MappedRegistry<BlockLayerPrototype, MappedRegistry<BlockPrototype, Option<BlockSpreader>>>,
	active_spreads: HashMap<(BlockPos, Id<BlockLayerPrototype>), Id<BlockPrototype>>,
}

impl Spreader {
	pub fn new(api: &Api) -> Spreader {
		Spreader {
			rand: Xoroshiro64Star::seed_from_u64(69420),
			baked_spreaders: api.carrier.block_layer.map(|_, _, layer| {
				layer.registry.map(|_, _, prototype| {
					prototype.spread.as_ref().map(|value| {
						BlockSpreader::new(layer, value)
					})
				})
			}),
			active_spreads: Default::default()
		}

	}

	pub fn tick(&mut self, chunks: &mut ChunkStorage, debug: &mut impl DebugRendererImpl) -> Vec<(BlockPos, Id<BlockLayerPrototype>, Id<BlockPrototype>)> {
		// Spread
		let mut remove = Vec::new();
		let mut spread = Vec::new();
		for ((pos, layer_id), block_id) in &self.active_spreads {
			if let Some(spreader) = self.baked_spreaders.get(*layer_id).get(*block_id) {
				let result = spreader.tick(*pos, *layer_id, chunks, &mut self.rand);
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

		spread
	}

	pub fn place_block(&mut self, pos: BlockPos,  layer_id: Id<BlockLayerPrototype>, block_id: Id<BlockPrototype>)  {
		self.active_spreads.remove(&(pos, layer_id));
		if self.baked_spreaders.get(layer_id).get(block_id).is_some() {
			self.active_spreads.insert((pos, layer_id), block_id);
		}
	}
}

pub struct BlockSpreader {
	chance: f32,
	convert_table: HashMap<Id<BlockPrototype>, Id<BlockPrototype>>,
}

impl BlockSpreader {
	pub fn new(layer: &BlockLayerPrototype, block: &BlockSpreaderPrototype) -> BlockSpreader {
		let mut out = HashMap::new();
		for (from, to) in &block.convert_table {
			if let Some(from_id) = layer.registry.ident_to_id(from) {
				if let Some(to_id) = layer.registry.ident_to_id(to) {
					out.insert(from_id, to_id);
				}
			}
		}

		BlockSpreader {
			chance: block.chance,
			convert_table: out
		}
	}

	pub fn tick(&self, pos: BlockPos, layer_id: Id<BlockLayerPrototype>, chunks: &mut ChunkStorage, rand: &mut Xoroshiro64Star) -> SpreadResult {
		if (self.chance / TPS as f32) >= rand.gen_range(0.0..1.0) as f32 {
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
				keep,
				spread
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
