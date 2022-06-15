use euclid::Vector2D;
use glium::implement_vertex;
use rustaria::ty::WS;

pub mod atlas;
pub mod entity;
pub mod buffer;
pub mod builder;
pub mod debug;
pub mod world;
mod neighbor;
mod chunk;

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
pub struct Camera {
    pub pos: Vector2D<f32, WS>,
    pub zoom: f32,
}
