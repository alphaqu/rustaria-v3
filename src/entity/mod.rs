use std::collections::HashMap;

use eyre::Result;
use hecs::{Component, DynamicBundle, Entity, EntityBuilder, EntityRef, Query, QueryBorrow, QueryMut, Ref, RefMut, TakenEntity};

use crate::{Chunk, ChunkPos, iter_components};
use crate::api::Carrier;
use crate::api::id::Id;
use crate::api::prototype::Prototype;
use crate::api::registry::MappedRegistry;
use crate::entity::prototype::EntityPrototype;
use crate::entity::system::{GravitySystem, VelocitySystem};
use crate::entity::system::collision::CollisionSystem;
use crate::entity::system::humanoid::HumanoidSystem;

pub mod component;
pub mod system;
pub mod prototype;

pub struct EntityStorage {
	world: hecs::World,
	templates: MappedRegistry<EntityPrototype, EntityBuilder>,
}

impl EntityStorage {
	pub fn new(carrier: &Carrier) -> EntityStorage {
		EntityStorage {
			world: Default::default(),
			templates: carrier.entity.map(|id, prototype| prototype.create(id))
		}
	}

	pub fn push(&mut self, id: Id<EntityPrototype>) -> Entity {
		self.world.spawn(self.templates.get_mut(id).build())
	}

	pub fn insert(&mut self, entity: Entity, id: Id<EntityPrototype>) {
		self.world.spawn_at(entity, self.templates.get_mut(id).build())
	}


	pub fn insert_raw(&mut self, entity: Entity, components: impl DynamicBundle) {
		self.world.spawn_at(entity, components)
	}

	pub fn remove(&mut self, entity: Entity) -> Option<TakenEntity<'_>> {
		self.world.take(entity).ok()
	}

	pub fn get(&self, entity: Entity) -> Option<EntityRef<'_>> {
		self.world.entity(entity).ok()
	}

	pub fn contains(&self, entity: Entity) -> bool {
		self.world.contains(entity)
	}


	pub fn get_comp<T: Component>(&self, entity: Entity) -> Option<Ref<'_, T>> {
		self.world.get(entity).ok()
	}

	pub fn get_mut_comp<T: Component>(&mut self, entity: Entity) -> Option<RefMut<'_, T>> {
		self.world.get_mut(entity).ok()
	}

	pub fn query<Q: Query>(&self) -> QueryBorrow<'_, Q> {
		self.world.query()
	}

	pub fn query_mut<Q: Query>(&mut self) -> QueryMut<'_, Q> {
		self.world.query_mut()
	}

	pub fn clone(&self, entity: Entity) -> Option<EntityBuilder> {
		let entity = self.world.entity(entity).ok()?;
		let mut builder = EntityBuilder::new();
		iter_components!({
			if let Some(component) = entity.get::<T>() {
				builder.add((*component).clone());
			}
		});

		Some(builder)
	}
}

pub struct EntityWorld {
	pub storage: EntityStorage,
	velocity: VelocitySystem,
	gravity: GravitySystem,
	collision: CollisionSystem,
	humanoid: HumanoidSystem,
}

impl EntityWorld {
	pub fn new(carrier: &Carrier) -> Result<EntityWorld> {
		Ok(EntityWorld  {
			storage: EntityStorage::new(carrier),
			velocity: VelocitySystem,
			gravity: GravitySystem,
			collision: CollisionSystem,
			humanoid: HumanoidSystem
		})
	}

	pub fn tick(&mut self, chunks: &HashMap<ChunkPos, Chunk>) {
		self.gravity.tick(&mut self.storage);
		self.humanoid.tick(&mut self.storage);
		self.collision.tick(&mut self.storage, chunks);
		self.velocity.tick(&mut self.storage);
	}
}