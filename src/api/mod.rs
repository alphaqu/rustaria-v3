use std::fs::read;
use std::io;
use crate::api::identifier::Identifier;
use crate::api::registry::Registry;
use crate::chunk::tile::TilePrototype;

pub mod registry;
pub mod prototype;
pub mod identifier;
pub mod id;

pub struct Assets {

}

impl Assets {
	pub fn get(&self, location: &Identifier) -> io::Result<Vec<u8>> {
		if location.namespace != "rustaria" {
			panic!("cringe")
		}
		read(format!("./plugin/asset/{}", location.path))
	}
}

pub struct Carrier {
	pub tile: Registry<TilePrototype>
}
