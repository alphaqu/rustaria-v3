use euclid::{Rect, Vector2D};

use crate::ty::id::Id;
use crate::world::entity::prototype::EntityPrototype;
use crate::ty::direction::DirMap;
use crate::ty::WS;

/// Our lovely components
#[macro_export]
macro_rules! iter_components {
    ($BLOCK:block) => {
	    { type T = $crate::world::entity::component::PositionComponent; $BLOCK; }
	    { type T = $crate::world::entity::component::PhysicsComponent; $BLOCK; }
	    { type T = $crate::world::entity::component::CollisionComponent; $BLOCK; }
	    { type T = $crate::world::entity::component::HumanoidComponent; $BLOCK; }
	    { type T = $crate::world::entity::component::PrototypeComponent; $BLOCK; }
	    { type T = $crate::world::entity::component::GravityComponent; $BLOCK; }
    };
}


#[derive(Debug, Clone)]
pub struct PrototypeComponent {
	pub id: Id<EntityPrototype>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GravityComponent {
	pub amount: f32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PhysicsComponent {
	pub vel: Vector2D<f32, WS>,
	pub accel: Vector2D<f32, WS>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct PositionComponent {
	pub pos: Vector2D<f32, WS>
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(transparent)]
pub struct CollisionComponent {
	pub collision_box: Rect<f32, WS>,
	#[serde(skip)]
	pub collided: DirMap<bool>,
}


#[derive(Debug, Clone, serde::Deserialize)]
pub struct HumanoidComponent {
	// Settings
	pub jump_amount: f32,
	pub jump_speed: f32,

	pub run_acceleration: f32,
	pub run_slowdown: f32,
	pub run_max_speed: f32,

	// Runtime stuff
	#[serde(skip)]
	pub dir: Vector2D<f32, WS>,
	#[serde(skip)]
	pub jumping: bool,
	#[serde(skip)]
	pub jumped: bool,
	#[serde(skip)]
	pub jump_ticks_remaining: u32,
}