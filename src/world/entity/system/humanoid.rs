use crate::world::entity::component::{CollisionComponent, HumanoidComponent, PhysicsComponent};
use crate::world::entity::EntityStorage;
use crate::ty::direction::Direction;
use crate::TPS;

pub struct HumanoidSystem;

impl HumanoidSystem {
    pub fn tick(&mut self, storage: &mut EntityStorage) {
        for (_, (physics, humanoid, collision)) in storage.query_mut::<(
            &mut PhysicsComponent,
            &mut HumanoidComponent,
            &CollisionComponent,
        )>() {
            physics.vel.x += humanoid.dir.x * (humanoid.run_acceleration / TPS as f32);
            physics.vel.y += humanoid.dir.y * (humanoid.run_acceleration / TPS as f32);

            //if physics.vel.x > humanoid.run_max_speed / TPS as f32 {
            //	physics.vel.x = humanoid.run_max_speed / TPS as f32;
            //} else if physics.vel.x < -(humanoid.run_max_speed / TPS as f32) {
            //	physics.vel.x = -humanoid.run_max_speed / TPS as f32;
            //}

	        if humanoid.dir.length() != 0.0 {
		        let max_speed = humanoid.run_max_speed / TPS as f32;
		        if physics.vel.x < -max_speed {
			        physics.vel.x = -max_speed;
		        } else if physics.vel.x > max_speed {
			        physics.vel.x = max_speed;
		        }
	        }

            if collision.collided[Direction::Up] {
	            // TODO cleanup terraria movement code
                if humanoid.dir.x < 0.0 {
                    if physics.vel.x > humanoid.run_slowdown / TPS as f32 {
                        physics.vel.x -= humanoid.run_slowdown / TPS as f32;
                    }
                    physics.vel.x -= humanoid.run_acceleration / TPS as f32;
                } else if humanoid.dir.x > 0.0
                    && physics.vel.x < humanoid.run_max_speed / TPS as f32
                {
                    if physics.vel.x < -(humanoid.run_slowdown / TPS as f32) {
                        physics.vel.x += humanoid.run_slowdown / TPS as f32;
                    }
                    physics.vel.x += humanoid.run_acceleration / TPS as f32;
                } else if physics.vel.x > humanoid.run_slowdown / TPS as f32 {
                    physics.vel.x -= humanoid.run_slowdown / TPS as f32;
                } else if physics.vel.x < -(humanoid.run_slowdown / TPS as f32) {
                    physics.vel.x += humanoid.run_slowdown / TPS as f32;
                } else {
                    physics.vel.x = 0.0;
                }

	            if humanoid.jump_frames_remaining > 0.0 {
		            humanoid.jump_frames_remaining = 0.0;
	            }  else if humanoid.jumping {
		            humanoid.jump_frames_remaining = humanoid.jump_amount / TPS as f32;
		            humanoid.jumping = false;
	            }
            }

            if humanoid.jump_frames_remaining > 0.0 {
	            physics.vel.y = physics.vel.y.max(humanoid.jump_speed / TPS as f32);
	            humanoid.jump_frames_remaining -= 1.0;
            }
        }
    }
}
