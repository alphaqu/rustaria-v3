use euclid::{point2, size2, Rect, Vector2D, vec2, Point2D, Size2D};
use rustaria::api::id::Id;
use rustaria::api::registry::MappedRegistry;
use rustaria::chunk::block::{Block, BlockPrototype, BlockLayerPrototype};
use rustaria::chunk::storage::ChunkStorage;
use rustaria::chunk::{BlockLayer, Chunk, CHUNK_SIZE, ChunkLayer, ConnectionType};
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::direction::{DirMap, Direction};
use rustaria::ty::{Offset, WS};
use eyre::Result;
use glium::{Blend, DrawParameters, Frame, Program, uniform};
use mlua::{LuaSerdeExt, Table, ToLua, Value};
use tracing::{info_span, span, trace, trace_span};
use rustaria::api::Api;

use rustaria::ty::block_pos::BlockPos;
use crate::{Camera, Frontend};

use crate::render::atlas::Atlas;
use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::neighbor::{NeighborMatrixBuilder, SpriteConnectionKind};
use crate::render::PosTexVertex;

pub struct ChunkRenderer {
    drawer: MeshDrawer<PosTexVertex>,
    layer_renderers: MappedRegistry<BlockLayerPrototype, BlockLayerRenderer>,
}

impl ChunkRenderer {
    pub fn new(api: &Api, frontend: &Frontend, atlas: &Atlas) -> Result<ChunkRenderer> {
        Ok(ChunkRenderer  {
            drawer: frontend.create_drawer()?,
            layer_renderers: api.carrier.block_layers.map(|id, prototype| {
                BlockLayerRenderer::new(api, prototype, atlas)
            })
        })
    }

    pub fn mesh(&mut self, player_pos: Vector2D<f32, WS>, chunks: &ChunkStorage) -> Result<()> {
        let mut builder = MeshBuilder::new();
        for y in -4..4 {
            for x in -4..4 {
                if let Ok(pos) = ChunkPos::try_from(
                    player_pos + vec2(x as f32 * CHUNK_SIZE as f32, y as f32 * CHUNK_SIZE as f32),
                ) {
                    if let Some(chunk) = chunks.get(pos) {
                        self.mesh_chunk(pos, chunk, chunks, &mut builder);
                    }
                }
            }
        }

        self.drawer.upload(&builder)?;
        Ok(())
    }

    pub fn draw(&mut self, frontend: &Frontend, atlas: &Atlas, camera: &Camera, pos_color_program: &Program, frame: &mut Frame) -> Result<()> {
        let uniforms = uniform! {
            screen_ratio: frontend.screen_ratio,
            atlas: &atlas.texture,
            player_pos: camera.pos.to_array(),
            zoom: camera.zoom,
        };

        let draw_parameters = DrawParameters {
            blend: Blend::alpha_blending(),
            ..DrawParameters::default()
        };
        self.drawer.draw(frame, &pos_color_program, &uniforms, &draw_parameters)?;
        Ok(())
    }

    fn mesh_chunk(
        &self,
        pos: ChunkPos,
        chunk: &Chunk,
        chunks: &ChunkStorage,
        builder: &mut MeshBuilder<PosTexVertex>,
    ) {
        let mut neighbors = DirMap::new([None; 4]);
        for dir in Direction::values() {
            if let Some(pos) = pos.checked_offset(dir) {
                if let Some(chunk) = chunks.get(pos) {
                    neighbors[dir] = Some(chunk);
                }
            }
        }

        for (id, layer) in chunk.layers.iter() {
            self.layer_renderers
                .get(id)
                .mesh_chunk_layer(pos, layer, neighbors.map(|dir, option| {
                    option.map(|c| c.layers.get(id))
                }), builder);
        }
    }
}

pub struct KindDesc {
    uv: Rect<f32, WS>,
    rect: Rect<f32, WS>,
}

pub struct BlockLayerRenderer {
    entry_renderers: MappedRegistry<BlockPrototype, Option<BlockRenderer>>,
    kind_descs: Vec<KindDesc>,
}

impl BlockLayerRenderer {
    pub fn new(api: &Api, prototype: &BlockLayerPrototype, atlas: &Atlas) -> BlockLayerRenderer {
        let _span = trace_span!("test").entered();
        let mut kind_uvs = Vec::new();
        for value in SpriteConnectionKind::iter() {
            kind_uvs.push(KindDesc {
                uv: api.lua.from_value(prototype.get_uv.call(format!("{value:?}")).unwrap()).unwrap(),
                rect: api.lua.from_value(prototype.get_rect.call(format!("{value:?}")).unwrap()).unwrap()
            });
        }
        BlockLayerRenderer {
            entry_renderers: prototype.registry.map(|id, prototype| {
                prototype.image.as_ref().map(|image| {
                    BlockRenderer {
                        tex_pos: atlas.get(image),
                        connection_type: prototype.connection_type
                    }
                })
            }),
            kind_descs: kind_uvs,
        }
    }

    pub fn mesh_chunk_layer(
        &self,
        chunk: ChunkPos,
        layer: &BlockLayer,
        neighbors: DirMap<Option<&BlockLayer>>,
        builder: &mut MeshBuilder<PosTexVertex>,
    ) {
        let func = |tile: &Block| {
            self.entry_renderers.get(tile.id).as_ref().map(|renderer| renderer.connection_type)
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
            if let Some(renderer) = self.entry_renderers.get(connection.id) {

                renderer.mesh(BlockPos::new(chunk, entry),
                              &self.kind_descs[connection_layer[entry] as u8 as usize],
                              builder);
            }
        });
    }
}

pub struct BlockRenderer {
    pub tex_pos: Rect<f32, Atlas>,
    pub connection_type: ConnectionType,
}

impl BlockRenderer {
    pub fn mesh(
        &self,
        pos: BlockPos,
        desc: &KindDesc,
        builder: &mut MeshBuilder<PosTexVertex>,
    ) {
        let mut texture = self.tex_pos;

        let tile_width = texture.size.width / 3.0;
        let tile_height = texture.size.height;

        texture.size.width = desc.uv.size.width * tile_width;
        texture.size.height = desc.uv.size.height * tile_height;
        texture.origin.x += desc.uv.origin.x * tile_width;
        texture.origin.y += desc.uv.origin.y * tile_height;

        let mut quad_pos = desc.rect;
        quad_pos.origin += size2(pos.x() as f32, pos.y() as f32);
        builder.push_quad((
            quad_pos,
            texture,
        ));
    }

}
