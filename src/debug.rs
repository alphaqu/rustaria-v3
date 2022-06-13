use euclid::{Rect, Vector2D};
use crate::ty::WS;

pub trait DebugRendererImpl {
	/// Draws a world based filled rectangle
	///
	/// # Arguments
	///
	/// * `kind`: The debug kind to render. If the kind is disabled this call will be ignored-
	/// * `color`: The hexadecimal color of this rectangle
	/// * `rect`: The WorldSpace rectangle.
	///
	/// # Examples
	///
	/// ```
	/// debug.draw_rect(DebugKind::Tile, 0xff0000, rect(-1.0, -1.0, 2.0, 2.0));
	/// ```
	fn draw_rect(&mut self, kind: DebugKind, color: u32, rect: Rect<f32, WS>);
	fn draw_hrect(&mut self, kind: DebugKind, color: u32, rect: Rect<f32, WS>);
	fn draw_line(&mut self, kind: DebugKind, color: u32, start: Vector2D<f32, WS>, stop: Vector2D<f32, WS>);
	fn draw_point(&mut self, kind: DebugKind, color: u32, pos: Vector2D<f32, WS>);
}

pub struct DummyRenderer;
impl DebugRendererImpl for DummyRenderer {
	fn draw_rect(&mut self, kind: DebugKind, color: u32, rect: Rect<f32, WS>) {

	}

	fn draw_hrect(&mut self, kind: DebugKind, color: u32, rect: Rect<f32, WS>) {
	}

	fn draw_line(&mut self, kind: DebugKind, color: u32, start: Vector2D<f32, WS>, stop: Vector2D<f32, WS>) {

	}

	fn draw_point(&mut self, kind: DebugKind, color: u32, pos: Vector2D<f32, WS>) {
	}
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum DebugKind {
	Tile,
	EntityVelocity,
	EntityCollision,
}
