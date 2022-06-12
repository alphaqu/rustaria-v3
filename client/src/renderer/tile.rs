use crate::renderer::builder::MeshBuilder;
use crate::renderer::PosTexVertex;
use euclid::{point2, Rect, size2};
use rustaria::ty::world_pos::WorldPos;
use crate::renderer::atlas::Atlas;

pub struct TileRenderer {
    pub tex_pos: Rect<f32, Atlas>,
}

impl TileRenderer {
    pub fn mesh(&self, pos: WorldPos, builder: &mut MeshBuilder<PosTexVertex>) {
        let mut texture = self.tex_pos;
        let tile_height = texture.size.height / 4.0;
        let tile_width = texture.size.width / 12.0;
        texture.size.width /= 12.0;
        texture.size.height /= 4.0;

        texture.origin.y += tile_height;
        texture.origin.x += tile_width;

        builder.push_quad((
            Rect::<f32, WorldPos>::new(
                point2(pos.x() as f32, pos.y() as f32),
                size2(1.0, 1.0),
            ),
            texture,
        ));
    }
}
