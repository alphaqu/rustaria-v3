use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::prototype::Prototype;

#[derive(Clone, Copy)]
pub struct Tile {
	pub id: Id<TilePrototype>,
}

pub struct TilePrototype {
	pub image: Identifier,
}

impl Prototype for TilePrototype {
	type Item = Tile;

	fn create(&self, id: Id<Self>) -> Self::Item {
		Tile { id }
	}
}