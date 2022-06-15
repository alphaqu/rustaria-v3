use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Instant;

use euclid::{rect, Rect, size2, vec2, Vector2D};
use eyre::Result;
use glium::{Blend, DrawParameters, Frame, Program, uniform};
use mlua::LuaSerdeExt;

use rustaria::api::Api;
use rustaria::api::registry::MappedRegistry;
use rustaria::chunk::{BlockLayer, Chunk, CHUNK_SIZE, ConnectionType};
use rustaria::chunk::block::{Block, BlockLayerPrototype, BlockPrototype};
use rustaria::chunk::storage::ChunkStorage;
use rustaria::debug::{DebugCategory, DebugRendererImpl};
use rustaria::draw_debug;
use rustaria::ty::{Offset, WS};
use rustaria::ty::block_pos::BlockPos;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::direction::{Direction, DirMap};

use crate::{Camera, DebugRenderer, Frontend};
use crate::render::atlas::Atlas;
use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::neighbor::{NeighborMatrixBuilder, SpriteConnectionKind};
use crate::render::PosTexVertex;

pub struct ChunkRenderer {
    drawer: MeshDrawer<PosTexVertex>,
    cached_meshes: HashMap<ChunkPos, MeshBuilder<PosTexVertex>>,
    layer_renderers: MappedRegistry<BlockLayerPrototype, BlockLayerRenderer>,
    chunks_dirty: bool,
}

impl ChunkRenderer {
    pub fn new(api: &Api, frontend: &Frontend, atlas: &Atlas) -> Result<ChunkRenderer> {
        Ok(ChunkRenderer {
            drawer: frontend.create_drawer()?,
            cached_meshes: Default::default(),
            layer_renderers: api
                .carrier
                .block_layer
                .map(|_, prototype| BlockLayerRenderer::new(api, prototype, atlas)),
            chunks_dirty: true,
        })
    }

    pub fn tick(
        &mut self,
        player_pos: Vector2D<f32, WS>,
        chunks: &ChunkStorage,
        debug: &mut DebugRenderer,
    ) -> Result<()> {
        for pos in chunks.get_dirty() {
            self.cached_meshes.remove(pos);
            self.chunks_dirty = true;
        }

        if self.chunks_dirty {
            let mut builder = MeshBuilder::new();
            for y in -4..4 {
                for x in -4..4 {
                    if let Ok(pos) = ChunkPos::try_from(
                        player_pos
                            + vec2(x as f32 * CHUNK_SIZE as f32, y as f32 * CHUNK_SIZE as f32),
                    ) {
                        if let Entry::Occupied(value) = self.cached_meshes.entry(pos) {
                            builder.extend(value.get());
                            draw_debug!(
                                debug,
                                DebugCategory::ChunkMeshing,
                                rect(
                                    pos.x as f32 * CHUNK_SIZE as f32,
                                    pos.y as f32 * CHUNK_SIZE as f32,
                                    CHUNK_SIZE as f32,
                                    CHUNK_SIZE as f32,
                                ),
                                0x5b595c,
                                2.0,
                                0.5
                            );
                        } else {
                            self.remesh_chunk(pos, chunks, debug);
                            builder.extend(self.cached_meshes.get(&pos).unwrap());
                        }
                    }
                }
            }

            self.drawer.upload(&builder)?;
            self.chunks_dirty = false;
        }
        Ok(())
    }

    pub fn remesh_world(&mut self) {
        self.cached_meshes.clear();
    }

    fn remesh_chunk(
        &mut self,
        pos: ChunkPos,
        chunks: &ChunkStorage,
        debug: &mut DebugRenderer,
    ) {
        if let Some(chunk) = chunks.get(pos) {
            let mut chunk_builder = MeshBuilder::new();
            self.mesh_chunk(pos, chunk, chunks, &mut chunk_builder, debug);
            self.cached_meshes.insert(pos, chunk_builder);
            draw_debug!(
                debug,
                DebugCategory::ChunkMeshing,
                rect(
                    pos.x as f32 * CHUNK_SIZE as f32,
                    pos.y as f32 * CHUNK_SIZE as f32,
                    CHUNK_SIZE as f32,
                    CHUNK_SIZE as f32,
                ),
                0xffffff,
                2.0,
                0.5
            );
        }
    }

