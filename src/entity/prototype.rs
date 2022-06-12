use euclid::Rect;
use hecs::EntityBuilder;

use crate::api::id::Id;
use crate::api::identifier::Identifier;
use crate::api::prototype::Prototype;
use crate::entity::component::{CollisionComponent, HumanoidComponent, PhysicsComponent, PositionComponent, PrototypeComponent};
use crate::ty::WS;

pub struct EntityPrototype {
    pub position: PositionComponent,
    pub velocity: Option<PhysicsComponent>,
    pub collision: Option<CollisionComponent>,
    pub humanoid: Option<HumanoidComponent>,
    // Rendering
    pub image: Option<Identifier>,
    pub panel: Rect<f32, WS>,
}

impl Prototype for EntityPrototype {
    type Item = EntityBuilder;

    fn create(&self, id: Id<Self>) -> Self::Item {
        let mut builder = EntityBuilder::new();
        builder.add(self.position.clone());
        builder.add(PrototypeComponent { id });
        if let Some(comp) = self.velocity.as_ref() {
            builder.add(comp.clone());
        };
        if let Some(comp) = self.collision.as_ref() {
            builder.add(comp.clone());
        };
        if let Some(comp) = self.humanoid.as_ref() {
            builder.add(comp.clone());
        };
        builder
    }
}
