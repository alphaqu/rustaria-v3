//! Holds our luna interface
use mlua::Lua;
use eyre::{Result, WrapErr};

mod log;
pub mod registry_builder;
pub mod stargate;
pub mod reload;

pub fn register(lua: &Lua) -> Result<()> {
	log::register(lua, &lua.globals()).wrap_err("Registering log")?;
	Ok(())
}