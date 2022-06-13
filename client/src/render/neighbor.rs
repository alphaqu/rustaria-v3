use std::ops::{Deref, DerefMut};
use rustaria::chunk::{CHUNK_SIZE, ChunkLayer};
use rustaria::chunk::tile::ConnectionType;
use rustaria::ty::chunk_entry_pos::ChunkEntryPos;
use rustaria::ty::direction::{Direction, DirMap};

#[derive(Clone)]
pub(crate) struct NeighborMatrixBuilder {
	matrix: ChunkLayer<DirMap<ConnectionType>>,
	layer: ChunkLayer<ConnectionType>,
}

impl NeighborMatrixBuilder {
	pub fn new(layer: ChunkLayer<ConnectionType>) -> NeighborMatrixBuilder {
		NeighborMatrixBuilder {
			matrix: ChunkLayer {
				data: [[DirMap::new([ConnectionType::Isolated; 4]); 16]; 16]
			},
			layer,
		}
	}

	pub fn compile_internal(&mut self) {
		for y in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				if self.layer.data[y][x] == ConnectionType::Connected {
					if y != CHUNK_SIZE - 1 && ConnectionType::Connected == self.layer.data[y + 1][x]
					{
						self.matrix.data[y][x][Direction::Up] = ConnectionType::Connected;
						self.matrix.data[y + 1][x][Direction::Down] = ConnectionType::Connected;
					}

					if x != CHUNK_SIZE - 1 && ConnectionType::Connected == self.layer.data[y][x + 1]
					{
						self.matrix.data[y][x][Direction::Right] = ConnectionType::Connected;
						self.matrix.data[y][x + 1][Direction::Left] = ConnectionType::Connected;
					}
				}
			}
		}
	}

	pub fn compile_edge(&mut self, dir: Direction, neighbor: &ChunkLayer<ConnectionType>) {
		let c = CHUNK_SIZE as u8 - 1;
		let y_offset = dir.offset_y().max(0) as u8 * c;
		let x_offset = dir.offset_x().max(0) as u8 * c;
		let x_length = dir.offset_x().unsigned_abs() * c;
		let y_length = dir.offset_y().unsigned_abs() * c;
		for y in y_offset..=x_length + y_offset {
			for x in x_offset..=y_length + x_offset {
				let neighbor_sub_pos = ChunkEntryPos::new(x, y).euclid_offset(dir.offset());
				let tile = self.layer.data[y as usize][x as usize];
				let neighbor_tile = neighbor[neighbor_sub_pos];
				let ty = if tile == ConnectionType::Connected
					&& neighbor_tile == ConnectionType::Connected
				{
					ConnectionType::Connected
				} else {
					ConnectionType::Isolated
				};

				self.matrix.data[y as usize][x as usize][dir] = ty;
			}
		}
	}

	pub fn export(self) -> ChunkLayer<SpriteConnectionKind> {
		ChunkLayer {
			data: self.matrix
				.data
				.map(|value| value.map(SpriteConnectionKind::new))
		}
	}
}

#[derive(Clone, Copy)]
pub(crate) struct NeighborCell {
	edges: DirMap<ConnectionType>,
}

impl Deref for NeighborCell {
	type Target = DirMap<ConnectionType>;

	fn deref(&self) -> &Self::Target {
		&self.edges
	}
}

impl DerefMut for NeighborCell {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.edges
	}
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum SpriteConnectionKind {
	Solid,
	Lonely,
	Vertical,
	Horizontal,
	CapTop,
	CapDown,
	CapLeft,
	CapRight,
	CornerTopLeft,
	CornerTopRight,
	CornerDownLeft,
	CornerDownRight,
	WallTop,
	WallDown,
	WallLeft,
	WallRight,
}

impl SpriteConnectionKind {
	pub fn new(values: DirMap<ConnectionType>) -> SpriteConnectionKind {
		use ConnectionType::{Connected, Isolated};
		match (
			values[Direction::Up],
			values[Direction::Down],
			values[Direction::Left],
			values[Direction::Right],
		) {
			(Connected, Connected, Connected, Connected) => SpriteConnectionKind::Solid,
			(Isolated, Isolated, Isolated, Isolated) => SpriteConnectionKind::Lonely,
			(Connected, Connected, Isolated, Isolated) => SpriteConnectionKind::Vertical,
			(Isolated, Isolated, Connected, Connected) => SpriteConnectionKind::Horizontal,
			(Isolated, Connected, Connected, Connected) => SpriteConnectionKind::WallTop,
			(Isolated, Connected, Isolated, Isolated) => SpriteConnectionKind::CapTop,
			(Isolated, Connected, Isolated, Connected) => SpriteConnectionKind::CornerTopLeft,
			(Isolated, Connected, Connected, Isolated) => SpriteConnectionKind::CornerTopRight,
			(Connected, Isolated, Connected, Connected) => SpriteConnectionKind::WallDown,
			(Connected, Isolated, Isolated, Isolated) => SpriteConnectionKind::CapDown,
			(Connected, Isolated, Isolated, Connected) => SpriteConnectionKind::CornerDownLeft,
			(Connected, Isolated, Connected, Isolated) => SpriteConnectionKind::CornerDownRight,
			(Connected, Connected, Isolated, Connected) => SpriteConnectionKind::WallLeft,
			(Isolated, Isolated, Isolated, Connected) => SpriteConnectionKind::CapLeft,
			(Connected, Connected, Connected, Isolated) => SpriteConnectionKind::WallRight,
			(Isolated, Isolated, Connected, Isolated) => SpriteConnectionKind::CapRight,
		}
	}
}
