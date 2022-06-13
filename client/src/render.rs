use glium::implement_vertex;

pub mod atlas;
pub mod entity;
pub mod tile;
pub mod buffer;
pub mod builder;
pub mod debug;
pub mod world;

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
    color: [f32; 3],
}

implement_vertex!(PosColorVertex, position, color);


#[derive(Copy, Clone)]
pub struct Camera {
    pub pos: [f32; 2],
    pub zoom: f32,
}
