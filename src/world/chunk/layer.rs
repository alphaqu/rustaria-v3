use eyre::{ContextCompat, WrapErr};
use tracing::error_span;

use crate::{
	api::{
		luna::{lib::registry_builder::RegistryBuilder, table::LunaTable},
		prototype::Prototype,
		registry::Registry,
	},
	ty::{id::Id, identifier::Identifier},
	util::blake3::Hasher,
	world::chunk::block::{Block, BlockPrototype},
};

pub struct BlockLayer {
	pub blocks:    Registry<Block>,
	pub default:   Id<Block>,
	pub collision: bool,
}

pub struct BlockLayerPrototype {
	pub blocks:    Registry<BlockPrototype>,
	pub default:   Identifier,
	pub collision: bool,
}

impl BlockLayerPrototype {
	pub fn bake(self) -> eyre::Result<BlockLayer> {
		let lookup = self
			.blocks
			.ident_to_id
			.iter()
			.map(|(ident, id)| (ident.clone(), id.build()))
			.collect();

		let mut out = Vec::new();
		for (id, ident, entry) in self.blocks.into_iter() {
			let prototype = entry
				.bake(&lookup)
				.wrap_err_with(|| format!("Failed to bake block {}", ident))?;
			out.push((id.build(), ident, prototype));
		}

		let registry: Registry<Block> = out.into_iter().collect();
		Ok(BlockLayer {
			default:   *registry
				.ident_to_id
				.get(&self.default)
				.wrap_err("Could not find default tile registered")?,
			blocks:    registry,
			collision: self.collision,
		})
	}
}

impl Prototype for BlockLayerPrototype {
	type Output = BlockLayer;

	fn get_name() -> &'static str { "block_layer" }

	fn from_lua(table: LunaTable, hasher: &mut Hasher) -> eyre::Result<Self> {
		let _span = error_span!(target: "lua", "block_layer").entered();
		let mut blocks = RegistryBuilder::<BlockPrototype>::new();
		blocks.register(table.lua, table.get("blocks")?)?;
		Ok(BlockLayerPrototype {
			blocks:    blocks
				.build(table.lua, hasher)
				.wrap_err("Failed to create blocks registry")?,
			default:   table.get("default")?,
			collision: table.get("collision")?,
		})
	}
}
