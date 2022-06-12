use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::prototype::Prototype;

#[derive(Clone, Copy)]
pub struct Tile {
	pub id: Id<TilePrototype>,
	pub collision: bool,
}

pub struct TilePrototype {
	pub image: Option<Identifier>,
	pub collision: bool,
}

impl Prototype for TilePrototype {
	type Item = Tile;

	fn create(&self, id: Id<Self>) -> Self::Item {
		Tile { id, collision: self.collision }
	}
}