use mlua::prelude::LuaResult;
use crate::api::Carrier;
use crate::api::luna::lib::registry_builder::RegistryBuilder;
use crate::entity::prototype::EntityPrototype;
use crate::chunk::block::BlockLayerPrototype;

/// Creates a carrier
pub struct Stargate {
	pub block_layer: RegistryBuilder<BlockLayerPrototype>,
	pub entity: RegistryBuilder<EntityPrototype>,
}

impl Stargate {
	pub fn new() -> Stargate {
		Stargate {
			block_layer: RegistryBuilder::new(),
			entity: RegistryBuilder::new()
		}
	}

	/// *Carrier has arrived!* - Carrier
	pub fn build(&mut self) -> Carrier {
		Carrier  {
			block_layer: self.block_layer.build(),
			entity: self.entity.build()
		}
	}
}

use apollo::*;

#[lua_impl]
impl Stargate {
	#[lua_field]
	pub fn get_block_layer(&mut self) -> LuaResult<&mut RegistryBuilder<BlockLayerPrototype>> {
		Ok(&mut self.block_layer)
	}

	#[lua_field]
	pub fn get_entity(&mut self) -> LuaResult<&mut RegistryBuilder<EntityPrototype>> {
		Ok(&mut self.entity)
	}
}