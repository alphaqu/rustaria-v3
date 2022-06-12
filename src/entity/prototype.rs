use crate::api::id::Id;
use crate::api::prototype::Prototype;
use crate::entity::component::{PositionComponent, VelocityComponent};
use hecs::EntityBuilder;

pub struct EntityPrototype {
    pub position: PositionComponent,
    pub velocity: Option<VelocityComponent>,
}

impl Prototype for EntityPrototype {
    type Item = EntityBuilder;

    fn create(&self, id: Id<Self>) -> Self::Item {
        let mut builder = EntityBuilder::new();
        builder.add(self.position.clone());
        if let Some(velocity) = self.velocity.as_ref() {
            builder.add(velocity.clone());
        };
        builder
    }
}
