use std::{collections::hash_set, iter};

use ahash::AHashSet;
use broccoli::{aabb::pin::AabbPin, axgeom::Rect, Tree, TreeData};
use palkia::prelude::*;

use crate::geom::{EntityAABB, Hitbox};

/// Keeps track of everything with both a [`Positioned`] and [`HasDims`]
#[derive(Debug, Default)]
pub struct HitboxTracker {
  es: AHashSet<Entity>,
}
impl Resource for HitboxTracker {}

impl HitboxTracker {
  pub fn on_create(&mut self, e: Entity) {
    self.es.insert(e);
  }

  pub fn on_remove(&mut self, e: Entity) {
    self.es.remove(&e);
  }

  pub fn iter(&self) -> iter::Copied<hash_set::Iter<'_, Entity>> {
    self.es.iter().copied()
  }

  pub fn contains(&self, e: Entity) -> bool {
    self.es.contains(&e)
  }
}

/// Janky-ass Broccoli tree holder
pub struct TreeHolder {
  bots: Vec<EntityAABB>,
  data: TreeData<f64>,
}
impl Resource for TreeHolder {}

impl TreeHolder {
  pub fn new(data: TreeData<f64>, bots: Vec<EntityAABB>) -> Self {
    Self { bots, data }
  }

  pub fn get_tree(&mut self) -> Tree<'_, EntityAABB> {
    let tree = Tree::from_tree_data(&mut self.bots, &self.data);
    #[cfg(debug_assertions)]
    broccoli::assert::assert_tree_invariants(&tree);
    tree
  }

  pub fn get_entities_in_box(
    &mut self,
    hitbox: Hitbox,
    filter: impl Fn(Entity) -> bool,
  ) -> Vec<Entity> {
    let mut tree = self.get_tree();

    let mut out = Vec::new();
    tree.find_all_intersect_rect(
      AabbPin::new(&mut (hitbox.0.inner_as(), ())),
      |_, hit| {
        let e = hit.e;
        if filter(e) {
          out.push(e);
        }
      },
    );

    out
  }
}
