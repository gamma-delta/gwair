use super::{grab_extant_rod_hbs, NormalState, PlayerController, PlayerState};

use std::f32::consts::{PI, TAU};

use glam::{vec2, Vec2};
use palkia::prelude::*;

use crate::{
  controls::ControlState,
  ecm::{
    component::{
      player::{Normal, Swinging},
      KinematicState, Positioned, SwingableOn, Velocitized,
    },
    resource::{FabCtxHolder, TreeHolder},
  },
  resources::Resources,
};

impl PlayerController {
  pub fn check_start_swinging(
    &mut self,
    controls: ControlState,
    access: &ListenerWorldAccess,
    me: Entity,
  ) {
    let stats = &self.stats;

    if let PlayerState::Normal(n) = &mut self.state {
      if controls.swing {
        let state_ok_to_swing = match n.state {
          NormalState::OnGround => false,
          NormalState::FallingFromLedge { .. }
          | NormalState::JumpingUp
          | NormalState::Falling => true,
        };
        if !n.was_swinging && state_ok_to_swing {
          let anchor_delta = if controls.movement.length_squared() < 0.0001 {
            Vec2::new(0.0, -1.0)
          } else {
            controls.movement.normalize()
          };
          let player_pos = access.query::<&Positioned>(me).unwrap();
          let anchor_pos =
            vec2(player_pos.pos.x as f32, player_pos.pos.y as f32)
              + anchor_delta * stats.rod_anchor_dist;

          // First try to swing on a rod in the world, prioritize that
          let extant_swingable = {
            let mut trees = access.write_resource::<TreeHolder>().unwrap();
            'found: {
              for check_hb in
                grab_extant_rod_hbs(player_pos.pos, anchor_delta, stats)
              {
                let out = trees.get_entities_in_box(check_hb, |e| {
                  access.query::<&SwingableOn>(e).is_some()
                });
                if let Some(it) = out.get(0) {
                  break 'found Some(*it);
                }
              }

              None
            }
          };
          let swingpoint = if let Some(it) = extant_swingable {
            let pos = access.query::<&Positioned>(it).unwrap().pos;
            Some((it, vec2(pos.x as f32, pos.y as f32)))
          } else if !self.deployed_rod_in_air
            && self.deployed_rod_entity.is_none()
          {
            self.deployed_rod_in_air = true;
            let res = Resources::get();
            let ctx = access.read_resource::<FabCtxHolder>().unwrap();
            let e = res
              .fabber()
              .instantiate(
                "immovable-rod",
                access.lazy_spawn().with(Positioned::from_vec(anchor_pos)),
                &ctx.0,
              )
              .unwrap();
            self.deployed_rod_entity = Some(e);
            Some((e, anchor_pos))
          } else {
            None
          };

          if let Some((swingee, anchor_pos)) = swingpoint {
            let player_vel = access.query::<&Velocitized>(me).unwrap();

            let anchor_delta = anchor_pos
              - vec2(player_pos.pos.x as f32, player_pos.pos.y as f32);
            let anchor_dir = anchor_delta.normalize();

            dbg!(player_vel.vel, anchor_delta);
            // how much in common does the player vel have with
            // orthagonal to the anchor delta?
            // vector rejection, but with a sign also
            let rej =
              player_vel.vel.reject_from_normalized(anchor_dir).length();
            let perp_dot = player_vel.vel.perp_dot(anchor_dir);
            let vel = rej * perp_dot.signum() * stats.vel_to_swing_vel_rate;

            let vel = if (stats.start_grab_speed_cheat_min
              ..=stats.start_grab_speed_cheat_max)
              .contains(&vel.abs())
            {
              stats.start_grab_speed_cheat_max * vel.signum()
            } else {
              vel
            };

            // We consider an angle of 0 to be straight down, so we
            // need the angle between down.
            let angle = vec2(0.0, -1.0).angle_between(anchor_dir);
            println!("initial: {} {}", vel, angle);

            self.state = PlayerState::Swinging(Swinging {
              angle,
              vel,
              anchor_pos,
              swingee,
            });
          }
        }
      } else {
        n.was_swinging = false;
      }
    }
  }

  pub fn swinging_movement(
    &mut self,
    access: &ListenerWorldAccess,
    entity: Entity,
    controls: ControlState,
    dt: f32,
  ) {
    let stats = &self.stats;

    let swinging = match self.state {
      PlayerState::Swinging(ref mut it) => it,
      _ => unreachable!(),
    };

    let ks = access.query::<&KinematicState>(entity).unwrap();

    swinging.angle = (swinging.angle + PI).rem_euclid(TAU) - PI;

    let gravity = if swinging.angle.abs() > stats.swing_too_far_angle {
      stats.swing_too_far_gravity
    } else {
      stats.swing_gravity
    };
    let control = controls.movement.x.signum();
    let acc =
      -gravity * swinging.angle.sin() + -control * stats.player_swing_acc;
    let friction = (swinging.vel * swinging.vel)
      * stats.swing_friction
      * swinging.vel.signum();

    swinging.vel += acc * dt - friction * dt;
    swinging.vel = swinging
      .vel
      .clamp(-stats.swing_terminal_vel, stats.swing_terminal_vel);
    swinging.angle += swinging.vel * dt;

    println!("{} -> {}", swinging.vel, swinging.angle);
    let player_pos = access.query::<&Positioned>(entity).unwrap();
    let mut player_vel = access.query::<&mut Velocitized>(entity).unwrap();
    let ideal_player_loc = swinging.anchor_pos
      - Vec2::from_angle(swinging.angle - TAU / 4.0) * stats.rod_anchor_dist;
    let vel =
      ideal_player_loc - vec2(player_pos.pos.x as _, player_pos.pos.y as _);
    player_vel.vel = vel / dt;

    if controls.jump || !controls.swing || ks.touching_any() {
      // If we're *jumping* off the rod, leave it there.
      if !controls.jump {
        if self.deployed_rod_entity == Some(swinging.swingee) {
          access.lazy_despawn(swinging.swingee);
          self.deployed_rod_entity = None;
        }
      }

      let cheated_angle =
        if swinging.angle.abs() > stats.angle_to_cheat_launch_vel_at {
          let reduced_extra = (swinging.angle.abs()
            - stats.angle_to_cheat_launch_vel_at)
            / stats.angle_launch_vel_cheat_factor;
          swinging.angle.signum()
            * (stats.angle_to_cheat_launch_vel_at + reduced_extra)
        } else {
          swinging.angle
        };

      // the angle is rotated, so it *should* have x be sin and y be cos...
      // but we also want to launch normal to the launch angle!
      // so it undoes itself
      let raw_launch_x = -cheated_angle.cos() * stats.swing_vel_to_vel_rate_x;
      let raw_launch_y = -cheated_angle.sin() * stats.swing_vel_to_vel_rate_y;
      let launch_vel =
        vec2(raw_launch_x, raw_launch_y) * swinging.vel * stats.rod_anchor_dist;

      player_vel.vel = launch_vel;
      self.state = PlayerState::Normal(Normal {
        state: NormalState::Falling,
        was_swinging: true,
      });
    }
  }
}
