use mlua::prelude::LuaResult;
use crate::api::luna::lib::stargate::Stargate;

pub struct Reload {
	pub stargate: Stargate,
	pub client: bool,
}

use apollo::*;

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