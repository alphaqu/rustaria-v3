use std::collections::HashSet;
use std::collections::hash_set::Iter;
use std::mem;
use crate::{Chunk, ChunkPos};

#[derive(Clone)]
pub struct ChunkStorage {
	width: u32,
	height: u32,
	chunks: Vec<Chunk>,
	dirty: HashSet<ChunkPos>,
}

impl ChunkStorage {
	pub fn new(width: u32, height: u32, chunks: Vec<Chunk>) -> Option<ChunkStorage> {
		if width  as usize * height as usize != chunks.len() {
			return None;
		}

		Some(ChunkStorage {
			width,
			height,
			chunks,
			dirty: Default::default()
		})
	}

	pub fn get(&self, pos: ChunkPos) -> Option<&Chunk> {
		let idx = self.get_idx(pos)?;
		Some(&self.chunks[idx])
	}

	pub fn get_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
		let idx = self.get_idx(pos)?;
		self.dirty.insert(pos);
		Some(&mut self.chunks[idx])
	}

	pub fn insert(&mut self, pos: ChunkPos, mut chunk: Chunk) -> Option<Chunk> {
		let idx = self.get_idx(pos)?;
		mem::swap(&mut self.chunks[idx], &mut chunk);
		self.dirty.insert(pos);
		Some(chunk)
	}

	pub fn get_dirty(&self) -> Iter<'_, ChunkPos> {
		self.dirty.iter()
	}

	pub fn reset_dirty(&mut self) {
		self.dirty.clear();
	}

	fn get_idx(&self, pos: ChunkPos) -> Option<usize> {
		if pos.x >= self.width || pos.y >= self.height	{
			return None;
		}

		Some(pos.x as usize  + (pos.y as usize * self.width as usize))
	}
}

