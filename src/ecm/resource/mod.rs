mod phys;

use std::{collections::hash_set, iter};

use ahash::AHashSet;
pub use phys::*;

use aglet::CoordVec;
use palkia::prelude::*;

pub struct ThePlayerEntity(pub Entity);
impl Resource for ThePlayerEntity {}

/// Where the world is viewed from
#[derive(Debug)]
pub struct Camera {
    pub center: CoordVec,
}
impl Resource for Camera {}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: CoordVec::new(0, 0),
        }
    }
}

#[derive(Debug, Default)]
pub struct BrainTracker {
    nerds: AHashSet<Entity>,
}
impl Resource for BrainTracker {}

impl BrainTracker {
    pub fn on_create(&mut self, e: Entity) {
        self.nerds.insert(e);
    }

    pub fn on_remove(&mut self, e: Entity) {
        self.nerds.remove(&e);
    }

    pub fn iter(&self) -> iter::Copied<hash_set::Iter<'_, Entity>> {
        self.nerds.iter().copied()
    }

    pub fn contains(&self, e: Entity) -> bool {
        self.nerds.contains(&e)
    }
}
