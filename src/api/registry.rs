use crate::api::prototype::Prototype;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::api::id::Id;
use crate::api::identifier::Identifier;

pub struct Registry<P: Prototype> {
    lookup: Vec<P>,
    identifier_lookup: HashMap<Identifier, Id<P>>,
}

impl<P: Prototype> Registry<P> {
    pub fn new(mut values: Vec<(Identifier, P)>) -> Registry<P> {
        values.sort_by(|(v0, _), (v1, _)| v0.cmp(v1));

        let mut lookup = Vec::with_capacity(values.len());
        let mut tag_to_id = HashMap::new();
        for (ident, prototype) in values {
            unsafe {
                tag_to_id.insert(
                    ident,
                    Id::new(lookup.len() as u32),
                );
            }
            lookup.push(prototype);
        }

        Registry {
            lookup,
            identifier_lookup: tag_to_id,
        }
    }


    pub fn entries(&self) -> &[P] {
        &self.lookup
    }

    /// Gets a prototype from this registry.
    pub fn get(&self, id: Id<P>) -> &P {
        &self.lookup[id.index()]
    }

    /// Creates an item using this prototype which is acquired from the id.
    pub fn create(&self, id: Id<P>) -> P::Item {
        self.get(id).create(id)
    }

    /// Converts an identifier to the id.
    pub fn identifier_to_id(&self, ident: &Identifier) -> Option<Id<P>> {
        self.identifier_lookup.get(ident).copied()
    }

    pub fn map<V>(&self, mut f: impl FnMut(Id<P>, &P) -> V) -> MappedRegistry<P, V>  {
        MappedRegistry {
            lookup: self.lookup.iter().enumerate().map(|(id, prototype)| unsafe {
                f(Id::new(id as u32), prototype)
            }).collect(),
            prototype: Default::default()
        }
    }
}

pub struct MappedRegistry<P: Prototype, V> {
    lookup: Vec<V>,
    prototype: PhantomData<P>
}

impl<P: Prototype, V> MappedRegistry<P, V> {
    pub fn get(&self, id: Id<P>) -> &V {
        &self.lookup[id.index()]
    }

    pub fn get_mut(&mut self, id: Id<P>) -> &mut V {
        &mut self.lookup[id.index()]
    }
}
