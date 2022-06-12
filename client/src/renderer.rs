use std::collections::HashMap;

use eyre::Result;
use glium::{Blend, DrawParameters, Frame, implement_vertex, Program, uniform};
use glium::program::SourceCode;

use rustaria::api::{Assets, Carrier};
use rustaria::api::registry::MappedRegistry;
use rustaria::chunk::Chunk;
use rustaria::chunk::tile::TilePrototype;
use rustaria::entity::component::{PositionComponent, PrototypeComponent};
use rustaria::entity::EntityStorage;
use rustaria::entity::prototype::EntityPrototype;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::world_pos::WorldPos;

use crate::{Frontend, PlayerSystem};
use crate::renderer::atlas::Atlas;
use crate::renderer::buffer::MeshDrawer;
use crate::renderer::builder::MeshBuilder;
use crate::renderer::entity::EntityRenderer;
use crate::renderer::tile::TileRenderer;

mod atlas;
mod buffer;
mod builder;
mod entity;
mod tile;

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

pub(crate) struct WorldRenderer {
    pos_color_program: Program,
    atlas: Atlas,

    chunk_drawer: MeshDrawer<PosTexVertex>,
    chunk_tile_renderers: MappedRegistry<TilePrototype, Option<TileRenderer>>,

    entity_drawer: MeshDrawer<PosTexVertex>,
    entity_renderers: MappedRegistry<EntityPrototype, Option<EntityRenderer>>,
}

impl WorldRenderer {
    pub fn new(frontend: &Frontend, carrier: &Carrier, assets: &Assets) -> Result<Self> {
        let mut images = Vec::new();
        for tile in carrier.tile.entries() {
            if let Some(image) = &tile.image {
                images.push(image.clone());
            }
        }

        for tile in carrier.entity.entries() {
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

        let entity_renderers = carrier.entity.map(|_, entity| {
            entity.image.as_ref().map(|image| EntityRenderer {
                tex_pos: atlas.get(image),
                panel: entity.panel,
            })
        });
        Ok(Self {
            chunk_drawer: MeshDrawer::new(frontend)?,
            pos_color_program: Program::new(
                &frontend.ctx,
                SourceCode {
                    vertex_shader: include_str!("renderer/builtin/pos_tex.vert.glsl"),
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: include_str!("renderer/builtin/pos_tex.frag.glsl"),
                },
            )?,
            atlas,
            chunk_tile_renderers: tile_renderers,
            entity_drawer: MeshDrawer::new(frontend)?,
            entity_renderers,
        })
    }

    pub fn tick(
        &mut self,
        entities: &EntityStorage,
        player: &PlayerSystem,
        chunks: &HashMap<ChunkPos, Chunk>,
    ) -> Result<()> {
        let mut builder = MeshBuilder::new();
        for (pos, chunk) in chunks {
            chunk.tile.entries(|entry, tile| {
                if let Some(renderer) = self.chunk_tile_renderers.get(tile.id) {
                    renderer.mesh(WorldPos::new(*pos, entry), &mut builder);
                }
            });
        }
        self.chunk_drawer.upload(&builder)?;

        let mut builder = MeshBuilder::new();
        for (entity, (position, prototype)) in entities
            .query::<(&PositionComponent, &PrototypeComponent)>()
            .iter()
        {
            if let Some(renderer) = self.entity_renderers.get(prototype.id) {
                // If this entity is our player, we use its predicted position instead of its server confirmed position.
                if let Some(player_entity) = player.server_player {
                    if player_entity == entity {
                        renderer.mesh(player.get_pos(), &mut builder);
                        continue;
                    }
                }
                renderer.mesh(position.pos, &mut builder);
            }
        }
        self.entity_drawer.upload(&builder)?;
        Ok(())
    }

    pub fn draw(&mut self, frontend: &Frontend, camera: Camera, frame: &mut Frame) -> Result<()> {
        let uniforms = uniform! {
            screen_ratio: frontend.screen_ratio,
            atlas: &self.atlas.texture,
            player_pos: camera.pos,
            zoom: camera.zoom,
        };

        let draw_parameters = DrawParameters {
            blend: Blend::alpha_blending(),
            ..DrawParameters::default()
        };
        self.chunk_drawer
            .draw(frame, &self.pos_color_program, &uniforms, &draw_parameters)?;
        self.entity_drawer
            .draw(frame, &self.pos_color_program, &uniforms, &draw_parameters)?;
        Ok(())
    }
}
