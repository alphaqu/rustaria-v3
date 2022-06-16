use std::collections::hash_map::Entry;
use std::collections::HashMap;

use euclid::{rect, vec2, Vector2D};
use eyre::Result;
use glium::{uniform, Blend, DrawParameters, Frame, Program};
use mlua::LuaSerdeExt;

use layer::BlockLayerRenderer;
use rustaria::api::registry::MappedRegistry;
use rustaria::chunk::layer::BlockLayerPrototype;
use rustaria::chunk::storage::ChunkStorage;
use rustaria::chunk::{Chunk, CHUNK_SIZE};
use rustaria::debug::{DebugCategory, DebugRendererImpl};
use rustaria::draw_debug;
use rustaria::ty::block_pos::BlockPos;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::direction::{DirMap, Direction};
use rustaria::ty::{Offset, WS};

use crate::render::atlas::Atlas;
use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::draw::Draw;
use crate::render::PosTexVertex;
use crate::{ClientApi, Debug, Frontend, PlayerSystem, Viewport};

pub mod block;
pub mod layer;

type LayerRenderers = MappedRegistry<BlockLayerPrototype, Option<BlockLayerRenderer>>;
pub struct WorldChunkRenderer {
    chunk_renderers: HashMap<ChunkPos, ChunkRenderer>,
    layer_renderers: LayerRenderers,
}

impl WorldChunkRenderer {
    pub fn new(api: &ClientApi, frontend: &Frontend, atlas: &Atlas) -> Result<WorldChunkRenderer> {
        Ok(WorldChunkRenderer {
            chunk_renderers: Default::default(),
            layer_renderers: api.carrier.block_layer.map(|ident, id, prototype| {
                let ident = api.carrier.block_layer.id_to_ident(id);
                api.c_carrier
                    .block_layer_renderer
                    .ident_to_id(ident)
                    .map(|id| {
                        api.c_carrier
                            .block_layer_renderer
                            .get(id)
                            .create(api, prototype, atlas)
                    })
            }),
        })
    }

    pub fn tick(
        &mut self,
        chunks: &ChunkStorage,
    ) -> Result<()> {
        for pos in chunks.get_dirty() {
            if let Some(renderer) = self.chunk_renderers.get_mut(pos) {
                renderer.dirty = true;
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, chunk: &ChunkStorage, program: &Program, draw: &mut Draw) -> Result<()> {
        let uniforms = uniform! {
            screen_ratio: draw.frontend.aspect_ratio,
            atlas: &draw.atlas.texture,
            player_pos: draw.viewport.pos.to_array(),
            zoom: draw.viewport.zoom,
        };
        let draw_parameters = DrawParameters {
            blend: Blend::alpha_blending(),
            ..DrawParameters::default()
        };

        {
            let cs = CHUNK_SIZE as f32;
            let view = draw.viewport.rect.scale(1.0 / cs, 1.0 / cs).round_out();
            let y_min = view.origin.y as i64;
            let y_max = view.origin.y as i64 + view.size.height as i64;
            let x_min = view.origin.x as i64;
            let x_max = view.origin.x as i64 + view.size.width as i64;
            for y in y_min..y_max {
                for x in x_min..x_max {
                    if let Ok(pos) = ChunkPos::try_from((x, y)) {
                        draw_debug!(draw.debug, DebugCategory::ChunkBorders, rect(
                        x as f32 * 16.0,
                        y as f32 * 16.0,
                        16.0,
                        16.0
                    ));

                        if let Some(render) = self.chunk_renderers.get_mut(&pos){
                            render.tick(pos, chunk, &self.layer_renderers, draw.debug)?;
                            render.drawer.draw(draw.frame, program, &uniforms, &draw_parameters)?;
                        } else {
                            self.chunk_renderers.insert(pos, ChunkRenderer {
                                drawer: draw.frontend.create_drawer()?,
                                builder: MeshBuilder::new(),
                                dirty: true,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct ChunkRenderer {
    drawer: MeshDrawer<PosTexVertex>,
    builder: MeshBuilder<PosTexVertex>,
    dirty: bool,
}

impl ChunkRenderer {
    pub fn tick(
        &mut self,
        pos: ChunkPos,
        chunks: &ChunkStorage,
        renderers: &LayerRenderers,
        debug: &mut Debug,
    ) -> Result<()> {
        if self.dirty {
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
            if let Some(chunk) = chunks.get(pos) {
                let mut neighbors = DirMap::new([None; 4]);
                for dir in Direction::values() {
                    if let Some(pos) = pos.checked_offset(dir) {
                        if let Some(chunk) = chunks.get(pos) {
                            neighbors[dir] = Some(chunk);
                        }
                    }
                }

                for (id, layer) in chunk.layers.iter() {
                    if let Some(renderer) = renderers.get(id) {
                        renderer.mesh_chunk_layer(
                            pos,
                            layer,
                            neighbors.map(|_, option| option.map(|c| c.layers.get(id))),
                            &mut self.builder,
                            debug,
                        );
                    }
                }
            }
            self.drawer.upload(&self.builder)?;
            self.builder.clear();
            self.dirty = false;
        }
        Ok(())
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
