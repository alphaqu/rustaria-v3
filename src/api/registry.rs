use std::cmp::Ordering;
use std::iter::{Map, Zip};
use crate::ty::identifier::Identifier;
use crate::util::blake3;

pub struct Registry<I> {
	pub table: IdTable<I, I>,
	pub id_to_ident: IdTable<I, Identifier>,
	pub ident_to_id: FxHashMap<Identifier, Id<I>>,
}

pub type RegistryIntoIter<I> = Map<Zip<IdTableIter<I, I, IntoIter<I>>, IdTableIter<I, Identifier, IntoIter<Identifier>>>, fn(((Id<I>, I), (Id<I>, Identifier))) -> (Id<I>, Identifier, I)>;
pub type RegistryIter<'a, I> = Map<Zip<IdTableIter<I, &'a I, Iter<'a, I>>, IdTableIter<I, &'a Identifier, Iter<'a, Identifier>>>, fn(((Id<I>, &'a I), (Id<I>, &'a Identifier))) -> (Id<I>, &'a Identifier, &'a I)>;

impl<I> Registry<I> {
	pub fn get(&self, id: Id<I>) -> &I {
		self.table.get(id)
	}

	pub fn get_mut(&mut self, id: Id<I>) -> &mut I {
		self.table.get_mut(id)
	}

	pub fn get_identifier(&self, id: Id<I>) -> &Identifier {
		self.id_to_ident.get(id)
	}

	pub fn get_id(&self, id: &Identifier) -> Option<Id<I>> {
		self.ident_to_id.get(id).copied()
	}

	pub fn into_iter(self) -> RegistryIntoIter<I> {
		self.table.into_iter().zip(self.id_to_ident.into_iter()).map(|((id, prototype), (_, identifier))| {
			(id, identifier, prototype)
		})
	}

	pub fn iter(&self) -> RegistryIter<I> {
		self.table.iter().zip(self.id_to_ident.iter()).map(|((id, prototype), (_, identifier))| {
			(id, identifier, prototype)
		})
	}

	pub fn new(values: FxHashMap<Identifier, (f32, I)>, hasher: &mut blake3::Hasher) -> Registry<I> {
		let mut values: Vec<((Identifier, f32), I)> = values
			.into_iter()
			.map(|(identifier, (priority, prototype))| ((identifier, priority), prototype))
			.collect();

		values.sort_by(|((id0, priority0), _), ((id1, priority1), _)| {
			let ordering = priority0.total_cmp(priority1);
			if ordering == Ordering::Equal {
				return id0.cmp(id1);
			}
			ordering
		});

		values.into_iter().enumerate().map(|(id, ((identifier, priority), value))| unsafe {
			hasher.update(&(id as u32).to_be_bytes());
			hasher.update(&priority.to_be_bytes());
			hasher.update(identifier.path.as_bytes());
			hasher.update(identifier.namespace.as_bytes());

			(Id::<I>::new(id), identifier, value)
		}).collect()
	}
}

impl<I: FactoryPrototype> Registry<I> {
	pub fn create(&self, id: Id<I>) -> I::Item {
		self.table.get(id).create(id)
	}
}

impl<I> FromIterator<(Id<I>, Identifier, I)> for Registry<I> {
	fn from_iter<T: IntoIterator<Item = (Id<I>, Identifier, I)>>(iter: T) -> Self {
		let mut lookup = Vec::new();
		let mut ident_to_id = FxHashMap::with_hasher(FxBuildHasher::default());
		let mut id_to_ident = Vec::new();

		for (id, ident, value) in iter {
			ident_to_id.insert(ident.clone(), id);
			lookup.push((id, value));
			id_to_ident.push((id, ident));
		}

		Registry {
			table: lookup.into_iter().collect(),
			id_to_ident: id_to_ident.into_iter().collect(),
			ident_to_id
		}
	}
}

impl<I> Default for Registry<I> {
	fn default() -> Self {
		Registry {
			table: IdTable::default(),
			id_to_ident: IdTable::default(),
			ident_to_id: Default::default()
		}
	}
}


use std::marker::PhantomData;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;
use fxhash::{FxBuildHasher, FxHashMap};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::api::prototype::FactoryPrototype;
use crate::ty::id::Id;

pub struct IdTable<I, V> {
	values: Vec<V>,
	_i: PhantomData<I>,
}

impl<I, V> IdTable<I, V> {

	pub fn get(&self, id: Id<I>) -> &V {
		&self.values[id.index()]
	}

	pub fn get_mut(&mut self, id: Id<I>) -> &mut V {
		&mut self.values[id.index()]
	}

	pub fn iter(&self) -> IdTableIter<I, &V, Iter<V>> {
		IdTableIter::new(self.values.iter())
	}

	pub fn iter_mut(&mut self) -> IdTableIter<I, &mut V, IterMut<V>> {
		IdTableIter::new(self.values.iter_mut())
	}
}

impl<I, V> Default for IdTable<I, V> {
	fn default() -> Self {
		IdTable {
			values: Vec::new(),
			_i: Default::default()
		}
	}
}

impl<I, V> FromIterator<(Id<I>, V)> for IdTable<I, V> {
	// this breaks the 100% safety but makes things 100x easier to use sooo ehm. yes
	fn from_iter<T: IntoIterator<Item = (Id<I>, V)>>(iter: T) -> Self {
		let mut items = Vec::new();
		for (id, value) in iter {
			items.push((id, value));
		}

		items.sort_by(|(id0, _), (id1, _)| {
			id0.index().cmp(&id1.index())
		});

		// safety check
		let mut last_id: Option<usize> = None;
		for (id, _) in &items {
			if let Some(last_id) = last_id {
				if id.index() == last_id {
					panic!("Duplicate id");
				} else if id.index() != last_id + 1 {
					panic!("Skipped id");
				}
			} else if id.index() != 0 {
				panic!("id does not start on 0");
			}

			last_id = Some(id.index());
		}

		IdTable {
			values: items.into_iter().map(|(_, value)| value).collect(),
			_i: Default::default()
		}
	}
}

impl<I, V: Clone> Clone for IdTable<I, V> {
	fn clone(&self) -> Self {
		IdTable {
			values: self.values.clone(),
			_i: Default::default()
		}
	}
}

impl<I, V: Serialize> Serialize for IdTable<I, V> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		self.values.serialize(serializer)
	}
}