    pub fn draw(
        &mut self,
        frontend: &Frontend,
        atlas: &Atlas,
        camera: &Camera,
        pos_color_program: &Program,
        frame: &mut Frame,
    ) -> Result<()> {
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
        self.drawer
            .draw(frame, pos_color_program, &uniforms, &draw_parameters)?;
        Ok(())
    }

    fn mesh_chunk(
        &self,
        pos: ChunkPos,
        chunk: &Chunk,
        chunks: &ChunkStorage,
        builder: &mut MeshBuilder<PosTexVertex>,
        debug: &mut DebugRenderer,
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
            self.layer_renderers.get(id).mesh_chunk_layer(
                pos,
                layer,
                neighbors.map(|_, option| option.map(|c| c.layers.get(id))),
                builder,
                debug
            );
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
        let mut kind_uvs = Vec::new();
        for value in SpriteConnectionKind::iter() {
            kind_uvs.push(KindDesc {
                uv: api
                    .luna.lua
                    .from_value(prototype.get_uv.call(format!("{value:?}")).unwrap())
                    .unwrap(),
                rect: api
                    .luna.lua
                    .from_value(prototype.get_rect.call(format!("{value:?}")).unwrap())
                    .unwrap(),
            });
        }
        BlockLayerRenderer {
            entry_renderers: prototype.registry.map(|_, prototype| {
                prototype.image.as_ref().map(|image| BlockRenderer {
                    tex_pos: atlas.get(image),
                    connection_type: prototype.connection_type,
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
        debug: &mut DebugRenderer,
    ) {
        let func = |tile: &Block| {
            self.entry_renderers
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
            if let Some(renderer) = self.entry_renderers.get(connection.id) {
                renderer.mesh(
                    BlockPos::new(chunk, entry),
                    &self.kind_descs[connection_layer[entry] as u8 as usize],
                    builder,
                    debug
                );
            }
        });
    }
}

pub struct BlockRenderer {
    pub tex_pos: Rect<f32, Atlas>,
    pub connection_type: ConnectionType,
}

impl BlockRenderer {
    pub fn mesh(&self, pos: BlockPos, desc: &KindDesc, builder: &mut MeshBuilder<PosTexVertex>, debug: &mut DebugRenderer) {
        let mut texture = self.tex_pos;

        let variation = get_variation(pos) % ((texture.size.width / texture.size.height) as u32);
        let layout_width = texture.size.width / 3.0;

        let layout_height = texture.size.height;
        texture.origin.x += layout_width * variation as f32;

        texture.size.width = desc.uv.size.width * layout_width;
        texture.size.height = desc.uv.size.height * layout_height;
        texture.origin.x += desc.uv.origin.x * layout_width;
        texture.origin.y += desc.uv.origin.y * layout_height;
        let mut quad_pos = desc.rect;

        quad_pos.origin += size2(pos.x() as f32, pos.y() as f32);

        const VARIATION_COLORS: [u32; 3] = [0xff0000, 0x00ff00, 0x0000ff];
        draw_debug!(debug, DebugCategory::ChunkMeshing, vec2(pos.x() as f32 + 0.5, pos.y() as f32 + 0.5), VARIATION_COLORS[(variation % 3) as usize], 5.0, 0.5);
        builder.push_quad((quad_pos, texture));
    }
}

fn get_variation(pos: BlockPos) -> u32 {
    let x = (pos.x() & 0xFFFFFFFF) as u32;
    let y = (pos.y() & 0xFFFFFFFF) as u32;
    let offset_x = x.overflowing_mul(69).0;
    let mut v = offset_x.overflowing_mul(y + 420).0;
    v ^= v.overflowing_shl(13).0;
    v ^= v.overflowing_shr(7).0;
    v ^= v.overflowing_shl(17).0;
    v
}
