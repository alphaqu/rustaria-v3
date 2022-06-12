use std::fs::read;
use std::io;

use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::prototype::Prototype;
use crate::api::registry::Registry;
use crate::chunk::tile::TilePrototype;
use crate::entity::prototype::EntityPrototype;

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
	pub tile: Registry<TilePrototype>,
	pub entity: Registry<EntityPrototype>,
}

pub trait CarrierAccess<P: Prototype> {
	fn get(&self, id: Id<P>) -> &P;
	fn create(&self, id: Id<P>) -> P::Item {
		self.get(id).create(id)
	}
}

macro_rules! access {
    ($($FIELD:ident => $PROTOTYPE:ty),*) => {
	    $(
	    impl CarrierAccess<$PROTOTYPE> for Carrier {
			fn get(&self, id: Id<$PROTOTYPE>) -> &$PROTOTYPE {
				self.$FIELD.get(id)
			}
		}
	    )*
    };
}

access!(tile => TilePrototype, entity => EntityPrototype);