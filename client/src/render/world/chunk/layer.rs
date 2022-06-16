use crate::render::atlas::Atlas;
use crate::render::builder::MeshBuilder;
use crate::render::world::chunk::block::{BlockRenderer, KindDesc};
use crate::render::world::neighbor::{NeighborMatrixBuilder, SpriteConnectionKind};
use crate::render::PosTexVertex;
use crate::{BlockRendererPrototype, ClientApi, Debug};
use mlua::{FromLua, Function, Lua, LuaSerdeExt, Value};
use rustaria::api::luna::lib::registry_builder::RegistryBuilder;
use rustaria::api::prototype::Prototype;
use rustaria::api::registry::{MappedRegistry, Registry};
use rustaria::api::util::lua_table;
use rustaria::world::chunk::block::{Block, BlockPrototype};
use rustaria::world::chunk::layer::BlockLayerPrototype;
use rustaria::world::chunk::{BlockLayer, ConnectionType};
use rustaria::ty::block_pos::BlockPos;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::direction::{DirMap, Direction};

pub struct BlockLayerRendererPrototype {
    pub registry: Registry<BlockRendererPrototype>,
    pub get_uv: Function,
    pub get_rect: Function,
}

impl BlockLayerRendererPrototype {
    pub fn create(&self, api: &ClientApi, parent: &BlockLayerPrototype, atlas: &Atlas) -> BlockLayerRenderer {
        let mut kind_uvs = Vec::new();
        for value in SpriteConnectionKind::iter() {
            let value = format!("{:?}", value);
            kind_uvs.push(KindDesc {
                uv: api
                    .luna
                    .lua
                    .from_value(self.get_uv.call(value.clone()).unwrap())
                    .unwrap(),
                rect: api
                    .luna
                    .lua
                    .from_value(self.get_rect.call(value).unwrap())
                    .unwrap(),
            });
        }

        BlockLayerRenderer {
            block_renderers: parent.registry.map(|_, id, prototype| {
                let ident = parent.registry.id_to_ident(id);
                self.registry.ident_to_id(ident).map(|id| {
                    self.registry.get(id).create(atlas)
                })
            }),
            kind_descs: kind_uvs,
        }
    }

}

impl Prototype for BlockLayerRendererPrototype {
    fn get_name() -> &'static str {
        "block_layer_renderer"
    }
}

impl FromLua for BlockLayerRendererPrototype {
    fn from_lua(lua_value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table = lua_table(lua_value)?;

        let mut builder = RegistryBuilder::new();
        builder.register(lua, table.get("entries")?)?;

        Ok(BlockLayerRendererPrototype {
            registry: builder.build(),
            get_uv: table.get("get_uv")?,
            get_rect: table.get("get_rect")?,
        })
    }
}

pub struct BlockLayerRenderer {
    block_renderers: MappedRegistry<BlockPrototype, Option<BlockRenderer>>,
    kind_descs: Vec<KindDesc>,
}

impl BlockLayerRenderer {
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
