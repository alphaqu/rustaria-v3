use std::fmt::{Display, Formatter, Write};
use mlua::{FromLua, Lua, Value};
use crate::api::util;

/// The identifier is a dual-string notifying which mod (namespace) the entry is from. and what it is.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(path: &'static str) -> Identifier {
        Identifier {
            namespace: "rustaria".to_string(),
            path: path.to_string()
        }
    }
}

impl FromLua for Identifier {
    fn from_lua(lua_value: Value, _: &Lua) -> mlua::Result<Self> {
        let table = util::lua_string(lua_value)?;
        Ok(Identifier {
            namespace: "rustaria".to_string(),
            path: table
        })
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.namespace)?;
        f.write_char(':')?;
        f.write_str(&self.path)?;
        Ok(())
    }
}