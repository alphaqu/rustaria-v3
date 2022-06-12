use crate::chunk::CHUNK_SIZE;

/// World Space
pub struct WS;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ChunkEntryPos {
	pub x: u8,
	pub y: u8,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ChunkPos {
	pub x: u32,
	pub y: u32,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct WorldPos {
	pub chunk: ChunkPos,
	pub entry: ChunkEntryPos,
}

impl WorldPos {
	pub fn new(chunk: ChunkPos, entry: ChunkEntryPos) -> WorldPos {
		WorldPos { chunk, entry }
	}

	pub fn get_x(&self) -> u32 {
		(self.chunk.x * CHUNK_SIZE as u32) + self.entry.x as u32
	}

	pub fn get_y(&self) -> u32 {
		(self.chunk.y * CHUNK_SIZE as u32) + self.entry.y as u32
	}
}

impl ChunkPos {
	pub fn zero() -> ChunkPos {
		ChunkPos {
			x: 0,
			y: 0
		}
	}
}