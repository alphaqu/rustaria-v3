use euclid::{rect, Rect, Vector2D};
use glium::implement_vertex;
use rustaria::ty::WS;
use crate::Frontend;

pub mod atlas;
pub mod buffer;
pub mod builder;
pub mod world;
pub mod draw;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PosTexVertex {
    position: [f32; 2],
    texture: [f32; 2],
}

implement_vertex!(PosTexVertex, position, texture);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PosColorVertex {
    position: [f32; 2],
    color: [f32; 4],
}

implement_vertex!(PosColorVertex, position, color);


#[derive(Copy, Clone)]
pub struct Viewport {
    pub pos: Vector2D<f32, WS>,
    pub zoom: f32,
    pub rect: Rect<f32, WS>,
}

impl Viewport {
    pub fn new(frontend: &Frontend, pos: Vector2D<f32, WS>, zoom: f32) -> Viewport {
        let mut viewport = Viewport {
            pos,
            zoom,
            rect: Rect::zero(),
        };

        viewport.recompute_rect(frontend);
        viewport
    }

    pub fn recompute_rect(&mut self, frontend: &Frontend) {
        let w = self.zoom / frontend.aspect_ratio;
        let h = self.zoom;
        self.rect = rect(self.pos.x - w, self.pos.y - h, w * 2.0, h * 2.0);
    }
}