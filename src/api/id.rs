use crate::api::prototype::Prototype;
use std::marker::PhantomData;

/// The internal id is a instance bound identifier to the registry,
/// absolutely not forward/backwards compatible across versions or even game instances.
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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
