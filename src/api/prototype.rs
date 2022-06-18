use crate::api::luna::table::LunaTable;
use crate::ty::id::Id;
use crate::util::blake3::Hasher;

pub trait LuaPrototype where Self: Sized + 'static {
	type Output: Prototype;
	fn get_name() -> &'static str;
	fn from_lua(table: LunaTable, hasher: &mut Hasher) -> eyre::Result<Self>;
}

pub trait Prototype where Self: Sized {
}

pub trait FactoryPrototype: Prototype {
	type Item;
	fn create(&self, id: Id<Self>) -> Self::Item;
}