use mlua::FromLua;
use crate::ty::id::Id;

pub trait Prototype where Self: Sized + FromLua + 'static {
	fn get_name() -> &'static str;
}

pub trait FactoryPrototype: Prototype {
	type Item;
	fn create(&self, id: Id<Self>) -> Self::Item;
}