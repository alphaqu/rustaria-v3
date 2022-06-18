use mlua::{Lua, MetaMethod, Table, Value};
use tracing::{debug, error, info, trace, warn};

pub fn register(lua: &Lua, globals: &Table) -> mlua::Result<()> {
	let log = lua.create_table()?;
	log.set("trace", lua.create_function(trace)?)?;
	log.set("debug", lua.create_function(debug)?)?;
	log.set("info", lua.create_function(info)?)?;
	log.set("warn", lua.create_function(warn)?)?;
	log.set("error", lua.create_function(error)?)?;
	globals.set("log", log)?;
	Ok(())
}

fn trace(lua: &Lua, msg: Value) -> mlua::Result<()> {
	trace!(target: "luna", "{}", to_string(msg)?);
	Ok(())
}

fn debug(lua: &Lua, msg: Value) -> mlua::Result<()> {
	debug!(target: "luna", "{}", to_string(msg)?);
	Ok(())
}

fn info(lua: &Lua, msg: Value) -> mlua::Result<()> {
	info!(target: "luna", "{}", to_string(msg)?);
	Ok(())
}

fn warn(lua: &Lua, msg: Value) -> mlua::Result<()> {
	warn!(target: "luna", "{}", to_string(msg)?);
	Ok(())
}

fn error(lua: &Lua, msg: Value) -> mlua::Result<()> {
	error!(target: "luna", "{}", to_string(msg)?);
	Ok(())
}

fn to_string(msg: Value) -> mlua::Result<String> {
	Ok(match msg {
		Value::Nil => "nil".to_string(),
		Value::Boolean(value) => value.to_string(),
		Value::Integer(value) => value.to_string(),
		Value::Number(value) => value.to_string(),
		Value::String(string) => string.to_str()?.to_string(),
		Value::UserData(userdata) => {
			if let Value::Function(func) = userdata.get_metatable()?.get(MetaMethod::ToString)? {
				func.call(userdata)?
			} else {
				"no __tostring".to_string()
			}
		}
		_ => "unknown".to_string(),
	})
}
