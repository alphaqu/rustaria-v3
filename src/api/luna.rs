pub mod lib;
pub mod glue;
pub mod table;

use std::fmt::Write;
use eyre::{Result, WrapErr};
use mlua::{Chunk, Lua, Table};
use mlua::prelude::LuaError;
use tracing::debug;
use crate::api::{ResourceKind, Plugins};
use crate::ty::identifier::Identifier;

/// Holds everything luna.
pub struct Luna {
	pub lua: Lua
}

impl Luna {
	pub fn new(resources: &Plugins) -> Result<Luna> {
		let lua = Lua::new();
		lib::register(&lua).wrap_err("Failed to register lua")?;

		let globals = lua.globals();
		let package: Table = globals.get("package")?;
		let searchers: Table = package.get("loaders")?;

		let resources = resources.clone();
		searchers.raw_insert(
			2,
			lua.create_function(move |lua, mut location: Identifier| {
				location.path.write_str(".lua").map_err(LuaError::external)?;
				debug!(target: "luna::loading", "Loading {}", location);
				let data = resources.get_resource(ResourceKind::Source, &location)?;
				Self::load_inner(lua, &location, &data)?.into_function()
			})?,
		)?;

		Ok(Luna {
			lua
		})
	}

	pub fn load<'a>(&self, name: &Identifier, data: &'a [u8]) -> mlua::Result<Chunk<'a>> {
		Self::load_inner(&self.lua, name ,data)
	}

	fn load_inner<'a>(lua: &Lua, name: &Identifier, data: &'a [u8]) -> mlua::Result<Chunk<'a>> {
		lua.load(data).set_name(format!("{name}"))
	}
}

pub struct LunaFile {
	data: Vec<u8>,

}