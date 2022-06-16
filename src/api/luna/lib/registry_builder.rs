use std::any::{Any, type_name};
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;
use mlua::prelude::{LuaError, LuaResult};
use mlua::{FromLua, Lua, Table, Value};
use tracing::{debug, trace};
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
		for value in value.pairs::<Value, Value>() {
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

			debug!("{}: Adding {identifier}", type_name::<P>());
			self.entries.insert(identifier, (priority, lua.unpack(prototype)?));
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

pub trait DynRegistryBuilder {
	fn lua_register(&mut self, lua: &Lua, value: Table) -> LuaResult<()>;
	fn  build(&mut self) -> Box<dyn Any>;
}

impl<P: Prototype> DynRegistryBuilder for RegistryBuilder<P> {
	fn lua_register(&mut self, lua: &Lua, value: Table) -> LuaResult<()> {
		self.register(lua, value)
	}

	fn build(&mut self) -> Box<dyn Any> {
		Box::new(self.build())
	}
}

pub struct LuaRegistryBuilder {
	inner: Box<dyn DynRegistryBuilder>,
}

impl LuaRegistryBuilder {
	pub fn new<P: Prototype>(inner: RegistryBuilder<P>) -> LuaRegistryBuilder  {
		LuaRegistryBuilder {
			inner: Box::new(inner)
		}
	}

	pub fn build<P: Prototype>(mut self) -> Registry<P> {
		*self.inner.build().downcast::<Registry<P>>().expect("we fucked up hard with downcasting here")
	}
}

#[lua_impl]
impl LuaRegistryBuilder {
	#[lua_method(register)]
	pub fn lua_register(&mut self, lua: &Lua, value: Table) -> LuaResult<()> {
		self.inner.lua_register(lua, value)
	}
}