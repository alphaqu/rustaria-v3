//! Systems and stuff

pub mod collision;

use crate::entity::component::PositionComponent;
use crate::entity::component::PhysicsComponent;
use crate::entity::EntityStorage;

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
