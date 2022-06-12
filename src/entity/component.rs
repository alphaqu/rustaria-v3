use euclid::Vector2D;
use crate::ty::WS;

/// Our lovely components
#[macro_export]
macro_rules! iter_components {
    ($BLOCK:block) => {
	    { type T = $crate::entity::component::PositionComponent; $BLOCK; }
	    { type T = $crate::entity::component::VelocityComponent; $BLOCK; }
    };
}

#[derive(Clone)]
pub struct VelocityComponent {
	pub velocity: Vector2D<f32, WS>
}

#[derive(Clone)]
pub struct PositionComponent {
	pub pos: Vector2D<f32, WS>
}
