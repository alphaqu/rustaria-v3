use mlua::{FromLua, Function, Lua, Value};
use std::fmt::{Debug, Formatter};
use tracing::info_span;
use crate::api::luna::lib::registry_builder::RegistryBuilder;
use crate::api::prototype::{Prototype};
use crate::api::registry::Registry;
use crate::api::util::lua_table;
use crate::chunk::block::BlockPrototype;
use crate::ty::identifier::Identifier;

pub struct BlockLayerPrototype {
	pub registry: Registry<BlockPrototype>,
	pub default: Identifier,
	pub get_uv: Function,
	pub get_rect: Function,
	pub collision: bool,
}

impl FromLua for BlockLayerPrototype {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let _span = info_span!("FromLua ChunkLayerPrototype").entered();
		let table = lua_table(value)?;


		let mut builder = RegistryBuilder::new();
		builder.register(lua,  table.get("entries")?)?;

		Ok(BlockLayerPrototype {
			registry: builder.build(),
			default: table.get("default")?,
			get_uv: table.get("get_uv")?,
			get_rect: table.get("get_rect")?,
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
}