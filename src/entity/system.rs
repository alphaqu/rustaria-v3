//! Systems and stuff

use crate::entity::component::{GravityComponent, PhysicsComponent};
use crate::entity::component::PositionComponent;
use crate::entity::EntityStorage;
use crate::TPS;

pub mod collision;
pub mod humanoid;

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
		for (_, (velocity, gravity)) in
		world.query_mut::<(&mut PhysicsComponent, &GravityComponent)>()
		{
			velocity.vel.y -= ((0.3 * 4.0) / TPS as f32) * gravity.amount;
		}
	}
}