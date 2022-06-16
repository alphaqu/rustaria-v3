use std::any::type_name;
use std::collections::HashMap;
use std::mem;
use mlua::prelude::{LuaError, LuaResult};
use mlua::{FromLua, Lua, Table, Value};
use tracing::debug;
use crate::api::prototype::Prototype;
use crate::api::registry::Registry;
use crate::ty::identifier::Identifier;

const DEFAULT_PRIORITY: f32 = 69420.0;

pub struct RegistryBuilder<P: Prototype> {
	entries: HashMap<Identifier, (f32, P)>
}

impl<P: Prototype> RegistryBuilder<P> {
	pub fn new() -> RegistryBuilder<P> {
		RegistryBuilder {
			entries: HashMap::new()
		}
	}

	pub fn register(&mut self, lua: &Lua, value: Table) -> LuaResult<()> {
		for value in value.pairs::<Value, P>() {
			let (identifier, prototype) = value?;
			let (identifier, priority) = match identifier {
				val @ Value::String(_) => {
					(Identifier::from_lua(val, lua)?, DEFAULT_PRIORITY)
				}
				Value::Table(table) => {
					(table.get::<_, Identifier>("name")?, table.get::<_, Option<f32>>("priority")?.unwrap_or(DEFAULT_PRIORITY))
				}
				_ => return Err(LuaError::external("Registry type must be Table { name = , priority = } or an identifier")),
			};

			debug!("{}: Added {identifier} {prototype:?}", type_name::<P>());
			self.entries.insert(identifier, (priority, prototype));
		}
		Ok(())
	}

	pub fn build(&mut self) -> Registry<P> {
		let mut out = HashMap::new();
		mem::swap(&mut self.entries, &mut out);
		Registry::new(out)
	}
}

use apollo::*;

#[lua_impl]
impl<P: Prototype> RegistryBuilder<P> {
	#[lua_method(register)]
	pub fn lua_register(&mut self, lua: &Lua, value: Table) -> LuaResult<()> {
		self.register(lua, value)
	}
}