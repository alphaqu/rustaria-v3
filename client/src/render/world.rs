use glium::program::SourceCode;
use glium::{uniform, Blend, DrawParameters, Frame, Program};

use rustaria::api::registry::MappedRegistry;
use rustaria::api::Api;
use rustaria::entity::component::{PositionComponent, PrototypeComponent};
use rustaria::entity::prototype::EntityPrototype;
use rustaria::world::World;

use crate::render::atlas::Atlas;
use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::PosTexVertex;
use crate::{Camera, ClientApi, Debug, Frontend, PlayerSystem};
use chunk::WorldChunkRenderer;
use entity::EntityRenderer;
use crate::render::world::entity::WorldEntityRenderer;

pub mod chunk;
pub mod entity;
pub mod neighbor;

pub(crate) struct WorldRenderer {
    pos_color_program: Program,
    atlas: Atlas,

    dirty_world: bool,
    chunk_renderer: WorldChunkRenderer,
    entity_renderer: WorldEntityRenderer,
}

impl WorldRenderer {
    pub fn new(frontend: &Frontend, api: &ClientApi) -> eyre::Result<Self> {
        let mut image_locations = Vec::new();
        for prototype in api.c_carrier.block_layer_renderer.entries() {
            for entry in prototype.registry.entries() {
                image_locations.push(entry.image.clone());
            }
        }

        for renderer in api.c_carrier.entity_renderer.entries() {
            image_locations.push(renderer.image.clone());
        }

        let atlas = Atlas::new(frontend, api, &image_locations)?;
        Ok(Self {
            pos_color_program: Program::new(
                &frontend.ctx,
                SourceCode {
                    vertex_shader: include_str!("builtin/pos_tex.vert.glsl"),
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: include_str!("builtin/pos_tex.frag.glsl"),
                },
            )?,
            chunk_renderer: WorldChunkRenderer::new(api, frontend, &atlas)?,
            entity_renderer: WorldEntityRenderer::new(api, frontend, &atlas)?,
            atlas,
            dirty_world: true,
        })
    }

    pub fn dirty_world(&mut self) {
        self.chunk_renderer.remesh_world();
    }

    pub fn tick(
        &mut self,
        player: &PlayerSystem,
        world: &World,
        debug: &mut Debug,
    ) -> eyre::Result<()> {
        let player_pos = player.get_pos();
        self.chunk_renderer.tick(player_pos, &world.chunk, debug)?;
        self.entity_renderer.tick(player, &world.entity)?;
        Ok(())
    }

    pub fn draw(
        &mut self,
        frontend: &Frontend,
        camera: &Camera,
        frame: &mut Frame,
    ) -> eyre::Result<()> {
        self.chunk_renderer.draw(frontend, &self.atlas, camera, &self.pos_color_program, frame)?;
        self.entity_renderer.draw(frontend, &self.atlas, camera, &self.pos_color_program, frame)?;
        Ok(())
    }
}
