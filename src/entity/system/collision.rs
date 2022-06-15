use std::ops::Index;

use euclid::{rect, vec2, Rect, Size2D, Vector2D};

use crate::debug::{DebugKind, DebugRendererImpl};
use crate::entity::component::{CollisionComponent, PhysicsComponent, PositionComponent};
use crate::entity::EntityStorage;
use crate::ty::direction::DirMap;
use crate::ty::block_pos::BlockPos;
use crate::ty::WS;
use crate::util::aabb;
use crate::{Api, ChunkStorage};

pub struct CollisionSystem;

impl CollisionSystem {
    pub fn tick(
        &mut self,
        api: &Api,
        storage: &mut EntityStorage,
        chunks: &ChunkStorage,
        debug: &mut impl DebugRendererImpl,
    ) {
        for (_, (collision, position, physics)) in storage.query_mut::<(
            &mut CollisionComponent,
            &PositionComponent,
            &mut PhysicsComponent,
        )>() {
            collision.collided = DirMap::new([false; 4]);

            // hitbox is the hitbox so we need to offset it to WorldSpace.
            let mut old_rect = collision.collision_box;
            old_rect.origin += position.pos;

            let mut new_rect = old_rect;
            new_rect.origin += physics.vel;

            let x1 = new_rect.min_x().min(old_rect.min_x()).floor() as i64;
            let y1 = new_rect.min_y().min(old_rect.min_y()).floor() as i64;
            let x2 = new_rect.max_x().max(old_rect.max_x()).ceil() as i64;
            let y2 = new_rect.max_y().max(old_rect.max_y()).ceil() as i64;
            debug.draw_hrect(
                DebugKind::EntityCollision,
                0x727072,
                rect(
                    x1 as f32,
                    y1 as f32,
                    x2 as f32 - x1 as f32,
                    y2 as f32 - y1 as f32,
                ),
            );
            debug.draw_hrect(DebugKind::EntityCollision, 0xfcfcfa, old_rect);

            let mut collisions = Vec::new();
            for x in x1..x2 {
                for y in y1..y2 {
                    test_collision(
                        api,
                        vec2(x as f32, y as f32),
                        physics.vel,
                        old_rect,
                        chunks,
                        &mut collisions,
                        debug,
                    )
                }
            }

            collisions.sort_by(|v0, v1| v0.1.total_cmp(&v1.1));

            for (pos, _) in collisions {
                if let Some(Some((d, contact))) =
                    aabb::resolve_dynamic_rect_vs_rect(physics.vel, old_rect, 1.0, pos)
                {
                    debug.draw_hrect(DebugKind::EntityCollision, 0xc1c0c0, pos);
                    physics.vel += d;
                    physics.accel += contact
                        .to_vec2()
                        .component_mul(vec2(physics.accel.x.abs(), physics.accel.y.abs()));
                    collision.collided[contact] = true;
                }
            }
        }
    }
}

fn test_collision(
    api: &Api,
    pos: Vector2D<f32, WS>,
    vel: Vector2D<f32, WS>,
    collision_area: Rect<f32, WS>,
    chunks: &ChunkStorage,
    collisions: &mut Vec<(Rect<f32, WS>, f32)>,
    debug: &mut impl DebugRendererImpl,
) {
    if let Ok(world_pos) = BlockPos::try_from(pos) {
        if let Some(chunk) = chunks.get(world_pos.chunk) {
            for (id, layer) in chunk.layers.iter() {
                let prototype = api.carrier.block_layers.get(id);
                if !prototype.collision || !layer[world_pos.entry].collision {
                    // dont move.
                    continue;
                }
                let tile = Rect::new(pos.to_point(), Size2D::new(1.0, 1.0));
                if let Some((pos, contact_time)) =
                    aabb::dynamic_rect_vs_rect(vel, collision_area, 1.0, tile)
                        .map(|collision| (tile, collision.contact_time))
                {
                    debug.draw_hrect(DebugKind::EntityCollision, 0x939293, pos);
                    collisions.push((pos, contact_time));
                }
            }
        }
    }
}
