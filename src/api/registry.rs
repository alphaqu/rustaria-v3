use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::Enumerate;
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};

use crate::ty::id::Id;
use crate::ty::identifier::Identifier;
use crate::api::prototype::{FactoryPrototype, Prototype};

pub struct Registry<P: Prototype> {
    lookup: Vec<P>,
    identifier_lookup: HashMap<Identifier, Id<P>>,
}

impl<P: Prototype> Registry<P> {
    pub fn empty() -> Registry<P> {
        Registry {
            lookup: Vec::new(),
            identifier_lookup: HashMap::new()
        }
    }
    
    pub fn new(mut values: HashMap<Identifier, (f32, P)>) -> Registry<P> {
        let mut values: Vec<((Identifier, f32), P)> = values.into_iter().map(|(identifier, (priority, prototype))| ((identifier, priority), prototype)).collect();

        values.sort_by(|((id0, priority0), _), ((id1, priority1), _)| {
            let ordering = priority0.total_cmp(priority1);
            if ordering == Ordering::Equal {
                return id0.cmp(id1);
            }
            ordering
        });

        let mut lookup = Vec::with_capacity(values.len());
        let mut tag_to_id = HashMap::new();
        for ((ident, _), prototype) in values {
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

    pub fn len(&self) -> usize {
        self.lookup.len()
    }

    pub fn entries(&self) -> &[P] {
        &self.lookup
    }

    /// Gets a prototype from this registry.
    pub fn get(&self, id: Id<P>) -> &P {
        &self.lookup[id.index()]
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

impl<P: Prototype + FactoryPrototype> Registry<P> {
    /// Creates an item using this prototype which is acquired from the id.
    pub fn create(&self, id: Id<P>) -> P::Item {
        self.get(id).create(id)
    }
}

pub struct MappedRegistry<P: Prototype, V> {
    lookup: Vec<V>,
    prototype: PhantomData<P>
}

//pub type RegistryIter<V> = std::iter::Map<std::iter::Enumerate<std::slice::Iter<'_, V>>;

impl<P: Prototype, V> MappedRegistry<P, V> {
    pub fn get(&self, id: Id<P>) -> &V {
        &self.lookup[id.index()]
    }

    pub fn get_mut(&mut self, id: Id<P>) -> &mut V {
        &mut self.lookup[id.index()]
    }

    pub fn iter(&self) -> RegistryIter<'_, P, V>{
        RegistryIter {
            values: self.lookup.iter().enumerate(),
            _p: Default::default()
        }
    }


    pub fn iter_mut(&mut self) -> RegistryIterMut<'_, P, V>{
        RegistryIterMut {
            values: self.lookup.iter_mut().enumerate(),
            _p: Default::default()
        }
    }
}

pub struct RegistryIter<'a, P: Prototype, V> {
    values: Enumerate<Iter<'a, V>>,
    _p: PhantomData<P>
}
pub struct RegistryIterMut<'a, P: Prototype, V> {
    values: Enumerate<IterMut<'a, V>>,
    _p: PhantomData<P>
}


impl<'a, P: Prototype, V: 'a> Iterator for RegistryIter<'a, P, V> {
    type Item = (Id<P>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next().map(|(id, value)|  unsafe{
            (Id::new(id as u32), value)
        })
    }
}
impl<'a, P: Prototype, V: 'a> Iterator for RegistryIterMut<'a, P, V> {
    type Item = (Id<P>, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.values.next().map(|(id, value)|  unsafe{
            (Id::new(id as u32), value)
        })
    }
}



impl<P: Prototype, V: Clone> Clone for MappedRegistry<P, V> {
    fn clone(&self) -> Self {
        MappedRegistry {
            lookup: self.lookup.clone(),
            prototype: Default::default()
        }
    }
}
