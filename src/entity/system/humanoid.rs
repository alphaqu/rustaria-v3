use crate::TPS;
use crate::entity::component::{CollisionComponent, HumanoidComponent, PhysicsComponent};
use crate::entity::EntityStorage;
use crate::ty::direction::Direction;

pub struct HumanoidSystem;

impl HumanoidSystem {
	pub fn tick(&mut self, storage: &mut EntityStorage) {
		for (_, (physics, humanoid, collision)) in storage.query_mut::<(&mut PhysicsComponent, &mut HumanoidComponent, &CollisionComponent)>() {
			physics.vel.x += humanoid.dir.x * (humanoid.run_acceleration / TPS as f32);
			physics.vel.y += humanoid.dir.y * (humanoid.run_acceleration / TPS as f32);

			if physics.vel.x > humanoid.run_max_speed / TPS as f32 {
				physics.vel.x = humanoid.run_max_speed / TPS as f32;
			} else if physics.vel.x < -(humanoid.run_max_speed / TPS as f32) {
				physics.vel.x = -humanoid.run_max_speed / TPS as f32;
			}

			if collision.collided[Direction::Up] {
				if physics.vel.x > humanoid.run_slowdown / TPS as f32 {
					physics.vel.x -= humanoid.run_slowdown / TPS as f32;
				} else if physics.vel.x < -(humanoid.run_slowdown / TPS as f32) {
					physics.vel.x += humanoid.run_slowdown / TPS as f32;
				} else {
					physics.vel.x = 0.0;
				}

				if humanoid.jumped && !humanoid.jumping {
					humanoid.jumped = false;
				}

				if humanoid.jumping && !humanoid.jumped {
					humanoid.jump_frames_remaining = humanoid.jump_frames;
					humanoid.jumped = true;
				}
			}

			if humanoid.jump_frames_remaining > 0.0 {
				if humanoid.jumping {
					physics.vel.y = humanoid.jump_speed / TPS as f32;
					humanoid.jump_frames_remaining -= 1.0 / TPS as f32;
				} else {
					humanoid.jump_frames_remaining = 0.0;
				}
			}
		}
	}
}