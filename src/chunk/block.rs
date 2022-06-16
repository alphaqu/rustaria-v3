use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use mlua::{FromLua, Function, Lua, LuaSerdeExt, Table, Value};
use tracing::{info_span};
use crate::api::luna::lib::registry_builder::RegistryBuilder;
use crate::ty::id::Id;
use crate::ty::identifier::Identifier;
use crate::api::prototype::{FactoryPrototype, Prototype};
use crate::api::registry::Registry;
use crate::api::util::lua_table;
use crate::chunk::ConnectionType;

#[derive(Clone, Copy)]
pub struct Block {
	pub id: Id<BlockPrototype>,
	pub collision: bool,
}

#[derive(Debug)]
pub struct BlockPrototype {
	pub image: Option<Identifier>,
	pub collision: bool,
	pub connection_type: ConnectionType,
}

impl Prototype for BlockPrototype {
}

impl FactoryPrototype for BlockPrototype {
	type Item = Block;
	fn create(&self, id: Id<Self>) -> Self::Item {
		Block { id, collision: self.collision }
	}
}

impl FromLua for BlockPrototype {
	fn from_lua(lua_value: Value, lua: &Lua) -> mlua::Result<Self> {
		let _span = info_span!("FromLua ChunkEntryPrototype").entered();

		let table = lua_table(lua_value)?;
		Ok(BlockPrototype {
			image: table.get("image")?,
			collision: table.get("collision")?,
			connection_type: lua.from_value(table.get("connection_type")?)?,
		})
	}
}

/// TODO sprite mappings and stuff
pub struct BlockLayerPrototype {
	pub registry: Registry<BlockPrototype>,
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