use glium::{Blend, DrawParameters, Frame, Program, uniform};
use glium::program::SourceCode;
use euclid::vec2;
use rustaria::api::Carrier;
use rustaria::api::registry::MappedRegistry;
use rustaria::chunk::CHUNK_SIZE;
use rustaria::chunk::storage::ChunkStorage;
use rustaria::chunk::tile::TilePrototype;
use rustaria::entity::component::{PositionComponent, PrototypeComponent};
use rustaria::entity::EntityStorage;
use rustaria::entity::prototype::EntityPrototype;
use rustaria::ty::chunk_pos::ChunkPos;
use rustaria::ty::world_pos::WorldPos;
use crate::{Camera, Frontend, PlayerSystem};
use crate::render::atlas::Atlas;
use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::entity::EntityRenderer;
use crate::render::PosTexVertex;
use crate::render::tile::TileRenderer;

pub(crate) struct WorldRenderer {
    pos_color_program: Program,
    atlas: Atlas,

    chunk_dirty: bool,
    chunk_drawer: MeshDrawer<PosTexVertex>,
    chunk_tile_renderers: MappedRegistry<TilePrototype, Option<TileRenderer>>,

    entity_drawer: MeshDrawer<PosTexVertex>,
    entity_renderers: MappedRegistry<EntityPrototype, Option<EntityRenderer>>,
}

impl WorldRenderer {
    pub fn new(frontend: &Frontend, carrier: &Carrier) -> eyre::Result<Self> {
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

        let atlas = Atlas::new(frontend, carrier, &images)?;
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
                    vertex_shader: include_str!("builtin/pos_tex.vert.glsl"),
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: include_str!("builtin/pos_tex.frag.glsl"),
                },
            )?,
            atlas,
            chunk_tile_renderers: tile_renderers,
            entity_drawer: MeshDrawer::new(frontend)?,
            entity_renderers,
            chunk_dirty: true
        })
    }

    pub fn dirty_world(&mut self) {
        self.chunk_dirty = true;
    }

    pub fn tick(
        &mut self,
        entities: &EntityStorage,
        player: &PlayerSystem,
        chunks: &ChunkStorage,
    ) -> eyre::Result<()> {
        if self.chunk_dirty {
            let mut builder = MeshBuilder::new();
            let pos = player.get_pos();
            for y in -4..4{
                for x in -4..4 {
                    if let Ok(pos) =ChunkPos::try_from(pos + vec2(x as f32 * CHUNK_SIZE as f32, y as f32 * CHUNK_SIZE as f32))  {
                        if let Some(chunk) = chunks.get(pos) {
                            chunk.tile.entries(|entry, tile| {
                                if let Some(renderer) = self.chunk_tile_renderers.get(tile.id) {
                                    renderer.mesh(WorldPos::new(pos, entry), &mut builder);
                                }
                            });
                        }
                    }
                }
            }

            self.chunk_drawer.upload(&builder)?;
            self.chunk_dirty = false;
        }

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

    pub fn draw(&mut self, frontend: &Frontend, camera: &Camera, frame: &mut Frame) -> eyre::Result<()> {
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
