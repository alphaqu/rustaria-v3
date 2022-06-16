use std::fmt::{Display, Formatter, Write};
use mlua::{FromLua, Lua, Value};
use tracing::trace;
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
        trace!("FromLua Identifier");
        let string = util::lua_string(lua_value)?;
        if let Some((namespace, path)) = string.split_once(':') {
            Ok(Identifier {
                namespace: namespace.to_string(),
                path: path.to_string()
            })
        } else {
            Ok(Identifier {
                namespace: "rustaria".to_string(),
                path: string
            })
        }
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