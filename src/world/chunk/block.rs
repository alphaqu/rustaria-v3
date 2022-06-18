use std::collections::HashMap;
use crate::api::prototype::{FactoryPrototype, LuaPrototype, Prototype};
use crate::ty::id::Id;
use crate::world::chunk::spread::{BlockSpreaderPrototype, LuaBlockSpreaderPrototype};
use eyre::WrapErr;
use crate::api::luna::table::LunaTable;
use crate::ty::identifier::Identifier;
use crate::util::blake3::Hasher;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub id: Id<BlockPrototype>,
    pub collision: bool,
}

pub struct BlockPrototype {
    pub collision: bool,
    pub spread: Option<BlockSpreaderPrototype>,
}

impl Prototype for BlockPrototype {}

impl FactoryPrototype for BlockPrototype {
    type Item = Block;
    fn create(&self, id: Id<Self>) -> Self::Item {
        Block {
            id,
            collision: self.collision,
        }
    }
}

pub struct LuaBlockPrototype {
    pub collision: bool,
    pub spread: Option<LuaBlockSpreaderPrototype>,
}

impl LuaBlockPrototype {
    pub fn bake(self, blocks: &HashMap<Identifier, Id<BlockPrototype>>) -> eyre::Result<BlockPrototype> {
        Ok(BlockPrototype {
            collision: self.collision,
            spread: if let Some(spread) = self.spread {
                Some(spread.bake(blocks).wrap_err("Could not bake spreader")?)
            } else {
                None
            },
        })
    }
}

impl LuaPrototype for LuaBlockPrototype {
    type Output = BlockPrototype;

    fn get_name() -> &'static str {
        "block"
    }

    fn from_lua(table: LunaTable, _: &mut Hasher) -> eyre::Result<Self> {
        Ok(LuaBlockPrototype {
            collision: table.get("collision")?,
            spread: table.get("spread")?,
        })
    }
}
