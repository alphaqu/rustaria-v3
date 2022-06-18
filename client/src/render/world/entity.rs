use std::collections::HashSet;
use euclid::{Rect, Vector2D};
use rustaria::api::prototype::{LuaPrototype, Prototype};
use rustaria::ty::identifier::Identifier;

use crate::{ClientApi, Frontend, PlayerSystem};
use rustaria::ty::WS;

use crate::render::atlas::Atlas;
use crate::render::ty::mesh_buffer::MeshDrawer;
use crate::render::ty::mesh_builder::MeshBuilder;
use crate::render::ty::vertex::PosTexVertex;
use eyre::Result;
use glium::{Blend, DrawParameters, Program, uniform};
use rustaria::api::luna::table::LunaTable;
use rustaria::ty::id::Id;
use rustaria::util::blake3::Hasher;
use rustaria::world::entity::component::{PhysicsComponent, PositionComponent, PrototypeComponent};
use rustaria::world::entity::EntityWorld;
use crate::render::ty::draw::Draw;

pub struct WorldEntityRenderer {
    drawer: MeshDrawer<PosTexVertex>,
}

impl WorldEntityRenderer {
    pub fn new(frontend: &Frontend) -> Result<WorldEntityRenderer> {
        Ok(WorldEntityRenderer {
            drawer: frontend.create_drawer()?,
        })
    }

    pub fn draw(
        &mut self,
        api: &ClientApi,
        player: &PlayerSystem,
        entity: &EntityWorld,
        program: &Program,
        draw: &mut Draw,
    ) -> Result<()> {
        let mut builder = MeshBuilder::new();
        for (entity, (position, prototype, physics)) in entity
            .storage
            .query::<(&PositionComponent, &PrototypeComponent, &PhysicsComponent)>()
            .iter()
        {

            if let Some(renderer) =  api.c_carrier.entity_renderer.get(prototype.id) {
                // If this entity is our player, we use its predicted position instead of its server confirmed position.
                let (mut position, mut vel) = (position.pos, physics.vel - physics.accel);
                if let Some(player_entity) = player.server_player {
                    if player_entity == entity {
                        if let Some(pos) = player.get_comp::<PositionComponent>() {
                            position = pos.pos;
                        }
                        if let Some(physics) = player.get_comp::<PhysicsComponent>() {
                            vel = physics.vel - physics.accel;
                        }
                    }
                }
                renderer.mesh((position - vel).lerp(position, draw.timing.delta()), &mut builder);
            }
        }
        self.drawer.upload(&builder)?;

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

        self.drawer.draw(draw.frame, program, &uniforms, &draw_parameters)?;
        Ok(())
    }
}

pub struct LuaEntityRendererPrototype {
    pub image: Identifier,
    pub panel: Rect<f32, WS>,
}

impl LuaEntityRendererPrototype {
    pub fn bake(&self, atlas: &Atlas) -> EntityRendererPrototype {
        EntityRendererPrototype {
            image: atlas.get(&self.image),
            panel: self.panel,
        }
    }

    pub fn get_sprites(&self, sprites: &mut HashSet<Identifier>) {
        sprites.insert(self.image.clone());
    }
}

impl LuaPrototype for LuaEntityRendererPrototype {
    type Output = EntityRendererPrototype;

    fn get_name() -> &'static str {
        "entity_renderer"
    }

    fn from_lua(table: LunaTable, _: &mut Hasher) -> Result<Self> {
        Ok(LuaEntityRendererPrototype {
            image: table.get("image")?,
            panel: table.get_ser("panel")?,
        })
    }
}

pub struct EntityRendererPrototype {
    pub image: Rect<f32, Atlas>,
    pub panel: Rect<f32, WS>,
}

impl EntityRendererPrototype {
    pub fn mesh(&self, pos: Vector2D<f32, WS>, builder: &mut MeshBuilder<PosTexVertex>) {
        let mut rect = self.panel;
        rect.origin += pos;
        builder.push_quad((rect, self.image));
    }
}

impl Prototype for EntityRendererPrototype {

}