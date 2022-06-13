use euclid::{Rect, Vector2D};

use crate::api::id::Id;
use crate::entity::prototype::EntityPrototype;
use crate::ty::direction::DirMap;
use crate::ty::WS;

/// Our lovely components
#[macro_export]
macro_rules! iter_components {
    ($BLOCK:block) => {
	    { type T = $crate::entity::component::PositionComponent; $BLOCK; }
	    { type T = $crate::entity::component::PhysicsComponent; $BLOCK; }
	    { type T = $crate::entity::component::CollisionComponent; $BLOCK; }
	    { type T = $crate::entity::component::HumanoidComponent; $BLOCK; }
	    { type T = $crate::entity::component::PrototypeComponent; $BLOCK; }
	    { type T = $crate::entity::component::GravityComponent; $BLOCK; }
    };
}


#[derive(Clone)]
pub struct PrototypeComponent {
	pub id: Id<EntityPrototype>,
}

#[derive(Clone)]
pub struct GravityComponent {
	pub amount: f32,
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


#[derive(Clone, Debug)]
pub struct HumanoidComponent {
	// Settings
	pub jump_amount: f32,
	pub jump_speed: f32,

	pub run_acceleration: f32,
	pub run_slowdown: f32,
	pub run_max_speed: f32,

	// Runtime stuff
	pub dir: Vector2D<f32, WS>,
	pub jumping: bool,
	pub jumped: bool,
	pub jump_frames_remaining: f32,
}