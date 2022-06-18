use std::any::Any;
use std::collections::HashMap;
use crate::api::luna::lib::registry_builder::{DynRegistryBuilder, LuaRegistryBuilder, RegistryBuilder};
use crate::api::Carrier;
use crate::world::chunk::layer::BlockLayerPrototype;
use crate::world::entity::prototype::EntityPrototype;
use mlua::prelude::LuaResult;
use std::time::Instant;
use eyre::WrapErr;
use mlua::{AnyUserData, Error, Lua};
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

    pub fn register_builder<P: LuaPrototype>(&mut self) {
        self.builders.insert(P::get_name().to_string(), LuaRegistryBuilder::new(RegistryBuilder::<P>::new()));
    }

    pub fn build_registry<P: LuaPrototype>(&mut self, lua: &Lua, hasher: &mut Hasher) -> eyre::Result<Registry<P>> {
        self.builders
            .remove(P::get_name())
            .expect("Registry unregistered")
            .build(lua, hasher).wrap_err_with(|| format!("Failed to create registry for {}s", P::get_name()))
    }
}

use crate::api::prototype::{LuaPrototype, Prototype};
use crate::api::registry::Registry;
use apollo::*;
use crate::util::blake3::Hasher;

#[lua_impl]
impl Stargate {
    #[lua_method]
    pub fn __index(&mut self, name: String) -> LuaResult<&mut LuaRegistryBuilder> {
        self.builders.get_mut(&name).ok_or_else(|| Error::external(format!("Registry {} does not exist in this context.", name)))
    }
}
