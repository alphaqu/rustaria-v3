use std::rc::Rc;

use eyre::{ContextCompat, Result};
use glium::{
    DrawParameters, Frame, IndexBuffer, Program, Surface, Vertex, VertexBuffer,
};
use glium::index::PrimitiveType;
use glium::uniforms::Uniforms;
use tracing::trace;

use crate::Frontend;
use crate::renderer::builder::MeshBuilder;

pub struct MeshDrawer<T: Copy + Vertex> {
    ctx: Rc<glium::backend::Context>,
    vertex: Option<VertexBuffer<T>>,
    index: Option<IndexBuffer<u32>>,
    vertex_length: usize,
    index_length: usize,
}

impl<T: Copy + Vertex> MeshDrawer<T> {
    pub fn new(frontend: &Frontend) -> Result<MeshDrawer<T>> {
        Ok(MeshDrawer {
            ctx: frontend.ctx.clone(),
            vertex: None,
            index: None,
            vertex_length: 0,
            index_length: 0,
        })
    }

    pub fn upload(&mut self, builder: &MeshBuilder<T>) -> Result<()> {
        // Vertex buffer
        match &mut self.vertex {
            None => {
                self.vertex = None;
                self.vertex = Some(VertexBuffer::dynamic(&self.ctx, &builder.vertex)?);
            }
            Some(buffer) => {
                if buffer.get_size() < builder.vertex.len() {
                    self.vertex = None;
                    self.vertex = Some(VertexBuffer::dynamic(&self.ctx, &builder.vertex)?);
                } else {
                    buffer
                        .slice_mut(0..builder.vertex.len())
                        .unwrap()
                        .write(&builder.vertex);
                }
            }
        }
        self.vertex_length = builder.vertex.len();

        // Index buffer
        match &mut self.index {
            None => {
                self.index = None;
                self.index = Some(IndexBuffer::dynamic(
                    &self.ctx,
                    PrimitiveType::TrianglesList,
                    &builder.index,
                )?);
            }
            Some(buffer) => {
                if buffer.get_size() < builder.index.len() {
                    self.index = None;
                    self.index = Some(IndexBuffer::dynamic(
                        &self.ctx,
                        PrimitiveType::TrianglesList,
                        &builder.index,
                    )?);
                } else {
                    buffer
                        .slice_mut(0..builder.index.len())
                        .unwrap()
                        .write(&builder.index);
                }
            }
        }
        self.index_length = builder.index.len();
        Ok(())
    }

    pub fn draw(
        &mut self,
        frame: &mut Frame,
        program: &Program,
        uniforms: &impl Uniforms,
        draw_parameters: &DrawParameters<'_>,
    ) -> Result<()> {
        frame.draw(
            self.vertex.as_ref().wrap_err("Vertex buffer inactive")?.slice(0..self.vertex_length).unwrap(),
            self.index.as_ref().wrap_err("Index buffer inactive")?.slice(0..self.index_length).unwrap(),
            program,
            uniforms,
            draw_parameters,
        )?;
        Ok(())
    }
}
