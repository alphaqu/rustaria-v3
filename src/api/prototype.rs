use mlua::FromLua;
use crate::api::id::Id;

pub trait Prototype where Self: Sized + FromLua {
	type Item;
	fn create(&self, id: Id<Self>) -> Self::Item;
}