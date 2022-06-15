use euclid::{point2, Rect, size2};
use rustaria::chunk::ConnectionType;

use rustaria::ty::block_pos::BlockPos;

use crate::render::atlas::Atlas;
use crate::render::builder::MeshBuilder;
use crate::render::neighbor::SpriteConnectionKind;
use crate::render::PosTexVertex;

pub struct TileRenderer {
    pub tex_pos: Rect<f32, Atlas>,
    pub connection_type: ConnectionType,
}

impl TileRenderer {
    pub fn mesh(&self, pos: BlockPos, kind: SpriteConnectionKind, builder: &mut MeshBuilder<PosTexVertex>) {
        let mut texture = self.tex_pos;
        let tile_height = texture.size.height / 4.0;
        let tile_width = texture.size.width / 12.0;
        texture.size.width /= 12.0;
        texture.size.height /= 4.0;

        let kind_pos = Self::get_pos(kind);
        texture.origin.x += tile_width * kind_pos[0] as f32;
        texture.origin.y += tile_height * (3 - kind_pos[1]) as f32;

        builder.push_quad((
            Rect::<f32, BlockPos>::new(
                point2(pos.x() as f32, pos.y() as f32),
                size2(1.0, 1.0),
            ),
            texture,
        ));
    }

    fn get_pos(kind: SpriteConnectionKind) -> [u32; 2] {
        match kind {
            SpriteConnectionKind::Solid => [1, 1],
            SpriteConnectionKind::Lonely => [3, 3],
            SpriteConnectionKind::Vertical => [3, 1],
            SpriteConnectionKind::Horizontal => [1, 3],
            SpriteConnectionKind::CapTop => [3, 0],
            SpriteConnectionKind::CapLeft => [0, 3],
            SpriteConnectionKind::CapDown => [3, 2],
            SpriteConnectionKind::CapRight => [2, 3],
            SpriteConnectionKind::WallTop => [1, 0],
            SpriteConnectionKind::WallDown => [1, 2],
            SpriteConnectionKind::WallLeft => [0, 1],
            SpriteConnectionKind::WallRight => [2, 1],
            SpriteConnectionKind::CornerTopLeft => [0, 0],
            SpriteConnectionKind::CornerTopRight => [2, 0],
            SpriteConnectionKind::CornerDownLeft => [0, 2],
            SpriteConnectionKind::CornerDownRight => [2, 2],
        }
    }
}
