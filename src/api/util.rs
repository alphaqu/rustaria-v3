use mlua::{Table, Value};

pub fn lua_table(value: Value) -> mlua::Result<Table> {
	match value {
		Value::Table(value) => Ok(value),
		_ => Err(mlua::Error::external(format!(
			"Expected table not {value:?}"
		))),
	}
}

pub fn lua_string(value: Value) -> mlua::Result<String> {
	match value {
		Value::String(value) => Ok(value.to_str()?.to_string()),
		_ => Err(mlua::Error::external(format!(
			"Expected string not {value:?}"
		))),
	}
}
