use euclid::{Rect, size2, vec2};
use mlua::{FromLua, Lua, LuaSerdeExt, Value};
use rustaria::api::prototype::{FactoryPrototype, Prototype};
use rustaria::api::util::lua_table;
use rustaria::chunk::ConnectionType;
use rustaria::debug::{DebugCategory, DebugRendererImpl};
use rustaria::draw_debug;
use rustaria::ty::block_pos::BlockPos;
use rustaria::ty::identifier::Identifier;
use rustaria::ty::WS;
use crate::Debug;
use crate::render::atlas::Atlas;
use crate::render::builder::MeshBuilder;
use crate::render::PosTexVertex;
use crate::render::world::chunk;

#[derive(Debug)]
pub struct BlockRendererPrototype {
    pub image: Identifier,
    pub connection_type: ConnectionType,
}
impl BlockRendererPrototype {
    pub fn create(&self, atlas: &Atlas) -> BlockRenderer {
        BlockRenderer {
            tex_pos:  atlas.get(&self.image),
            connection_type: self.connection_type
        }
    }
}

impl FromLua for BlockRendererPrototype {
    fn from_lua(lua_value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table = lua_table(lua_value)?;

        Ok(BlockRendererPrototype {
            image: table.get("image")?,
            connection_type: lua.from_value(table.get("connection_type")?)?
        })
    }
}

impl Prototype for BlockRendererPrototype {
    fn get_name() -> &'static str {
        "block_renderer"
    }
}

pub struct BlockRenderer {
    pub tex_pos: Rect<f32, Atlas>,
    pub connection_type: ConnectionType,
}

impl BlockRenderer {
    pub fn mesh(&self, pos: BlockPos, desc: &KindDesc, builder: &mut MeshBuilder<PosTexVertex>, debug: &mut Debug) {
        let mut texture = self.tex_pos;

        let variation = chunk::get_variation(pos) % ((texture.size.width / texture.size.height) as u32);
        let layout_width = texture.size.width / 3.0;

        let layout_height = texture.size.height;
        texture.origin.x += layout_width * variation as f32;

        texture.size.width = desc.uv.size.width * layout_width;
        texture.size.height = desc.uv.size.height * layout_height;
        texture.origin.x += desc.uv.origin.x * layout_width;
        texture.origin.y += desc.uv.origin.y * layout_height;
        let mut quad_pos = desc.rect;

        quad_pos.origin += size2(pos.x() as f32, pos.y() as f32);

        const VARIATION_COLORS: [u32; 3] = [0xff0000, 0x00ff00, 0x0000ff];
        draw_debug!(debug, DebugCategory::ChunkMeshing, vec2(pos.x() as f32 + 0.5, pos.y() as f32 + 0.5), VARIATION_COLORS[(variation % 3) as usize], 5.0, 0.5);
        builder.push_quad((quad_pos, texture));
    }
}


pub struct KindDesc {
    pub(crate) uv: Rect<f32, WS>,
    pub(crate) rect: Rect<f32, WS>,
}

