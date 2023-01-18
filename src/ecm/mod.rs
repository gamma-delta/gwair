use palkia::prelude::*;

pub mod actions;
pub mod component;
pub mod message;
pub mod resource;

use component::*;

use crate::EntityFab;

use self::resource::{Camera, HitboxTracker};

/// Register components and insert resources
pub fn setup_world(world: &mut World) {
  world.register_component::<Positioned>();
  world.register_component::<HasDims>();
  world.register_component::<Mover>();
  world.register_component::<Velocitized>();
  world.register_component::<KinematicState>();

  world.register_component::<FrictionHaver>();
  world.register_component::<Bonker>();
  world.register_component::<Collider>();

  world.register_component::<AgeTracker>();
  world.register_component::<LimitedTimeOffer>();

  world.register_component::<ZLevel>();
  world.register_component::<ColoredHitbox>();
  world.register_component::<DrawTexture>();

  world.register_component::<PlayerController>();

  world.register_component::<SwingableOn>();
  world.register_component::<PickuppableRod>();

  //

  world.insert_resource(Camera::new());
  world.insert_resource_default::<HitboxTracker>();
}

pub fn setup_fabber(fab: &mut EntityFab) {
  // dims, mover, vel
  fab.register("physic-body", PhysicFactory);

  fab.register_serde::<Bonker>("bonker"); // but here's the bonker
  fab.register("friction", FrictionFactory);
  fab.register_serde::<Collider>("collider");

  fab.register_serde::<AgeTracker>("age-tracker");
  fab.register_serde::<LimitedTimeOffer>("despawn-timer");

  fab.register_serde::<ZLevel>("zlevel");
  fab.register_serde::<ColoredHitbox>("colored-hitbox");
  fab.register_serde::<DrawTexture>("texture");

  fab.register_serde::<SwingableOn>("swingable-on");
  fab.register_serde::<PickuppableRod>("pickuppable-rod");

  fab.register("player", PlayerFactory);
}
