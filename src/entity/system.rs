//! Systems and stuff

use crate::entity::component::PositionComponent;
use crate::entity::component::VelocityComponent;
use crate::entity::EntityStorage;

pub struct VelocitySystem;

impl VelocitySystem {
	pub fn tick(&mut self, world: &mut EntityStorage) {
		for (_, (position, velocity)) in
		world.query_mut::<(&mut PositionComponent, &VelocityComponent)>()
		{
			position.pos += velocity.velocity;
		}
	}
}
