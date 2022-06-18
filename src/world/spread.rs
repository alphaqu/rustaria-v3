use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoroshiro64Star;
use crate::{Api, ChunkStorage, draw_debug, TPS};
use crate::debug::{DebugCategory, DebugRendererImpl};
use crate::ty::block_pos::BlockPos;
use crate::ty::direction::Direction;
use crate::ty::id::Id;
use crate::ty::Offset;
use crate::world::chunk::block::BlockPrototype;
use crate::world::chunk::layer::BlockLayerPrototype;
use crate::world::chunk::spread::BlockSpreaderPrototype;

pub struct Spreader {
	rand: Xoroshiro64Star,
	active_spreads: HashMap<(BlockPos, Id<BlockLayerPrototype>), Id<BlockPrototype>>,
}

impl Spreader {
	pub fn new() -> Spreader {
		Spreader {
			rand: Xoroshiro64Star::seed_from_u64(69420),
			active_spreads: Default::default()
		}
	}

	pub fn tick(&mut self, api: &Api, chunks: &mut ChunkStorage, debug: &mut impl DebugRendererImpl) -> Vec<(BlockPos, Id<BlockLayerPrototype>, Id<BlockPrototype>)> {
		// Spread
		let mut remove = Vec::new();
		let mut spread = Vec::new();
		for ((pos, layer_id), block_id) in &self.active_spreads {
			let prototype = api.carrier.block_layer.get(*layer_id).blocks.get(*block_id);
			if let Some(prototype) = &prototype.spread {
				let result = prototype.tick_spread(*pos, *layer_id, chunks, &mut self.rand);
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

	pub fn place_block(&mut self, pos: BlockPos,  layer_id: Id<BlockLayerPrototype>, block_id: Id<BlockPrototype>, prototype: &BlockPrototype)  {
		self.active_spreads.remove(&(pos, layer_id));
		if prototype.spread.is_some() {
			self.active_spreads.insert((pos, layer_id), block_id);
		}
	}
}