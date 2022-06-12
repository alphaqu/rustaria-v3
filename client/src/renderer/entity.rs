use euclid::{Rect, Vector2D};

use rustaria::ty::WS;

use crate::renderer::atlas::Atlas;
use crate::renderer::builder::MeshBuilder;
use crate::renderer::PosTexVertex;

pub struct EntityRenderer {
	pub tex_pos: Rect<f32, Atlas>,
	pub panel: Rect<f32, WS>,
}

impl EntityRenderer {
	pub fn mesh(&self, pos: Vector2D<f32, WS>, builder: &mut MeshBuilder<PosTexVertex>) {
		let mut rect = self.panel;
		rect.origin += pos;
		builder.push_quad((
			rect,
			self.tex_pos,
		));
	}
}