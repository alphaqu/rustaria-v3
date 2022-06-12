use euclid::Rect;
use crate::renderer::atlas::Atlas;
use crate::renderer::builder::Quad;
use crate::renderer::PosTexVertex;

impl<U> Quad<PosTexVertex> for (Rect<f32, U>, Rect<f32, Atlas>)  {
	fn expand(self) -> [PosTexVertex; 4] {
		[
			PosTexVertex {
				position: [self.0.min_x(), self.0.min_y()],
				texture: [self.1.min_x(), self.1.min_y()],
			},
			PosTexVertex {
				position: [self.0.min_x(), self.0.max_y()],
				texture: [self.1.min_x(), self.1.max_y()],
			},
			PosTexVertex {
				position: [self.0.max_x(), self.0.max_y()],
				texture: [self.1.max_x(), self.1.max_y()],
			},
			PosTexVertex {
				position: [self.0.max_x(), self.0.min_y()],
				texture:[self.1.max_x(), self.1.min_y()],
			},
		]
	}
}