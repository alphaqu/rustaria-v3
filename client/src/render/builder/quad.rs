use std::ops::Add;
use euclid::Rect;

use crate::render::builder::Quad;
use crate::render::{PosColorVertex, PosTexVertex};

impl<P: Quad<[f32; 2]>, T: Quad<[f32; 2]>> Quad<PosTexVertex> for (P, T) {
    fn expand(self) -> [PosTexVertex; 4] {
	    let p = self.0.expand();
	    let t = self.1.expand();
        [
            PosTexVertex {
                position: p[0],
                texture: t[0],
            },
            PosTexVertex {
	            position: p[1],
	            texture: t[1],
            },
            PosTexVertex {
	            position: p[2],
	            texture: t[2],
            },
            PosTexVertex {
	            position: p[3],
	            texture: t[3],
            },
        ]
    }
}

impl<P: Quad<[f32; 2]>, C: Quad<[f32; 3]>> Quad<PosColorVertex> for (P, C) {
	fn expand(self) -> [PosColorVertex; 4] {
		let p = self.0.expand();
		let t = self.1.expand();
		[
			PosColorVertex {
				position: p[0],
				color: t[0],
			},
			PosColorVertex {
				position: p[1],
				color: t[1],
			},
			PosColorVertex {
				position: p[2],
				color: t[2],
			},
			PosColorVertex {
				position: p[3],
				color: t[3],
			},
		]
	}
}

impl<U, T: Copy + Add<T, Output = T>> Quad<[T; 2]> for Rect<T, U> {
    fn expand(self) -> [[T; 2]; 4] {
        [
            [self.min_x(), self.min_y()],
            [self.min_x(), self.max_y()],
            [self.max_x(), self.max_y()],
            [self.max_x(), self.min_y()],
        ]
    }
}

impl<V: Clone> Quad<V> for V {
	fn expand(self) -> [V; 4] {
		[
			self.clone(),
			self.clone(),
			self.clone(),
			self,
		]
	}
}

