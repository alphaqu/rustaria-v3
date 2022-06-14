use mlua::{Lua, Table};
use std::fs::read;
use std::io;

use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::prototype::Prototype;
use crate::api::registry::Registry;
use crate::chunk::tile::TilePrototype;
use crate::entity::prototype::EntityPrototype;

pub mod id;
pub mod identifier;
pub mod prototype;
pub mod registry;
pub mod util;

pub struct Api {
    pub lua: Lua,
}

impl Api {
    pub fn reload(&self, resources: &Resources) -> Carrier {
        let result = resources
            .get_src(&Identifier::new("init.lua"))
            .expect("init where");
        self.lua.load(&result).exec().unwrap();

        Carrier {
            tile: self.extract("tile"),
            entity: self.extract("entity"),
        }
    }

    pub fn extract<P: Prototype>(&self, name: &str) -> Registry<P> {
        Registry::new(
            self.lua
                .globals()
                .get::<_, Table>(name)
                .unwrap()
                .pairs::<Identifier, P>()
                .map(|value| match value {
                    Ok(ok) => ok,
                    Err(error) => {
                        panic!("{error}");
                    }
                })
                .collect(),
        )
    }
}

pub struct Resources {}

impl Resources {
    pub fn get_asset(&self, location: &Identifier) -> io::Result<Vec<u8>> {
        if location.namespace != "rustaria" {
            panic!("cringe")
        }
        read(format!("./plugin/asset/{}", location.path))
    }

    pub fn get_src(&self, location: &Identifier) -> io::Result<Vec<u8>> {
        if location.namespace != "rustaria" {
            panic!("cringe")
        }
        read(format!("./plugin/src/{}", location.path))
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
