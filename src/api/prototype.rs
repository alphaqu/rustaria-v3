use crate::{api::luna::table::LunaTable, util::blake3::Hasher};

pub trait Prototype
where
	Self: Sized + 'static,
{
	type Output;
	fn get_name() -> &'static str;
	fn from_lua(table: LunaTable, hasher: &mut Hasher) -> eyre::Result<Self>;
}
