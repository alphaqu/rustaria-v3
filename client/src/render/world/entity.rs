use euclid::{Rect, Vector2D};
use mlua::{FromLua, Lua, LuaSerdeExt, Value};
use rustaria::api::prototype::Prototype;
use rustaria::api::util::lua_table;
use rustaria::ty::identifier::Identifier;

use crate::{ClientApi, Frontend, PlayerSystem};
use rustaria::ty::WS;

use crate::render::atlas::Atlas;
use crate::render::ty::mesh_buffer::MeshDrawer;
use crate::render::ty::mesh_builder::MeshBuilder;
use crate::render::ty::vertex::PosTexVertex;
use eyre::Result;
use glium::{Blend, DrawParameters, Program, uniform};
use rustaria::api::registry::MappedRegistry;
use rustaria::world::entity::component::{PhysicsComponent, PositionComponent, PrototypeComponent};
use rustaria::world::entity::prototype::EntityPrototype;
use rustaria::world::entity::EntityWorld;
use crate::render::ty::draw::Draw;

pub struct WorldEntityRenderer {
    drawer: MeshDrawer<PosTexVertex>,
    renderers: MappedRegistry<EntityPrototype, Option<EntityRenderer>>

}

impl WorldEntityRenderer {
    pub fn new(api: &ClientApi, frontend: &Frontend, atlas: &Atlas) -> Result<WorldEntityRenderer> {
        Ok(WorldEntityRenderer {
            drawer: frontend.create_drawer()?,
            renderers: api.carrier.entity.map(|ident, _, _| {
                api.c_carrier
                    .entity_renderer
                    .ident_to_id(ident)
                    .map(|id| api.c_carrier.entity_renderer.get(id).create(atlas))
            }),
        })
    }

    pub fn draw(
        &mut self,
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
            if let Some(renderer) = self.renderers.get(prototype.id) {
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

pub struct EntityRendererPrototype {
    pub image: Identifier,
    pub panel: Rect<f32, WS>,
}

impl Prototype for EntityRendererPrototype {
    fn get_name() -> &'static str {
        "entity_renderer"
    }
}

impl FromLua for EntityRendererPrototype {
    fn from_lua(lua_value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table = lua_table(lua_value)?;

        Ok(EntityRendererPrototype {
            image: table.get("image")?,
            panel: lua.from_value(table.get("panel")?)?,
        })
    }
}

impl EntityRendererPrototype {
    pub fn create(&self, atlas: &Atlas) -> EntityRenderer {
        EntityRenderer {
            tex_pos: atlas.get(&self.image),
            panel: self.panel,
        }
    }
}

pub struct EntityRenderer {
    pub tex_pos: Rect<f32, Atlas>,
    pub panel: Rect<f32, WS>,
}

impl EntityRenderer {
    pub fn mesh(&self, pos: Vector2D<f32, WS>, builder: &mut MeshBuilder<PosTexVertex>) {
        let mut rect = self.panel;
        rect.origin += pos;
        builder.push_quad((rect, self.tex_pos));
    }
}
