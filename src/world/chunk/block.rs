use std::collections::HashMap;

use eyre::WrapErr;
use tracing::error_span;

use crate::{
	api::{luna::table::LunaTable, prototype::Prototype},
	ty::{id::Id, identifier::Identifier},
	util::blake3::Hasher,
	world::chunk::spread::{BlockSpreader, BlockSpreaderPrototype},
};

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BlockInstance {
	pub id:        Id<Block>,
	pub collision: bool,
}

pub struct Block {
	pub collision: bool,
	pub spread:    Option<BlockSpreader>,
}

impl Block {
	pub fn create(&self, id: Id<Block>) -> BlockInstance {
		BlockInstance {
			id,
			collision: self.collision,
		}
	}
}

pub struct BlockPrototype {
	pub collision: bool,
	pub spread:    Option<BlockSpreaderPrototype>,
}

impl BlockPrototype {
	pub fn bake(self, blocks: &HashMap<Identifier, Id<Block>>) -> eyre::Result<Block> {
		Ok(Block {
			collision: self.collision,
			spread:    if let Some(spread) = self.spread {
				Some(spread.bake(blocks).wrap_err("Could not bake spreader")?)
			} else {
				None
			},
		})
	}
}

impl Prototype for BlockPrototype {
	type Output = Block;

	fn get_name() -> &'static str { "block" }

	fn from_lua(table: LunaTable, _: &mut Hasher) -> eyre::Result<Self> {
		let _span = error_span!(target: "lua", "block").entered();
		Ok(BlockPrototype {
			collision: table.get("collision")?,
			spread:    table.get("spread")?,
		})
	}
}
