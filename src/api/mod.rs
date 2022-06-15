use mlua::{Lua, Table};
use std::fs::read;
use std::io;
use std::sync::Arc;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::ty::identifier::Identifier;
use crate::api::prototype::Prototype;
use crate::api::registry::Registry;
use crate::chunk::block::BlockLayerPrototype;
use crate::entity::prototype::EntityPrototype;
use crate::multi_deref_fields;
use crate::ty::MultiDeref;
use eyre::Result;
use crate::api::luna::glue::Glue;
use crate::api::luna::lib::stargate::Stargate;
use crate::api::luna::Luna;

pub mod prototype;
pub mod registry;
pub mod util;
pub mod luna;

pub struct Api {
    pub luna: Luna,
    pub carrier: Carrier,
    pub resources: Resources,
    pub thread_pool: Arc<ThreadPool>,
}

impl Api {
    pub fn new() -> Result<Api> {
        let resources = Resources {};
        Ok( Api {
           luna: Luna::new(&resources)?,
           carrier: Carrier {
               block_layer: Registry::empty(),
               entity: Registry::empty()
           },
           resources,
           thread_pool: Arc::new(ThreadPoolBuilder::new().build()?)
       })
    }

    pub fn reload(&mut self) -> Result<()> {
        // Prepare for reload
        let mut stargate = Stargate::new();
        {
            let glue = Glue::new(&mut stargate);
            self.luna.lua.globals().set("reload", glue.clone())?;

            // TODO plugins
            // reload our mod
            let location = Identifier::new("init.lua");
            let data = self.resources.get_src(&location)?;
            let chunk = self.luna.load(&location, &data)?;
            chunk.exec()?;
        }

        self.carrier = stargate.build();
        Ok(())
    }
}

#[derive(Clone)]
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
    pub block_layer: Registry<BlockLayerPrototype>,
    pub entity: Registry<EntityPrototype>,
}

multi_deref_fields!(Carrier {
    block_layer: Registry<BlockLayerPrototype>,
    entity: Registry<EntityPrototype>
});