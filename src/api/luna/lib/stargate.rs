use std::any::Any;
use std::collections::HashMap;
use crate::api::luna::lib::registry_builder::{DynRegistryBuilder, LuaRegistryBuilder, RegistryBuilder};
use crate::api::Carrier;
use crate::world::chunk::layer::BlockLayerPrototype;
use crate::world::entity::prototype::EntityPrototype;
use mlua::prelude::LuaResult;
use std::time::Instant;
use mlua::{AnyUserData, Error};
use tracing::info;
use type_map::TypeMap;

/// Creates a carrier
pub struct Stargate {
    pub start: Instant,
    pub builders: HashMap<String, LuaRegistryBuilder>,
}

impl Stargate {
    pub fn new() -> Stargate {
        Stargate {
            start: Instant::now(),
            builders: Default::default(),
        }
    }

    pub fn register_builder<P: Prototype>(&mut self) {
        self.builders.insert(P::get_name().to_string(), LuaRegistryBuilder::new(RegistryBuilder::<P>::new()));
    }

    pub fn build_registry<P: Prototype>(&mut self) -> Registry<P> {
        self.builders
            .remove(P::get_name())
            .expect("Registry unregistered")
            .build()
    }

    // *Carrier has arrived!* - Carrier
    //pub fn build(&mut self) -> Carrier {
    //    info!(
    //        "Rustaria reloaded in {}ms",
    //        self.start.elapsed().as_secs_f32() / 1000.0
    //    );
    //    Carrier {
    //        block_layer: self.block_layer.build(),
    //        entity: self.entity.build(),
    //    }
    //}
}

use crate::api::prototype::Prototype;
use crate::api::registry::Registry;
use apollo::*;

#[lua_impl]
impl Stargate {
    #[lua_method]
    pub fn __index(&mut self, name: String) -> LuaResult<&mut LuaRegistryBuilder> {
        self.builders.get_mut(&name).ok_or_else(|| Error::external(format!("Registry {} does not exist in this context.", name)))
    }
}
