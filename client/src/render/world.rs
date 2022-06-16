use glium::program::SourceCode;
use glium::{uniform, Blend, DrawParameters, Frame, Program};

use rustaria::api::registry::MappedRegistry;
use rustaria::api::{Api, Resources};
use rustaria::chunk::storage::ChunkStorage;
use rustaria::entity::component::{PositionComponent, PrototypeComponent};
use rustaria::entity::prototype::EntityPrototype;
use rustaria::entity::EntityStorage;
use rustaria::world::World;

use crate::render::atlas::Atlas;
use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::chunk::ChunkRenderer;
use crate::render::entity::EntityRenderer;
use crate::render::PosTexVertex;
use crate::{Camera, Debug, Frontend, PlayerSystem};

pub(crate) struct WorldRenderer {
    pos_color_program: Program,
    atlas: Atlas,

    dirty_world: bool,
    chunk_renderer: ChunkRenderer,

    entity_drawer: MeshDrawer<PosTexVertex>,
    entity_renderers: MappedRegistry<EntityPrototype, Option<EntityRenderer>>,
}

impl WorldRenderer {
    pub fn new(frontend: &Frontend, api: &Api) -> eyre::Result<Self> {
        let mut image_locations = Vec::new();
        for layer in api.carrier.block_layer.entries() {
            for entry in layer.registry.entries() {
                if let Some(image) = &entry.image {
                    image_locations.push(image.clone());
                }
            }
        }

        for tile in api.carrier.entity.entries() {
            if let Some(image) = &tile.image {
                image_locations.push(image.clone());
            }
        }

        let atlas = Atlas::new(frontend, api, &image_locations)?;

        let entity_renderers = api.carrier.entity.map(|_, entity| {
            entity.image.as_ref().map(|image| EntityRenderer {
                tex_pos: atlas.get(image),
                panel: entity.panel,
            })
        });
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
            chunk_renderer: ChunkRenderer::new(api, frontend, &atlas)?,
            entity_drawer: MeshDrawer::new(frontend)?,
            atlas,
            entity_renderers,
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

        let mut builder = MeshBuilder::new();
        for (entity, (position, prototype)) in world.entity.storage
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

    pub fn draw(
        &mut self,
        frontend: &Frontend,
        camera: &Camera,
        frame: &mut Frame,
    ) -> eyre::Result<()> {
        self.chunk_renderer.draw(
            frontend,
            &self.atlas,
            camera,
            &self.pos_color_program,
            frame,
        )?;
        let uniforms = uniform! {
            screen_ratio: frontend.screen_ratio,
            atlas: &self.atlas.texture,
            player_pos: camera.pos.to_array(),
            zoom: camera.zoom,
        };

        let draw_parameters = DrawParameters {
            blend: Blend::alpha_blending(),
            ..DrawParameters::default()
        };
        self.entity_drawer
            .draw(frame, &self.pos_color_program, &uniforms, &draw_parameters)?;
        Ok(())
    }
}
