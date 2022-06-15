use mlua::{Lua, Table};
use std::fs::read;
use std::io;
use std::sync::Arc;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::api::identifier::Identifier;
use crate::api::prototype::{Prototype};
use crate::api::registry::Registry;
use crate::chunk::block::{BlockLayerPrototype};
use crate::entity::prototype::EntityPrototype;
use crate::multi_deref_fields;
use crate::ty::MultiDeref;
use eyre::Result;

pub mod id;
pub mod identifier;
pub mod prototype;
pub mod registry;
pub mod util;

pub struct Api {
    pub lua: Lua,
    pub carrier: Carrier,
    pub resources: Resources,
    pub thread_pool: Arc<ThreadPool>,
}

impl Api {
    pub fn new() -> Result<Api> {
       Ok( Api {
           lua: Lua::new(),
           carrier: Carrier {
               block_layers: Registry::new(vec![]),
               entity: Registry::new(vec![])
           },
           resources: Resources {},
           thread_pool: Arc::new(ThreadPoolBuilder::new().build()?)
       })
    }

    pub fn reload(&mut self) {
        let result = self.resources
            .get_src(&Identifier::new("init.lua"))
            .expect("init where");
        self.lua.load(&result).exec().unwrap();

        self.carrier = Carrier {
            block_layers: self.extract("chunk_layers"),
            entity: self.extract("entity"),
        };
    }

    fn extract<P: Prototype>(&self, name: &str) -> Registry<P> {
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
    pub block_layers: Registry<BlockLayerPrototype>,
    pub entity: Registry<EntityPrototype>,
}


multi_deref_fields!(Carrier {
    block_layers: Registry<BlockLayerPrototype>,
    entity: Registry<EntityPrototype>
});