use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::PosColorVertex;
use crate::{Camera, Frontend};
use euclid::{rect, vec2, Rect, Vector2D};
use eyre::Result;
use glium::program::SourceCode;
use glium::{uniform, DrawParameters, Frame, Program};
use rustaria::debug::{DebugKind, DebugRendererImpl};
use rustaria::ty::WS;
use std::collections::HashSet;

pub struct DebugRenderer {
    program: Program,
    builder: MeshBuilder<PosColorVertex>,
    drawer: MeshDrawer<PosColorVertex>,

    line_size: f32,
    enabled_kinds: HashSet<DebugKind>,
}

impl DebugRenderer {
    pub fn new(frontend: &Frontend) -> Result<DebugRenderer> {
        Ok(DebugRenderer {
            program: frontend.create_program(SourceCode {
                vertex_shader: include_str!("./builtin/pos_color.vert.glsl"),
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: include_str!("./builtin/pos_color.frag.glsl"),
            })?,
            drawer: frontend.create_drawer()?,
            builder: MeshBuilder::new(),
            line_size: 0.1,
            enabled_kinds: Default::default(),
        })
    }

    pub fn enable(&mut self, kind: DebugKind) {
        self.enabled_kinds.insert(kind);
    }

    pub fn disable(&mut self, kind: DebugKind) {
        self.enabled_kinds.remove(&kind);
    }

    pub fn finish(&mut self) -> Result<()> {
        self.drawer.upload(&self.builder)?;
        self.builder.clear();
        Ok(())
    }

    pub fn draw(&mut self, frontend: &Frontend, camera: &Camera, frame: &mut Frame) -> Result<()> {
        let uniforms = uniform! {
            screen_ratio: frontend.screen_ratio,
            player_pos: camera.pos.to_array(),
            zoom: camera.zoom,
        };

        let draw_parameters = DrawParameters {
            ..DrawParameters::default()
        };
        self.drawer
            .draw(frame, &self.program, &uniforms, &draw_parameters)?;
        Ok(())
    }

    fn get_color(&self, color: u32) -> [f32; 3] {
        let b = (color & 0xff) as f32 / 255.0;
        let g = ((color >> 8) & 0xff) as f32 / 255.0;
        let r = ((color >> 16) & 0xff) as f32 / 255.0;

        fn convert(f: f32) -> f32 {
            if f <= 0.04045 {
                f / 12.92
            } else {
                let a = 0.055_f32;

                ((f + a) * (1. + a).powf(-1.)).powf(2.4)
            }
        }

        [convert(r), convert(g), convert(b)]
    }

    fn enabled(&self, kind: DebugKind) -> bool {
        self.enabled_kinds.contains(&kind)
    }
}

impl DebugRendererImpl for DebugRenderer {
    fn draw_rect(&mut self, kind: DebugKind, color: u32, rect: Rect<f32, WS>) {
        if self.enabled(kind) {
            let color = self.get_color(color);
            self.builder.push_quad((rect, color));
        }
    }

    fn draw_hrect(&mut self, kind: DebugKind, color: u32, rect: Rect<f32, WS>) {
        if self.enabled(kind) {
            let color = self.get_color(color);

            // top
            {
                let mut rect = rect;
                let height = rect.height();
                rect.origin.y += height - self.line_size;
                rect.size.height = self.line_size;
                self.builder.push_quad((rect, color));
            }

            // bottom
            {
                let mut rect = rect;
                rect.size.height = self.line_size;
                self.builder.push_quad((rect, color));
            }

            // right
            {
                let mut rect = rect;
                let width = rect.width();
                rect.origin.x += width - self.line_size;
                rect.size.width = self.line_size;
                self.builder.push_quad((rect, color));
            }

            // left
            {
                let mut rect = rect;
                rect.size.width = self.line_size;
                self.builder.push_quad((rect, color));
            }
        }
    }

    fn draw_line(
        &mut self,
        kind: DebugKind,
        color: u32,
        start: Vector2D<f32, WS>,
        stop: Vector2D<f32, WS>,
    ) {
        if self.enabled(kind) {
            let color = self.get_color(color);
            let furbertensvector = stop - start;
            let not_furbertensvector =
                vec2::<_, WS>(-furbertensvector.y, furbertensvector.x).normalize();

            let t_start = start + (not_furbertensvector * (self.line_size / 2.0));
            let b_start = start + (-not_furbertensvector * (self.line_size / 2.0));
            let t_stop = stop + (not_furbertensvector * (self.line_size / 2.0));
            let b_stop = stop + (-not_furbertensvector * (self.line_size / 2.0));

            self.builder.push_quad((
                [
                    t_start.to_array(),
                    b_start.to_array(),
                    b_stop.to_array(),
                    t_stop.to_array(),
                ],
                color,
            ));
        }
    }

    fn draw_point(&mut self, kind: DebugKind, color: u32, pos: Vector2D<f32, WS>) {
        if self.enabled(kind) {
            let color = self.get_color(color);
            let half = (self.line_size / 2.0);
            self.builder.push_quad((
                rect::<_, WS>(pos.x - half, pos.y - half, self.line_size, self.line_size),
                color,
            ))
        }
    }
}
