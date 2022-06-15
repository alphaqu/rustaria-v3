use mlua::FromLua;
use crate::ty::id::Id;

pub trait Prototype where Self: Sized + FromLua + 'static {
}

pub trait FactoryPrototype: Prototype {
	type Item;
	fn create(&self, id: Id<Self>) -> Self::Item;
}