use std::collections::HashMap;

use eyre::Result;
use glium::program::SourceCode;
use glium::{implement_vertex, uniform, Blend, DrawParameters, Frame, Program};

use rustaria::api::registry::MappedRegistry;
use rustaria::api::{Assets, Carrier};
use rustaria::chunk::tile::TilePrototype;
use rustaria::chunk::{Chunk};
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::world_pos::WorldPos;

use crate::renderer::atlas::Atlas;
use crate::renderer::buffer::MeshDrawer;
use crate::renderer::builder::MeshBuilder;
use crate::renderer::tile::TileRenderer;
use crate::Frontend;

mod atlas;
mod buffer;
mod builder;
mod tile;
mod entity;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PosTexVertex {
    position: [f32; 2],
    texture: [f32; 2],
}

implement_vertex!(PosTexVertex, position, texture);

#[derive(Copy, Clone)]
pub struct Camera {
    pub pos: [f32; 2],
    pub zoom: f32,
}

pub struct WorldRenderer {
    pos_color_program: Program,
    atlas: Atlas,

    chunk_drawer: MeshDrawer<PosTexVertex>,
    chunk_tile_renderers: MappedRegistry<TilePrototype, Option<TileRenderer>>,

    entity_drawer: MeshDrawer<PosTexVertex>,


}

impl WorldRenderer {
    pub fn new(frontend: &Frontend, carrier: &Carrier, assets: &Assets) -> Result<Self> {
        let mut images = Vec::new();
        for tile in carrier.tile.entries() {
            if let Some(image) = &tile.image {
                images.push(image.clone());
            }
        }

        let atlas = Atlas::new(frontend, assets, &images)?;

        let tile_renderers = carrier.tile.map(|_, tile| {
            tile.image.as_ref().map(|image| TileRenderer {
                tex_pos: atlas.get(image),
            })
        });
        Ok(Self {
            chunk_drawer: MeshDrawer::new(frontend)?,
            pos_color_program: Program::new(
                &frontend.ctx,
                SourceCode {
                    vertex_shader: include_str!("renderer/shader/pos_tex.vert.glsl"),
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: include_str!("renderer/shader/pos_tex.frag.glsl"),
                },
            )?,
            atlas,
            chunk_tile_renderers: tile_renderers,
        })
    }

    pub fn tick(&mut self, chunks: &HashMap<ChunkPos, Chunk>) -> Result<()> {
        let mut builder = MeshBuilder::new();
        for (pos, chunk) in chunks {
            chunk.tile.entries(|entry, tile| {
                if let Some(renderer) = self.chunk_tile_renderers.get(tile.id) {
                    renderer.mesh(WorldPos::new(*pos, entry), &mut builder);
                }
            });
        }
        self.chunk_drawer.upload(&builder)?;
        Ok(())
    }

    pub fn draw(&mut self, frontend: &Frontend, camera: Camera, frame: &mut Frame) -> Result<()> {
        self.chunk_drawer.draw(
            frame,
            &self.pos_color_program,
            &uniform! {
                screen_ratio: frontend.screen_ratio,
                atlas: &self.atlas.texture,
                player_pos: camera.pos,
                zoom: camera.zoom,
            },
            &DrawParameters {
                blend: Blend::alpha_blending(),
                ..DrawParameters::default()
            },
        )?;
        Ok(())
    }
}
