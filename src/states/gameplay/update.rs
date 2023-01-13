use super::StateGameplay;

use aglet::{CoordVec, Direction8};
use ahash::AHashMap;
use broccoli::{aabb::pin::AabbPin, Tree};
use itertools::Itertools;
use macroquad::prelude::{self as mq, Vec2};
use palkia::prelude::*;

use crate::{
    ecm::{
        actions,
        component::{HasDims, Mover, Positioned},
        message::{MsgRecvHit, MsgSendHit},
        resource::{HitboxTracker, TreeHolder},
    },
    geom::{EntityAABB, Hitbox},
};

pub(super) fn do_collision(state: &mut StateGameplay) {
    let hitboxeds = {
        let tracker = state.world.read_resource::<HitboxTracker>().unwrap();
        tracker
            .iter()
            .map(|e| {
                let (pos, dims) =
                    state.world.query::<(&Positioned, &HasDims)>(e).unwrap();
                EntityAABB::new(e, pos.make_hitbox(*dims))
            })
            .collect_vec()
    };
    let mut hitboxeds_for_tree = hitboxeds.clone();
    let mut tree = Tree::new(hitboxeds_for_tree.as_mut_slice());

    let mut cache = BonkCache::default();

    for profile in hitboxeds.iter() {
        if let Some(mut mover) = state.world.query::<&mut Mover>(profile.e) {
            let mut pos = profile.hb().center();

            let bonk_x = do_axis_movement(
                &state.world,
                profile.e,
                mover.remainder,
                Hitbox::new(pos.x, pos.y, profile.hb().w(), profile.hb().h()),
                true,
                &mut tree,
                &mut cache,
            );
            mover.remainder = bonk_x.remainder;
            pos = bonk_x.new_center;

            let bonk_y = do_axis_movement(
                &state.world,
                profile.e,
                mover.remainder,
                Hitbox::new(pos.x, pos.y, profile.hb().w(), profile.hb().h()),
                false,
                &mut tree,
                &mut cache,
            );
            mover.remainder = bonk_y.remainder;
            pos = bonk_y.new_center;

            {
                let mut pos_comp =
                    state.world.query::<&mut Positioned>(profile.e).unwrap();
                pos_comp.pos = pos;
            }

            let bonkees = calculate_bonkees(bonk_x, bonk_y);
            for (bonked, norm) in bonkees {
                state
                    .world
                    .dispatch(profile.e, MsgSendHit::new(bonked, norm));
                state
                    .world
                    .dispatch(bonked, MsgRecvHit::new(profile.e, norm));
            }
        }
    }

    let data = tree.get_tree_data();
    drop(tree);
    state
        .world
        .insert_resource(TreeHolder::new(data, hitboxeds_for_tree));
}

fn calculate_bonkees(
    bonk_x: AxisMove,
    bonk_y: AxisMove,
) -> Vec<(Entity, Direction8)> {
    let bonkees = match (bonk_x.bonk, bonk_y.bonk) {
        (Some((e, pos)), None) => {
            vec![(
                e,
                if pos {
                    Direction8::East
                } else {
                    Direction8::West
                },
            )]
        }
        (None, Some((e, pos))) => {
            vec![(
                e,
                if pos {
                    Direction8::South
                } else {
                    Direction8::North
                },
            )]
        }
        (None, None) => Vec::new(),
        (Some((xe, xp)), Some((ye, yp))) => {
            if xe == ye {
                // Prevent double-collisions
                let dir = match (yp, xp) {
                    (true, true) => Direction8::SouthEast,
                    (true, false) => Direction8::SouthWest,
                    (false, true) => Direction8::NorthEast,
                    (false, false) => Direction8::NorthEast,
                };
                vec![(xe, dir)]
            } else {
                vec![
                    (
                        xe,
                        if xp {
                            Direction8::East
                        } else {
                            Direction8::West
                        },
                    ),
                    (
                        ye,
                        if yp {
                            Direction8::South
                        } else {
                            Direction8::North
                        },
                    ),
                ]
            }
        }
    };
    bonkees
}

/// The boolean represents "is the normal in the positive direction of this axis?"
struct AxisMove {
    remainder: Vec2,
    new_center: CoordVec,
    bonk: Option<(Entity, bool)>,
}

#[derive(Debug, Default)]
struct BonkCache(AHashMap<(Entity, Entity), Vec2>);

impl BonkCache {
    fn sort(a: Entity, b: Entity) -> (Entity, Entity) {
        if a < b {
            (a, b)
        } else {
            (b, a)
        }
    }

    fn insert(&mut self, a: Entity, b: Entity, norm: Vec2) {
        self.0.insert(BonkCache::sort(a, b), norm);
    }

    fn get(&self, a: Entity, b: Entity) -> Option<Vec2> {
        let sorted = BonkCache::sort(a, b);
        let norm = *self.0.get(&sorted)?;
        Some(if sorted.0 == a {
            // then it was not flipped; we put this in as a bonking b
            norm
        } else {
            -norm
        })
    }
}

/// https://maddythorson.medium.com/celeste-and-towerfall-physics-d24bd2ae0fc5
/// Returns colliders bonked into and the normal of the bonked face.
fn do_axis_movement<'t>(
    world: &World,
    me: Entity,
    mut remainder: Vec2,
    hb: Hitbox,
    horiz: bool,
    tree: &mut Tree<'t, EntityAABB>,
    cache: &mut BonkCache,
) -> AxisMove {
    let delta = (if horiz { remainder.x } else { remainder.y }).round() as i32;
    let mut pos = hb.center();

    if delta != 0 {
        let slot = if horiz {
            &mut remainder.x
        } else {
            &mut remainder.y
        };
        *slot -= delta as f32;
        let sign = delta.signum();

        let mut delta = delta;
        while delta != 0 {
            let (dx, dy) = if horiz { (sign, 0) } else { (0, sign) };
            let proposed_pos = pos + CoordVec::new(dx, dy);
            let proposed_aabb =
                Hitbox::new(proposed_pos.x, proposed_pos.y, hb.w(), hb.h());

            let mut collision_found = None;
            tree.find_all_intersect_rect(
                AabbPin::new(&mut (proposed_aabb.0.inner_as(), ())),
                |_, hit| {
                    // TODO: is the ability to not shortcut out sooner a problem?
                    // may be worth pr-ing that
                    if collision_found.is_some() {
                        return;
                    }

                    if let Some(bonk) = cache.get(me, hit.e) {
                        collision_found = Some((hit.e, bonk));
                    } else if actions::collides_with(world, me, hit.e) {
                        collision_found = Some((
                            hit.e,
                            if horiz {
                                mq::vec2(sign as f32, 0.0)
                            } else {
                                mq::vec2(0.0, sign as f32)
                            },
                        ));
                    }
                },
            );

            if let Some((other, norm)) = collision_found {
                cache.insert(me, other, norm);
                return AxisMove {
                    new_center: pos,
                    remainder,
                    bonk: Some((other, sign > 0)),
                };
            } else {
                pos = proposed_pos;
                delta -= sign;
            }
        }
    }

    AxisMove {
        new_center: pos,
        remainder,
        bonk: None,
    }
}
