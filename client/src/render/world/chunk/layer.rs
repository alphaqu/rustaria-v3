use std::collections::HashSet;

use eyre::WrapErr;
use mlua::{Function, Lua, LuaSerdeExt};
use rustaria::{
	api::{
		id_table::IdTable,
		luna::{lib::registry_builder::RegistryBuilder, table::LunaTable},
		prototype::Prototype,
		registry::Registry,
	},
	ty::{
		block_pos::BlockPos,
		chunk_pos::ChunkPos,
		direction::{DirMap, Direction},
		identifier::Identifier,
	},
	util::blake3::Hasher,
	world::chunk::{
		block::{Block, BlockInstance},
		layer::BlockLayer,
		ChunkLayer, ConnectionType,
	},
};
use tracing::error_span;

use crate::{
	render::{
		atlas::Atlas,
		ty::{mesh_builder::MeshBuilder, vertex::PosTexVertex},
		world::{
			chunk::block::{BlockRendererPrototype, KindDesc},
			neighbor::{NeighborMatrixBuilder, SpriteConnectionKind},
		},
	},
	BlockRenderer, Debug,
};

pub struct BlockLayerRenderer {
	block_renderers: IdTable<Block, Option<BlockRenderer>>,
	kind_descs: Vec<KindDesc>,
}

impl BlockLayerRenderer {
	pub fn mesh_chunk_layer(
		&self,
		chunk: ChunkPos,
		layer: &ChunkLayer<BlockInstance>,
		neighbors: DirMap<Option<&ChunkLayer<BlockInstance>>>,
		builder: &mut MeshBuilder<PosTexVertex>,
		debug: &mut Debug,
	) {
		let func = |tile: &BlockInstance| {
			self.block_renderers
				.get(tile.id)
				.as_ref()
				.map(|renderer| renderer.connection_type)
		};

		let mut matrix = NeighborMatrixBuilder::new(layer.map(ConnectionType::Isolated, func));
		matrix.compile_internal();

		for dir in Direction::values() {
			if let Some(neighbor) = neighbors[dir] {
				matrix.compile_edge(dir, &neighbor.map(ConnectionType::Isolated, func));
			}
		}

		let connection_layer = matrix.export();
		layer.entries(|entry, connection| {
			if let Some(renderer) = self.block_renderers.get(connection.id) {
				renderer.mesh(
					BlockPos::new(chunk, entry),
					&self.kind_descs[connection_layer[entry] as u8 as usize],
					builder,
					debug,
				);
			}
		});
	}
}

pub struct BlockLayerRendererPrototype {
	pub blocks: Registry<BlockRendererPrototype>,
	pub get_uv: Function,
	pub get_rect: Function,
}

impl BlockLayerRendererPrototype {
	pub fn bake(
		self,
		lua: &Lua,
		atlas: &Atlas,
		parent: &BlockLayer,
	) -> eyre::Result<BlockLayerRenderer> {
		let mut kind_uvs = Vec::new();
		for value in SpriteConnectionKind::iter() {
			let value = format!("{:?}", value);
			kind_uvs.push(KindDesc {
				uv: lua
					.from_value(
						self.get_uv
							.call(value.clone())
							.wrap_err("Failed to get uv.")?,
					)
					.wrap_err("Failed to get uv result.")?,
				rect: lua
					.from_value(self.get_rect.call(value).wrap_err("Failed to get rect.")?)
					.wrap_err("Failed to get rect result.")?,
			});
		}

		Ok(BlockLayerRenderer {
			block_renderers: parent
				.blocks
				.id_to_ident
				.iter()
				.map(|(id, entry)| {
					(
						id,
						self.blocks
							.ident_to_id
							.get(entry)
							.map(|entry| self.blocks.get(*entry).bake(atlas)),
					)
				})
				.collect(),
			kind_descs: kind_uvs,
		})
	}

	pub fn get_sprites(&self, sprites: &mut HashSet<Identifier>) {
		for (_, entry) in self.blocks.table.iter() {
			entry.get_sprites(sprites);
		}
	}
}

impl Prototype for BlockLayerRendererPrototype {
	type Output = BlockLayerRenderer;

	fn get_name() -> &'static str { "block_layer_renderer" }

	fn from_lua(table: LunaTable) -> eyre::Result<Self> {
		let mut blocks = RegistryBuilder::<BlockRendererPrototype>::new();
		blocks.register(table.lua, table.get("blocks")?)?;

		Ok(BlockLayerRendererPrototype {
			blocks: blocks
				.build(table.lua)
				.wrap_err("Building \"blocks\" registry")?,
			get_uv: table.get("get_uv")?,
			get_rect: table.get("get_rect")?,
		})
	}
}
