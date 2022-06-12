use std::collections::HashMap;
use std::ops::Index;

use euclid::{point2, Rect, size2, Vector2D};
use eyre::Result;
use glfw::Glfw;
use glium::{Blend, DrawParameters, Frame, implement_vertex, Program, uniform};
use glium::program::SourceCode;

use rustaria::api::{Assets, Carrier};
use rustaria::api::registry::MappedRegistry;
use rustaria::chunk::{Chunk, CHUNK_SIZE};
use rustaria::chunk::tile::TilePrototype;
use rustaria::ty::{ChunkPos, WorldPos, WS};

use crate::Frontend;
use crate::renderer::atlas::Atlas;
use crate::renderer::buffer::MeshDrawer;
use crate::renderer::builder::MeshBuilder;
use crate::renderer::tile::TileRenderer;

mod buffer;
mod builder;
mod tile;
mod atlas;

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
    drawer: MeshDrawer<PosTexVertex>,
    pos_color_program: Program,

    atlas: Atlas,
    tile_renderers: MappedRegistry<TilePrototype, TileRenderer>,
}

impl WorldRenderer {
    pub fn new(frontend: &Frontend, carrier: &Carrier, assets: &Assets) -> Result<Self> {
        let mut images = Vec::new();
        for tile in carrier.tile.entries() {
            images.push(tile.image.clone());
        }

        let atlas = Atlas::new(frontend, assets, &images)?;


        let tile_renderers = carrier.tile.map(|tile| {
            TileRenderer { tex_pos: atlas.get(&tile.image) }
        });
        Ok(Self {
            drawer: MeshDrawer::new(frontend)?,
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
            tile_renderers,
        })
    }

    pub fn tick(&mut self, chunks: &HashMap<ChunkPos, Chunk>) -> Result<()> {
        let mut builder = MeshBuilder::new();
        for (pos, chunk) in chunks {
            chunk.tile.entries(|entry, tile| {
                self.tile_renderers
                    .get(tile.id)
                    .mesh(WorldPos::new(*pos, entry), &mut builder);
            });
        }
        self.drawer.upload(&builder)?;
        Ok(())
    }

    pub fn draw(&mut self, frontend: &Frontend, camera: Camera, frame: &mut Frame) -> Result<()> {
        self.drawer.draw(
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
