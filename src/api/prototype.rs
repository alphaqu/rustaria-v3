use mlua::FromLua;
use crate::api::id::Id;

pub trait Prototype where Self: Sized + FromLua + 'static {
}

pub trait FactoryPrototype: Prototype {
	type Item;
	fn create(&self, id: Id<Self>) -> Self::Item;
}

pub trait KernelId<P: Prototype> {
	fn get_id(&self) -> Id<P>;
}