impl<'de, I, V: Deserialize<'de>> Deserialize<'de> for IdTable<I, V> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de>  {
		Vec::<V>::deserialize(deserializer).map(|lookup| {
			IdTable {
				values: lookup,
				_i: Default::default()
			}
		})
	}
}

impl<I, V> IntoIterator for IdTable<I, V> {
	type Item = (Id<I>, V);
	type IntoIter = IdTableIter<I, V, IntoIter<V>>;

	fn into_iter(self) -> Self::IntoIter {
		IdTableIter::new(self.values.into_iter())
	}
}


pub struct IdTableIter<I, V, Iter: Iterator<Item = V>> {
	iter: Iter,
	id: usize,
	_p: PhantomData<I>
}

impl<I, V, Iter: Iterator<Item = V>>  IdTableIter<I, V, Iter> {
	fn new(iter: Iter) ->  IdTableIter<I, V, Iter> {
		IdTableIter {
			iter,
			id: 0,
			_p: Default::default()
		}
	}
}


impl<I, V, Iter: Iterator<Item = V>>  Iterator for IdTableIter<I, V, Iter> {
	type Item = (Id<I>, V);

	fn next(&mut self) -> Option<Self::Item> {
		let out = self.iter.next().map(|v| (unsafe {
			Id::new(self.id)
		}, v));
		self.id += 1;
		out
	}
}