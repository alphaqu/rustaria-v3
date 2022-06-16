use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::api::prototype::Prototype;

/// The internal id is a instance bound identifier to the registry,
/// absolutely not forward/backwards compatible across versions or even game instances.
#[derive( Ord, PartialOrd, Debug)]
pub struct Id<P: Prototype> {
    id: u32,
    prototype: PhantomData<P>,
}
impl<P: Prototype> Id<P> {
    // DO NOT EVEN THINK ABOUT MAKING THIS. THIS SHOULD ONLY BE MADE FROM THE REGISTRY ITS FROM
    pub unsafe fn new(id: u32) -> Id<P> {
        Id {
            id,
            prototype: Default::default(),
        }
    }

    pub fn index(&self) -> usize {
        self.id as usize
    }
}

impl<P: Prototype> Hash for Id<P> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
impl<P: Prototype> PartialEq<Self> for Id<P> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<P: Prototype> Eq for Id<P> {}

// This is needed as rustc cringes on the phantomdata
impl<P: Prototype> Clone for Id<P> {
    fn clone(&self) -> Id<P> {
        Id {
            id: self.id,
            prototype: Default::default(),
        }
    }
}

impl<P: Prototype> Copy for Id<P> {}
