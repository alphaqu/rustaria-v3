use euclid::{Rect, Vector2D};
use crate::ty::direction::DirMap;
use crate::ty::WS;

/// Our lovely components
#[macro_export]
macro_rules! iter_components {
    ($BLOCK:block) => {
	    { type T = $crate::entity::component::PositionComponent; $BLOCK; }
	    { type T = $crate::entity::component::PhysicsComponent; $BLOCK; }
    };
}

#[derive(Clone)]
pub struct PhysicsComponent {
	pub vel: Vector2D<f32, WS>,
	pub accel: Vector2D<f32, WS>,
}

#[derive(Clone)]
pub struct PositionComponent {
	pub pos: Vector2D<f32, WS>
}


#[derive(Clone)]
pub struct CollisionComponent {
	pub collision_box: Rect<f32, WS>,
	pub collided: DirMap<bool>,
}
