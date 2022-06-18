use hecs::EntityBuilder;
use mlua::{FromLua, Lua, LuaSerdeExt, Table, Value};
use crate::api::luna::table::LunaTable;

use crate::ty::id::Id;
use crate::api::prototype::{FactoryPrototype, LuaPrototype, Prototype};
use crate::api::util;
use crate::util::blake3::Hasher;
use crate::world::entity::component::{CollisionComponent, GravityComponent, HumanoidComponent, PhysicsComponent, PositionComponent, PrototypeComponent};

#[derive(Debug)]
pub struct EntityPrototype {
    pub position: PositionComponent,
    pub velocity: Option<PhysicsComponent>,
    pub collision: Option<CollisionComponent>,
    pub humanoid: Option<HumanoidComponent>,
    pub gravity: Option<GravityComponent>,
}

impl LuaPrototype for EntityPrototype {
    type Output = EntityPrototype;

    fn get_name() -> &'static str {
        "entity"
    }

    fn from_lua(table: LunaTable, _: &mut Hasher) -> eyre::Result<Self> {
        Ok(EntityPrototype {
            position: table.get_ser("position")?,
            velocity: table.get_ser("velocity")?,
            collision:  table.get_ser("collision")?,
            humanoid: table.get_ser("humanoid")?,
            gravity: table.get_ser("gravity")?
        })
    }
}

impl Prototype for EntityPrototype {

}
impl FactoryPrototype for EntityPrototype {
    type Item = EntityBuilder;
    fn create(&self, id: Id<Self>) -> Self::Item {
        let mut builder = EntityBuilder::new();
        builder.add(self.position.clone());
        builder.add(PrototypeComponent { id });
        if let Some(comp) = self.velocity.as_ref() {
            builder.add(comp.clone());
        };
        if let Some(comp) = self.collision.as_ref() {
            builder.add(comp.clone());
        };
        if let Some(comp) = self.humanoid.as_ref() {
            builder.add(comp.clone());
        };
        if let Some(comp) = self.gravity.as_ref() {
            builder.add(comp.clone());
        };
        builder
    }
}

impl FromLua for EntityPrototype {
    fn from_lua(lua_value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table = util::lua_table(lua_value)?;
        Ok(EntityPrototype {
            position: lua.from_value(table.get("position")?)?,
            velocity: lua.from_value(table.get("velocity")?)?,
            collision:  lua.from_value(table.get("collision")?)?,
            humanoid: lua.from_value(table.get("humanoid")?)?,
            gravity: lua.from_value(table.get("gravity")?)?
        })
    }
}
