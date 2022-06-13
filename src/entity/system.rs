//! Systems and stuff

use crate::debug::{DebugKind, DebugRendererImpl};
use crate::entity::component::{GravityComponent, PhysicsComponent};
use crate::entity::component::PositionComponent;
use crate::entity::EntityStorage;
use crate::TPS;

pub mod collision;
pub mod humanoid;

pub struct VelocitySystem;

impl VelocitySystem {
	pub fn tick(&mut self, world: &mut EntityStorage, debug: &mut impl DebugRendererImpl) {
		for (_, (position, velocity)) in
		world.query_mut::<(&mut PositionComponent, &mut PhysicsComponent)>()
		{
			debug.draw_line(DebugKind::EntityVelocity, 0xff6188, position.pos, position.pos + (velocity.vel * (TPS as f32 / 30.0)));
			position.pos += velocity.vel;
			velocity.vel += velocity.accel;
		}
	}
}


pub struct GravitySystem;

impl GravitySystem {
	pub fn tick(&mut self, world: &mut EntityStorage) {
		for (_, (velocity, gravity)) in
		world.query_mut::<(&mut PhysicsComponent, &GravityComponent)>()
		{
			velocity.vel.y -= (0.4 / TPS as f32) * gravity.amount;
		}
	}
}