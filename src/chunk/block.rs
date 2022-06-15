use mlua::{FromLua, Function, Lua, LuaSerdeExt, Table, Value};
use tracing::{info_span};
use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::prototype::{FactoryPrototype, KernelId, Prototype};
use crate::api::registry::Registry;
use crate::api::util::lua_table;
use crate::chunk::ConnectionType;

#[derive(Clone, Copy)]
pub struct Block {
	pub id: Id<BlockPrototype>,
	pub collision: bool,
}

impl KernelId<BlockPrototype> for Block {
	fn get_id(&self) -> Id<BlockPrototype> {
		self.id
	}
}

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
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		let _span = info_span!("FromLua ChunkLayerPrototype").entered();
		let table = lua_table(value)?;
		let entries: Table = table.get("entries")?;

		let mut out = Vec::new();
		for value in entries.pairs::<Identifier, BlockPrototype>() {
			out.push(value?);
		}

		Ok(BlockLayerPrototype {
			registry: Registry::new(out),
			get_uv: table.get("get_uv")?,
			get_rect: table.get("get_rect")?,
			collision: table.get("collision")?,
		})
	}
}

impl Prototype for BlockLayerPrototype {
}