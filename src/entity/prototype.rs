use crate::api::id::Id;
use crate::api::prototype::Prototype;
use crate::entity::component::{PositionComponent, PhysicsComponent, CollisionComponent};
use hecs::EntityBuilder;

pub struct EntityPrototype {
    pub position: PositionComponent,
    pub velocity: Option<PhysicsComponent>,
    pub collision: Option<CollisionComponent>,
}

impl Prototype for EntityPrototype {
    type Item = EntityBuilder;

    fn create(&self, id: Id<Self>) -> Self::Item {
        let mut builder = EntityBuilder::new();
        builder.add(self.position.clone());
        if let Some(comp) = self.velocity.as_ref() {
            builder.add(comp.clone());
        };
        if let Some(comp) = self.collision.as_ref() {
            builder.add(comp.clone());
        };
        builder
    }
}
