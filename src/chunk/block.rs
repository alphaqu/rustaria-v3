use std::collections::HashMap;
use std::fmt::Debug;
use mlua::{FromLua, Lua, LuaSerdeExt, Value};
use tracing::{debug, info_span, trace};
use crate::ty::id::Id;
use crate::ty::identifier::Identifier;
use crate::api::prototype::{FactoryPrototype, Prototype};
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
	pub spread: Option<BlockSpreader>,
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
		trace!("FromLua ChunkEntryPrototype");

		let table = lua_table(lua_value)?;
		Ok(BlockPrototype {
			image: table.get("image")?,
			collision: table.get("collision")?,
			connection_type: lua.from_value(table.get("connection_type")?)?,
			spread: table.get("spread")?
		})
	}
}

#[derive(Debug)]
pub struct BlockSpreader {
	pub chance: f32,
	pub convert_table: HashMap<Identifier, Identifier>,
}

impl FromLua for BlockSpreader {
	fn from_lua(lua_value: Value, _: &Lua) -> mlua::Result<Self> {
		trace!("FromLua BlockSpread");

		let table = lua_table(lua_value)?;
		Ok(BlockSpreader {
			chance: table.get("chance")?,
			convert_table: table.get("convert_table")?
		})
	}
}