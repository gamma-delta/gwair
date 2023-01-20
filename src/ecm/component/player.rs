//! https://gmtk.itch.io/platformer-toolkit/devlog/395523/behind-the-code

mod stats;

use std::f32::consts::{PI, TAU};

use aglet::{CoordVec, Direction8};
use dialga::factory::ComponentFactory;
use glam::{vec2, Vec2};
use kdl::KdlNode;
use macroquad::prelude::{self as mq, Color};
use palkia::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
  controls::ControlState,
  ecm::{
    component::{
      KinematicState, PickuppableRod, Positioned, SwingableOn, Velocitized,
    },
    message::{MsgDraw, MsgPhysicsTick},
    resource::{Camera, FabCtxHolder, TreeHolder},
  },
  fabctx::FabCtx,
  geom::Hitbox,
  gfx::{de_hexcol, hexcol, ser_hexcol},
  resources::Resources,
};

use self::stats::PlayerStats;

use super::HasDims;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerController {
  was_pressing_jump: bool,
  jump_buffer_countdown: f32,

  rod_deployments_left: u32,

  state: PlayerState,
  #[serde(serialize_with = "ser_hexcol")]
  #[serde(deserialize_with = "de_hexcol")]
  color: Color,

  stats: PlayerStats,
}

impl Component for PlayerController {
  fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
  where
    Self: Sized,
  {
    builder
      .handle_write(|this, msg: MsgPhysicsTick, me, access| {
        let controls = ControlState::calculate();
        this.update_from_controls(me, msg.dt(), controls, access);
        msg
      })
      .handle_read(|this, msg: MsgDraw, me, access| {
        let pos = access.query::<&Positioned>(me).unwrap();
        let dims = access.query::<&HasDims>(me).unwrap();
        let cam = access.read_resource::<Camera>().unwrap();

        let corner =
          pos.pos - CoordVec::new(dims.w / 2, dims.h / 2) - cam.center();
        mq::draw_rectangle(
          corner.x as f32,
          corner.y as f32,
          dims.w as f32,
          dims.h as f32,
          this.color,
        );

        if this.stats.debugdraw_grab_hbs {
          let controls = ControlState::calculate();
          let anchor_delta = if controls.movement.length_squared() < 0.0001 {
            Vec2::new(0.0, -1.0)
          } else {
            controls.movement.normalize()
          };
          let player_pos = access.query::<&Positioned>(me).unwrap();
          for hb in
            grab_extant_rod_hbs(player_pos.pos, anchor_delta, &this.stats)
          {
            mq::draw_rectangle(
              (hb.x() - cam.center().x) as f32,
              (hb.y() - cam.center().y) as f32,
              hb.w() as _,
              hb.h() as _,
              mq::Color::from_rgba(255, 120, 0, 100),
            );
          }
        }

        msg
      })
  }
}

impl PlayerController {
  pub fn new(color: Color) -> Self {
    Self {
      was_pressing_jump: false,
      jump_buffer_countdown: 0.0,

      rod_deployments_left: 0,
      color,

      state: PlayerState::default(),

      stats: PlayerStats::default(),
    }
  }

  pub fn update_from_controls(
    &mut self,
    me: Entity,
    dt: f32,
    controls: ControlState,
    access: &ListenerWorldAccess,
  ) {
    self.check_start_swinging(controls, access, me);

    match self.state {
      PlayerState::Normal(..) => {
        self.normal_movement(me, dt, controls, access);
      }
      PlayerState::Swinging(..) => {
        self.swinging_movement(access, me, controls, dt)
      }
    }

    let jump_rising_edge = controls.jump && !self.was_pressing_jump;
    if jump_rising_edge {
      self.jump_buffer_countdown = self.stats.jump_buffer_len;
    }
    self.jump_buffer_countdown = (self.jump_buffer_countdown - dt).max(0.0);

    self.was_pressing_jump = controls.jump;

    if controls.reset {
      let mut pos = access.query::<&mut Positioned>(me).unwrap();
      pos.pos = CoordVec::new(0, 0);
    }
  }

