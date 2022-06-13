use std::ops::{Index, IndexMut};

use tile::Tile;

use crate::ty::chunk_entry_pos::ChunkEntryPos;

pub mod tile;
pub mod storage;

pub const CHUNK_SIZE: usize = 16;

#[derive(Clone)]
pub struct Chunk {
	pub tile: ChunkLayer<Tile>,
}

// Layer
#[derive(Clone)]
pub struct ChunkLayer<T: Clone> {
	pub data: [[T; CHUNK_SIZE]; CHUNK_SIZE],
}

impl<T: Clone> ChunkLayer<T>  {
	pub fn entries(&self, mut func: impl FnMut(ChunkEntryPos, &T)) {
		for y in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				func(ChunkEntryPos::new(x as u8, y as u8), &self.data[y][x]);
			}
		}
	}

	pub fn entries_mut(&mut self, mut func: impl FnMut(ChunkEntryPos, &mut T)) {
		for y in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				func(ChunkEntryPos::new(x as u8, y as u8), &mut self.data[y][x]);
			}
		}
	}

	pub fn map<O: Clone + Copy>(&self, default: O, mut func: impl FnMut(&T) -> O) -> ChunkLayer<O> {
		let mut out = ChunkLayer::new_copy(default);
		for y in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				out.data[y][x] = func(&self.data[y][x]);
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

impl<T: Clone> Index<ChunkEntryPos> for ChunkLayer<T> {
	type Output = T;

	fn index(&self, index: ChunkEntryPos) -> &Self::Output {
		&self.data[index.y() as usize][index.x() as usize]
	}
}

impl<T: Clone> IndexMut<ChunkEntryPos> for ChunkLayer<T> {
	fn index_mut(&mut self, index: ChunkEntryPos) -> &mut Self::Output {
		&mut self.data[index.y() as usize][index.x() as usize]
	}
}
