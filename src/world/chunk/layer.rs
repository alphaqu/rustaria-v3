use eyre::{ContextCompat, WrapErr};
use crate::api::luna::lib::registry_builder::RegistryBuilder;
use crate::api::luna::table::LunaTable;
use crate::api::prototype::{LuaPrototype, Prototype};
use crate::api::registry::{Registry};
use crate::ty::id::Id;
use crate::ty::identifier::Identifier;
use crate::util::blake3::Hasher;
use crate::world::chunk::block::{BlockPrototype, LuaBlockPrototype};


pub struct BlockLayerPrototype {
	pub blocks: Registry<BlockPrototype>,
	pub default: Id<BlockPrototype>,
	pub collision: bool,
}

impl Prototype for BlockLayerPrototype {

}

pub struct LuaBlockLayerPrototype {
	pub blocks: Registry<LuaBlockPrototype>,
	pub default: Identifier,
	pub collision: bool,
}

impl LuaBlockLayerPrototype {
	pub fn bake(self) -> eyre::Result<BlockLayerPrototype> {
		let lookup = self.blocks.ident_to_id.iter().map(|(ident, id)| {
			(ident.clone(), id.build())
		}).collect();

		let mut out = Vec::new();
		for (id, ident, entry) in self.blocks.into_iter() {
			let prototype = entry.bake(&lookup).wrap_err_with(|| format!("Failed to bake block {}", ident))?;
			out.push((id.build(), ident, prototype));
		}


		let registry: Registry<BlockPrototype> = out.into_iter().collect();
		Ok(BlockLayerPrototype {
			default: *registry.ident_to_id.get(&self.default).wrap_err("Could not find default tile registered")?,
			blocks: registry,
			collision: self.collision,
		})
	}
}

impl LuaPrototype for LuaBlockLayerPrototype {
	type Output = BlockLayerPrototype;

	fn get_name() -> &'static str {
		"block_layer"
	}

	fn from_lua(table: LunaTable, hasher: &mut Hasher) -> eyre::Result<Self> {
		let mut blocks = RegistryBuilder::<LuaBlockPrototype>::new();
		blocks.register(table.lua, table.get("blocks")?)?;
		Ok(LuaBlockLayerPrototype {
			blocks: blocks.build(table.lua, hasher).wrap_err("Failed to build blocks registry")?,
			default: table.get("default")?,
			collision: table.get("collision")?
		})
	}
}
