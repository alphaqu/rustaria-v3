use glium::{Frame, Program};
use glium::program::SourceCode;

use chunk::WorldChunkRenderer;
use rustaria::world::World;

use crate::{ClientApi, Debug, Frontend, PlayerSystem, Timing};
use crate::render::atlas::Atlas;
use crate::render::ty::draw::Draw;
use crate::render::ty::viewport::Viewport;
use crate::render::world::entity::WorldEntityRenderer;

pub mod chunk;
pub mod entity;
pub mod neighbor;

pub struct WorldRenderer {
    pos_color_program: Program,
    dirty_world: bool,
    chunk_renderer: WorldChunkRenderer,
    entity_renderer: WorldEntityRenderer,
}

impl WorldRenderer {
    pub fn reset(&mut self) {
        self.chunk_renderer.dirty();
    }
}

impl WorldRenderer {
    pub fn new(frontend: &Frontend, api: &ClientApi) -> eyre::Result<Self> {
        //let mut image_locations = Vec::new();
        //for prototype in api.c_carrier.block_layer_renderer.entries() {
        //    for entry in prototype.registry.entries() {
        //        image_locations.push(entry.image.clone());
        //    }
        //}
//
        //for renderer in api.c_carrier.entity_renderer.entries() {
        //    image_locations.push(renderer.image.clone());
        //}
//
        //let atlas = Atlas::new(frontend, api, &image_locations)?;
        Ok(Self {
            pos_color_program: Program::new(
                &frontend.ctx,
                SourceCode {
                    vertex_shader: include_str!("../builtin/pos_tex.vert.glsl"),
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: include_str!("../builtin/pos_tex.frag.glsl"),
                },
            )?,
            chunk_renderer: WorldChunkRenderer::new()?,
            entity_renderer: WorldEntityRenderer::new(frontend)?,
            dirty_world: true,
        })
    }

    pub fn tick(
        &mut self,
        frontend: &Frontend,
        player: &PlayerSystem,
        world: &World,
        debug: &mut Debug,
    ) -> eyre::Result<()> {
        self.chunk_renderer.tick(frontend, &world.chunks)?;
       // self.entity_renderer.tick(player, &world.entity)?;
        Ok(())
    }

    pub fn draw(
        &mut self,
        api: &ClientApi,
        frontend: &Frontend,
        player: &PlayerSystem,
        world: &World,
        frame: &mut Frame,
        viewport: &Viewport,
        debug: &mut Debug,
        timing: &Timing,
    ) -> eyre::Result<()> {
        let mut draw = Draw { frame, viewport, atlas: api.atlas.as_ref().unwrap(), frontend, debug, timing };
        self.chunk_renderer.draw(api,&world.chunks, &self.pos_color_program, &mut draw)?;
        self.entity_renderer.draw(api, player, &world.entities, &self.pos_color_program, &mut draw)?;
        Ok(())
    }
}
