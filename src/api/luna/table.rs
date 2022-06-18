use eyre::Context;
use eyre::Result;
use mlua::{FromLua, Lua, LuaSerdeExt, Table, ToLua};
use serde::Deserialize;
use std::any::type_name;
use std::fmt::Display;

pub struct LunaTable<'a> {
    pub lua: &'a Lua,
    pub table: Table,
}

impl<'a> LunaTable<'a> {
    pub fn get<K: ToLua + Display + Clone, V: FromLua>(&self, key: K) -> Result<V> {
        self.table
            .get(key.clone())
            .wrap_err_with(|| format!("Failed to get {}", key))
    }

    pub fn get_ser<'de, K: ToLua + Display + Clone, V: Deserialize<'de>>(&self, key: K) -> Result<V> {
        self.lua
            .from_value(self.get(key.clone())?)
            .wrap_err_with(|| format!("Failed to convert {} to {}", key, type_name::<V>()))
    }
}
