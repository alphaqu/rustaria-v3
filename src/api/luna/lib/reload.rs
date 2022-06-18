use mlua::prelude::LuaResult;
use crate::api::luna::lib::stargate::Stargate;
use apollo::*;

pub struct Reload {
	pub stargate: Stargate,
	pub client: bool,
}

impl Reload {
}

#[lua_impl]
impl Reload {
	#[lua_field]
	pub fn get_stargate(&mut self) -> LuaResult<&mut Stargate> {
		Ok(&mut self.stargate)
	}

	#[lua_field]
	pub fn get_client(&mut self) -> LuaResult<bool> {
		Ok(self.client)
	}
}