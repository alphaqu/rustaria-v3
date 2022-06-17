use std::ops::{Index, IndexMut};


use block::Block;
use crate::api::registry::MappedRegistry;
use layer::BlockLayerPrototype;

use crate::ty::block_layer_pos::BlockLayerPos;


pub mod block;
pub mod layer;
pub mod storage;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_F32: f32 = CHUNK_SIZE as f32;

pub type BlockLayer = ChunkLayer<Block>;

#[derive(Clone)]
pub struct Chunk {
	pub layers: MappedRegistry<BlockLayerPrototype, BlockLayer>,
}

// Layer
#[derive(Clone)]
pub struct ChunkLayer<T: Clone> {
	pub data: [[T; CHUNK_SIZE]; CHUNK_SIZE],
}

impl<T: Clone> ChunkLayer<T>  {
	pub fn entries(&self, mut func: impl FnMut(BlockLayerPos, &T)) {
		for y in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				func(BlockLayerPos::new(x as u8, y as u8), &self.data[y][x]);
			}
		}
	}

	pub fn entries_mut(&mut self, mut func: impl FnMut(BlockLayerPos, &mut T)) {
		for y in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				func(BlockLayerPos::new(x as u8, y as u8), &mut self.data[y][x]);
			}
		}
	}

	pub fn map<O: Clone + Copy>(&self, default: O, mut func: impl FnMut(&T) -> Option<O>) -> ChunkLayer<O> {
		let mut out = ChunkLayer::new_copy(default);
		for y in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				if let Some(value) = func(&self.data[y][x]) {
					out.data[y][x] = value;
				}
			}
		}

		out
	}
}

impl<T: Clone + Copy> ChunkLayer<T> {
	pub fn new_copy(value: T) -> Self {
		ChunkLayer { data: [[value; CHUNK_SIZE]; CHUNK_SIZE] }
	}
}

impl<T: Clone> Index<BlockLayerPos> for ChunkLayer<T> {
	type Output = T;

	fn index(&self, index: BlockLayerPos) -> &Self::Output {
		&self.data[index.y() as usize][index.x() as usize]
	}
}

impl<T: Clone> IndexMut<BlockLayerPos> for ChunkLayer<T> {
	fn index_mut(&mut self, index: BlockLayerPos) -> &mut Self::Output {
		&mut self.data[index.y() as usize][index.x() as usize]
	}
}


#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize)]
pub enum ConnectionType {
	// air
	Isolated,
	// tiles
	Connected,
}