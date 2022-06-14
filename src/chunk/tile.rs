use mlua::{FromLua, Lua, LuaSerdeExt, Value};
use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::prototype::Prototype;
use crate::api::util;

#[derive(Clone, Copy)]
pub struct Tile {
	pub id: Id<TilePrototype>,
	pub collision: bool,
}

#[derive(Clone, Copy, Eq, PartialEq, serde::Deserialize)]
pub enum ConnectionType {
	// air
	Isolated,
	// tiles
	Connected,

}


pub struct TilePrototype {
	pub image: Option<Identifier>,
	pub collision: bool,
	pub connection_type: ConnectionType,
}

impl Prototype for TilePrototype {
	type Item = Tile;

	fn create(&self, id: Id<Self>) -> Self::Item {
		Tile { id, collision: self.collision }
	}
}

impl FromLua for TilePrototype {
	fn from_lua(lua_value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table = util::lua_table(lua_value)?;
		Ok(TilePrototype {
			image: table.get("image")?,
			collision: table.get("collision")?,
			connection_type: lua.from_value(table.get("connection_type")?)?,
		})
	}
}