  fn check_start_swinging(
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
          } else if self.rod_deployments_left > 0 {
            self.rod_deployments_left -= 1;
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

  fn swinging_movement(
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

    if !controls.swing || ks.touching_any() {
      // If we *just* placed our own rod this frame, trying to see if it's
      // swingable will panic because it's not finalized yet.
      // For now assume any half-formed entity is ours.
      if access.liveness(swinging.swingee) == EntityLiveness::PartiallySpawned
        || access.query::<&PickuppableRod>(swinging.swingee).is_some()
      {
        access.lazy_despawn(swinging.swingee);
        // self.rod_deployments_left += 1;
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
      let launch_vel = -Vec2::from_angle(cheated_angle)
        * swinging.vel
        * stats.rod_anchor_dist
        * stats.swing_vel_to_vel_rate;

      player_vel.vel = launch_vel;
      self.state = PlayerState::Normal(Normal {
        state: NormalState::Falling,
        was_swinging: true,
      });
    }
  }

  fn normal_movement(
    &mut self,
    entity: Entity,
    dt: f32,
    controls: ControlState,
    access: &ListenerWorldAccess,
  ) {
    let stats = &self.stats;

    let normal = match self.state {
      PlayerState::Normal(ref mut it) => it,
      _ => unreachable!(),
    };

    let mut player_vel = access.query::<&mut Velocitized>(entity).unwrap();
    let ks = access.query::<&KinematicState>(entity).unwrap();
    let on_ground = ks.touching(Direction8::South);

    if on_ground {
      self.rod_deployments_left = stats.rod_deployments_from_ground;
    }

    let walk_acc = stats.walk_accel;
    let walk_dec = stats.walk_friction;
    let walk_turn = stats.walk_turn_accel;
    let target_vel_x = controls.movement.x * stats.walk_terminal_vel;
    let acc = if controls.movement.x == 0.0 {
      walk_dec
    } else if player_vel.vel.x == 0.0
      || controls.movement.x.signum() == player_vel.vel.x.signum()
    {
      walk_acc
    } else {
      walk_turn
    };
    player_vel.vel.x = move_towards(player_vel.vel.x, target_vel_x, acc * dt);

    let state2 = match normal.state {
      NormalState::OnGround => {
        if !on_ground {
          Some(NormalState::FallingFromLedge {
            coyote_countdown: stats.coyote_time,
          })
        } else if self.jump_buffer_countdown > 0.0 {
          Some(NormalState::JumpingUp)
        } else {
          None
        }
      }
      NormalState::FallingFromLedge { coyote_countdown } => {
        if on_ground {
          Some(NormalState::OnGround)
        } else if coyote_countdown > 0.0 && self.jump_buffer_countdown > 0.0 {
          Some(NormalState::JumpingUp)
        } else {
          Some(NormalState::FallingFromLedge {
            coyote_countdown: (coyote_countdown - dt).max(0.0),
          })
        }
      }
      NormalState::JumpingUp => {
        if controls.jump && player_vel.vel.y < 0.0 {
          None
        } else {
          Some(NormalState::Falling)
        }
      }
      NormalState::Falling => {
        if on_ground {
          Some(NormalState::OnGround)
        } else {
          None
        }
      }
    };
    if let Some(state2) = state2 {
      if matches!(&state2, NormalState::JumpingUp) {
        player_vel.vel.y = -stats.jump_impulse_vel;
      }
      println!("changing to {:?}", &state2);
      normal.state = state2;
    }

    let gravity = match normal.state {
      NormalState::OnGround => stats.falling_gravity,
      NormalState::FallingFromLedge { coyote_countdown } => {
        if coyote_countdown > 0.0 {
          stats.coyote_gravity
        } else {
          stats.falling_gravity
        }
      }
      NormalState::JumpingUp => {
        if controls.jump {
          stats.jump_gravity
        } else {
          stats.jump_release_gravity
        }
      }
      NormalState::Falling => stats.falling_gravity,
    };
    let terminal_vel = if controls.movement.y > 0.0 {
      stats.plummet_terminal_vel
    } else {
      stats.fall_terminal_vel
    };
    player_vel.vel.y =
      move_towards(player_vel.vel.y, terminal_vel, gravity * dt);
  }
}

fn move_towards(src: f32, target: f32, max_delta: f32) -> f32 {
  if max_delta == 0.0 || src == target {
    return src;
  }

  let target_delta = target - src;
  let sign = target_delta.signum();
  let delta = max_delta.min(target_delta.abs());
  src + delta * sign
}

#[derive(Debug, Serialize, Deserialize)]
enum PlayerState {
  Normal(Normal),
  Swinging(Swinging),
}

impl Default for PlayerState {
  fn default() -> Self {
    Self::Normal(Normal::default())
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Normal {
  state: NormalState,
  /// prevent swinging until swing is released and pressed again
  was_swinging: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
enum NormalState {
  #[default]
  OnGround,
  FallingFromLedge {
    coyote_countdown: f32,
  },
  JumpingUp,
  Falling,
}

#[derive(Debug, Serialize, Deserialize)]
struct Swinging {
  /// 0 = straight down; tau/4 = left
  angle: f32,
  vel: f32,
  anchor_pos: Vec2,
  swingee: Entity,
}

// ===

pub struct PlayerFactory;

impl ComponentFactory<FabCtx> for PlayerFactory {
  fn assemble<'a, 'w>(
    &self,
    mut builder: EntityBuilder<'a, 'w>,
    node: &KdlNode,
    _ctx: &FabCtx,
  ) -> eyre::Result<EntityBuilder<'a, 'w>> {
    #[derive(Deserialize)]
    struct Raw {
      color: u32,
    }

    let raw: Raw = knurdy::deserialize_node(node)?;

    builder.insert(PlayerController::new(hexcol(raw.color)));
    Ok(builder)
  }
}

fn grab_extant_rod_hbs(
  player_pos: CoordVec,
  anchor_delta: Vec2,
  stats: &PlayerStats,
) -> impl Iterator<Item = Hitbox> + '_ {
  let start_center =
    vec2(player_pos.x as f32, player_pos.y as f32) + anchor_delta;
  let start_hb = Hitbox::new(
    start_center.x.round() as i32,
    start_center.y.round() as i32,
    stats.grab_extant_start_size * 2,
    stats.grab_extant_start_size * 2,
  );

  std::iter::once(start_hb).chain((1..stats.grab_extant_step_count).map(
    move |i| {
      let check_center = vec2(player_pos.x as f32, player_pos.y as f32)
        + anchor_delta * (i as f32 * stats.grab_extant_step_size);
      let radius = stats.grab_extant_swingable_radius
        + i as i32 * stats.grab_extant_swingable_radius_increment;
      Hitbox::new(
        check_center.x.round() as i32,
        check_center.y.round() as i32,
        radius * 2,
        radius * 2,
      )
    },
  ))
}
