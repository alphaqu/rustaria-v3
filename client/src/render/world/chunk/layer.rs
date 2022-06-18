use crate::render::atlas::Atlas;
use crate::render::ty::mesh_builder::MeshBuilder;
use crate::render::ty::vertex::PosTexVertex;
use crate::render::world::chunk::block::{KindDesc, LuaBlockRendererPrototype};
use crate::render::world::neighbor::{NeighborMatrixBuilder, SpriteConnectionKind};
use crate::{BlockRendererPrototype, Debug};
use eyre::{WrapErr};
use mlua::{Function, Lua, LuaSerdeExt};
use rustaria::api::luna::lib::registry_builder::RegistryBuilder;
use rustaria::api::luna::table::LunaTable;
use rustaria::api::prototype::{LuaPrototype, Prototype};
use rustaria::api::registry::{IdTable, Registry};
use rustaria::ty::block_pos::BlockPos;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::direction::{DirMap, Direction};
use rustaria::ty::identifier::Identifier;
use rustaria::util::blake3::Hasher;
use rustaria::world::chunk::block::{Block, BlockPrototype};
use rustaria::world::chunk::{BlockLayer, ConnectionType};
use std::collections::HashSet;
use rustaria::world::chunk::layer::BlockLayerPrototype;

pub struct LuaBlockLayerRendererPrototype {
    pub blocks: Registry<LuaBlockRendererPrototype>,
    pub get_uv: Function,
    pub get_rect: Function,
}

impl LuaBlockLayerRendererPrototype {
    pub fn bake(
        self,
        lua: &Lua,
        atlas: &Atlas,
        parent: &BlockLayerPrototype,
    ) -> eyre::Result<BlockLayerRendererPrototype> {
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


        Ok(BlockLayerRendererPrototype {
            block_renderers: parent.blocks.id_to_ident.iter().map(|(id, entry)| {
               (id, self.blocks.ident_to_id.get(&entry).map(|entry| {
                    self.blocks.get(*entry).bake(&atlas)
                }))
            }).collect(),
            kind_descs: kind_uvs,
        })
    }

    pub fn get_sprites(&self, sprites: &mut HashSet<Identifier>) {
        for (_, entry) in self.blocks.table.iter() {
            entry.get_sprites(sprites);
        }
    }
}

pub struct BlockLayerRendererPrototype {
    block_renderers: IdTable<BlockPrototype, Option<BlockRendererPrototype>>,
    kind_descs: Vec<KindDesc>,
}

impl Prototype for BlockLayerRendererPrototype {}

impl LuaPrototype for LuaBlockLayerRendererPrototype {
    type Output = BlockLayerRendererPrototype;

    fn get_name() -> &'static str {
        "block_layer_renderer"
    }

    fn from_lua(table: LunaTable, hasher: &mut Hasher) -> eyre::Result<Self> {
        let mut blocks = RegistryBuilder::<LuaBlockRendererPrototype>::new();
        blocks.register(table.lua, table.get("blocks")?)?;

        Ok(LuaBlockLayerRendererPrototype {
            blocks: blocks
                .build(table.lua, hasher)
                .wrap_err("Failed to build blocks registry")?,
            get_uv: table.get("get_uv")?,
            get_rect: table.get("get_rect")?,
        })
    }
}

impl BlockLayerRendererPrototype {
    pub fn mesh_chunk_layer(
        &self,
        chunk: ChunkPos,
        layer: &BlockLayer,
        neighbors: DirMap<Option<&BlockLayer>>,
        builder: &mut MeshBuilder<PosTexVertex>,
        debug: &mut Debug,
    ) {
        let func = |tile: &Block| {
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
