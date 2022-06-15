use crate::render::buffer::MeshDrawer;
use crate::render::builder::MeshBuilder;
use crate::render::PosColorVertex;
use crate::{Camera, Frontend};
use euclid::{rect, vec2, Rect, Vector2D};
use eyre::Result;
use glium::program::SourceCode;
use glium::{uniform, Blend, DrawParameters, Frame, Program};
use rustaria::debug::{DebugCategory, DebugDraw, DebugEvent, DebugRendererImpl};
use rustaria::ty::WS;
use rustaria::TPS;
use std::collections::HashSet;
use std::time::Duration;

pub struct DebugRenderer {
    program: Program,
    builder: MeshBuilder<PosColorVertex>,
    drawer: MeshDrawer<PosColorVertex>,
    events: Vec<DebugEvent>,

    line_size: f32,

    enabled_kinds: HashSet<DebugCategory>,
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
            events: vec![],
        })
    }

    pub fn enable(&mut self, kind: DebugCategory) {
        self.enabled_kinds.insert(kind);
    }

    pub fn disable(&mut self, kind: DebugCategory) {
        self.enabled_kinds.remove(&kind);
    }

    pub fn draw(&mut self, frontend: &Frontend, camera: &Camera, frame: &mut Frame) -> Result<()> {
        for event in &mut self.events {
            if event.ticks_remaining > 0 {
                event.ticks_remaining -= 1;
            }

            if self.enabled_kinds.contains(&event.category) {
                let color = Self::get_color(event.color);
                let line_size = self.line_size;
                Self::mesh_event(&mut self.builder, color, line_size, event);
            }
        }
        self.events.drain_filter(|event| event.ticks_remaining == 0);

        self.drawer.upload(&self.builder)?;
        self.builder.clear();

        let uniforms = uniform! {
            screen_ratio: frontend.screen_ratio,
            player_pos: camera.pos.to_array(),
            zoom: camera.zoom,
        };

        let draw_parameters = DrawParameters {
            blend: Blend::alpha_blending(),
            ..DrawParameters::default()
        };
        self.drawer
            .draw(frame, &self.program, &uniforms, &draw_parameters)?;
        Ok(())
    }

    fn mesh_event(builder: &mut MeshBuilder<PosColorVertex>, color: [f32; 3], line_size: f32, event: &DebugEvent) {
        let opacity = if event.duration > 0 {
            event.ticks_remaining as f32 / event.duration as f32
        } else {
            1.0
        };
        let color = [color[0], color[1], color[2], opacity];
        let line_size = line_size * event.line_size;
        match event.draw {
            DebugDraw::Quad(quad) => {
                Self::mesh_rect(builder, color, quad, line_size);
            }
            DebugDraw::Line { start, stop } => {
                Self::mesh_line(builder, color, line_size, start, stop);
            }
            DebugDraw::Point(pos) => {
                Self::mesh_point(builder, color, line_size, pos);
            }
        }
    }

    fn mesh_rect(
        builder: &mut MeshBuilder<PosColorVertex>,
        color: [f32; 4],
        rect: Rect<f32, WS>,
        border_size: f32,
    ) {
        if border_size == 0.0
            || (border_size * 2.0 > rect.width() && border_size * 2.0 > rect.height())
        {
            // common case where the border is full
            builder.push_quad((rect, color));
        } else {
            // top
            {
                let mut rect = rect;
                let height = rect.height();
                rect.origin.y += height - border_size;
                rect.size.height = border_size;
                builder.push_quad((rect, color));
            }

            // bottom
            {
                let mut rect = rect;
                rect.size.height = border_size;
                builder.push_quad((rect, color));
            }

            // right
            {
                let mut rect = rect;
                let width = rect.width();
                rect.origin.x += width - border_size;
                rect.size.width = border_size;
                // overlap correction
                rect.size.height -= border_size * 2.0;
                rect.origin.y += border_size;
                builder.push_quad((rect, color));
            }

            // left
            {
                let mut rect = rect;
                rect.size.width = border_size;
                // overlap correction
                rect.size.height -= border_size * 2.0;
                rect.origin.y += border_size;
                builder.push_quad((rect, color));
            }
        }
    }

    fn mesh_point(
        builder: &mut MeshBuilder<PosColorVertex>,
        color: [f32; 4],
        size: f32,
        pos: Vector2D<f32, WS>,
    ) {
        let half = (size / 2.0);
        builder.push_quad((rect::<_, WS>(pos.x - half, pos.y - half, size, size), color))
    }

    fn mesh_line(
        builder: &mut MeshBuilder<PosColorVertex>,
        color: [f32; 4],
        line_size: f32,
        start: Vector2D<f32, WS>,
        stop: Vector2D<f32, WS>,
    ) {
        let furbertensvector = stop - start;
        let not_furbertensvector =
            vec2::<_, WS>(-furbertensvector.y, furbertensvector.x).normalize();

        let t_start = start + (not_furbertensvector * (line_size / 2.0));
        let b_start = start + (-not_furbertensvector * (line_size / 2.0));
        let t_stop = stop + (not_furbertensvector * (line_size / 2.0));
        let b_stop = stop + (-not_furbertensvector * (line_size / 2.0));
        builder.push_quad((
            [
                t_start.to_array(),
                b_start.to_array(),
                b_stop.to_array(),
                t_stop.to_array(),
            ],
            color,
        ));
    }

    fn get_color(color: u32) -> [f32; 3] {
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

    fn enabled(&self, kind: DebugCategory) -> bool {
        self.enabled_kinds.contains(&kind)
    }
}

impl DebugRendererImpl for DebugRenderer {
    fn event(&mut self, event: DebugEvent) {
        self.events.push(event);
    }
}
