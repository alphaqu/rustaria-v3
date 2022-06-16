use mlua::{FromLua, Lua, Value};
use std::fmt::{Debug, Formatter};
use crate::api::luna::lib::registry_builder::RegistryBuilder;
use crate::api::prototype::{Prototype};
use crate::api::registry::Registry;
use crate::api::util::lua_table;
use crate::world::chunk::block::BlockPrototype;
use crate::ty::identifier::Identifier;

pub struct BlockLayerPrototype {
	pub registry: Registry<BlockPrototype>,
	pub default: Identifier,
	pub collision: bool,
}

impl FromLua for BlockLayerPrototype {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table = lua_table(value)?;
		let mut builder = RegistryBuilder::new();
		builder.register(lua,  table.get("entries")?)?;

		Ok(BlockLayerPrototype {
			registry: builder.build(),
			default: table.get("default")?,
			collision: table.get("collision")?,
		})
	}
}

impl Debug for BlockLayerPrototype {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("{} entries", self.registry.len()))
	}
}

impl Prototype for BlockLayerPrototype {
	fn get_name() -> &'static str {
		"block_layer"
	}
}