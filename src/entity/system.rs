//! Systems and stuff

pub mod collision;
pub mod humanoid;

use crate::entity::component::PositionComponent;
use crate::entity::component::PhysicsComponent;
use crate::entity::EntityStorage;
use crate::TPS;

pub struct VelocitySystem;

impl VelocitySystem {
	pub fn tick(&mut self, world: &mut EntityStorage) {
		for (_, (position, velocity)) in
		world.query_mut::<(&mut PositionComponent, &PhysicsComponent)>()
		{
			position.pos += velocity.vel;
		}
	}
}


pub struct GravitySystem;

impl GravitySystem {
	pub fn tick(&mut self, world: &mut EntityStorage) {
		for (_, (position, velocity)) in
		world.query_mut::<(&mut PositionComponent, &mut PhysicsComponent)>()
		{
			velocity.vel.y -= (0.3 * 4.0) / TPS as f32;
		}
	}
}