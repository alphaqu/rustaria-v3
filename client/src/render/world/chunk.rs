use std::collections::hash_map::Entry;
use std::collections::HashMap;
use euclid::{rect, vec2, Vector2D};
use eyre::Result;
use glium::{Blend, DrawParameters, Frame, Program, uniform};
use mlua::LuaSerdeExt;
use layer::BlockLayerRenderer;

use rustaria::api::Api;
use rustaria::api::registry::MappedRegistry;
use rustaria::chunk::{Chunk, CHUNK_SIZE};
use rustaria::chunk::layer::BlockLayerPrototype;
use rustaria::chunk::storage::ChunkStorage;
use rustaria::debug::{DebugCategory, DebugRendererImpl};
use rustaria::draw_debug;
use rustaria::ty::{Offset, WS};
use rustaria::ty::block_pos::BlockPos;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::direction::{Direction, DirMap};

use crate::{Camera, ClientApi, Debug, Frontend};
use crate::render::atlas::Atlas;
use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::PosTexVertex;

pub mod layer;
pub mod block;

pub struct WorldChunkRenderer {
    drawer: MeshDrawer<PosTexVertex>,
    cached_meshes: HashMap<ChunkPos, MeshBuilder<PosTexVertex>>,
    layer_renderers: MappedRegistry<BlockLayerPrototype, Option<BlockLayerRenderer>>,
    chunks_dirty: bool,
}

impl WorldChunkRenderer {
    pub fn new(api: &ClientApi, frontend: &Frontend, atlas: &Atlas) -> Result<WorldChunkRenderer> {
        Ok(WorldChunkRenderer {
            drawer: frontend.create_drawer()?,
            cached_meshes: Default::default(),
            layer_renderers: api
                .carrier
                .block_layer
                .map(|ident, id, prototype| {
                    let ident = api.carrier.block_layer.id_to_ident(id);
                    if let Some(id) = api.c_carrier.block_layer_renderer.ident_to_id(ident) {
                        Some(api.c_carrier.block_layer_renderer.get(id).create(api, prototype, atlas))
                    } else {
                        None
                    }
                }),
            chunks_dirty: true,
        })
    }

    pub fn tick(
	    &mut self,
	    player_pos: Vector2D<f32, WS>,
	    chunks: &ChunkStorage,
	    debug: &mut Debug,
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
	    debug: &mut Debug,
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
	    debug: &mut Debug,
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
            if let Some(renderer) = self.layer_renderers.get(id) {
                renderer.mesh_chunk_layer(
                    pos,
                    layer,
                    neighbors.map(|_, option| option.map(|c| c.layers.get(id))),
                    builder,
                    debug
                );
            }
        }
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
