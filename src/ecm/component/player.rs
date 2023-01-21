//! https://gmtk.itch.io/platformer-toolkit/devlog/395523/behind-the-code

mod stats;
mod swinging;

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
    component::{KinematicState, Positioned, Velocitized},
    message::{MsgDraw, MsgPhysicsTick},
    resource::Camera,
  },
  fabctx::FabCtx,
  geom::{signum0, Hitbox},
  gfx::{de_hexcol, hexcol, ser_hexcol},
};

use self::stats::PlayerStats;

use super::HasDims;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerController {
  was_pressing_jump: bool,
  jump_buffer_countdown: f32,

  /// Set to true when deploying the rod, set to false when hitting the
  /// ground.
  deployed_rod_in_air: bool,
  /// If this is `Some`, it's a reference to our very own immovable rod that
  /// we've placed.
  deployed_rod_entity: Option<Entity>,

  state: PlayerState,
  #[serde(serialize_with = "ser_hexcol")]
  #[serde(deserialize_with = "de_hexcol")]
  color: Color,

  stats: PlayerStats,

  /// For the benefit of drawing.
  /// The skip attr will "deserialize" it as default
  #[serde(skip)]
  cached_controls: Option<ControlState>,
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
      .handle_read(Self::on_draw)
  }
}

impl PlayerController {
  pub fn new(color: Color) -> Self {
    Self {
      was_pressing_jump: false,
      jump_buffer_countdown: 0.0,

      deployed_rod_in_air: false,
      deployed_rod_entity: None,

      color,

      state: PlayerState::default(),

      stats: PlayerStats::default(),

      cached_controls: None,
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

    self.cached_controls = Some(controls);
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
      self.deployed_rod_in_air = false;
    }

    let (accel, friction, turn_accel, terminal_vel, overfriction) = if on_ground
    {
      (
        stats.walk_accel,
        stats.walk_friction,
        stats.walk_turn_accel,
        stats.walk_terminal_vel,
        stats.walk_overfast_friction,
      )
    } else {
      (
        stats.air_accel,
        stats.air_friction,
        stats.air_turn_accel,
        stats.air_terminal_vel,
        stats.air_overfast_friction,
      )
    };

    let target_vel_x = controls.movement.x * terminal_vel;
    let (acc, dec) = if controls.movement.x == 0.0 {
      // slowing down to a stop
      (turn_accel, friction)
    } else if player_vel.vel.x == 0.0
      || signum0(controls.movement.x) == signum0(player_vel.vel.x)
    {
      // from a standstill, or when moving in the same direction
      // as the control. speed up with normal walk speed, slow down
      // more slowly.
      (accel, overfriction)
    } else {
      // turning around.
      (turn_accel, friction)
    };
    player_vel.vel.x =
      accelerate_towards(player_vel.vel.x, target_vel_x, acc * dt, dec * dt);

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
    let (terminal_vel, decel) = if controls.movement.y > 0.0 {
      (stats.plummet_terminal_vel, stats.plummet_friction_y)
    } else {
      (stats.fall_terminal_vel, stats.fall_friction_y)
    };
    player_vel.vel.y = accelerate_towards(
      player_vel.vel.y,
      terminal_vel,
      gravity * dt,
      decel * dt,
    );
  }

  fn on_draw(
    &self,
    msg: MsgDraw,
    me: Entity,
    access: &ListenerWorldAccess,
  ) -> MsgDraw {
    let pos = access.query::<&Positioned>(me).unwrap();
    let dims = access.query::<&HasDims>(me).unwrap();
    let cam = access.read_resource::<Camera>().unwrap();
    let corner = pos.pos - CoordVec::new(dims.w / 2, dims.h / 2) - cam.center();
    mq::draw_rectangle(
      corner.x as f32,
      corner.y as f32,
      dims.w as f32,
      dims.h as f32,
      self.color,
    );
    if self.stats.debugdraw_grab_hbs {
      if let Some(ref controls) = self.cached_controls {
        let anchor_delta = if controls.movement.length_squared() < 0.0001 {
          Vec2::new(0.0, -1.0)
        } else {
          controls.movement.normalize()
        };
        let player_pos = access.query::<&Positioned>(me).unwrap();
        for hb in grab_extant_rod_hbs(player_pos.pos, anchor_delta, &self.stats)
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
    }
    msg
  }
}

fn accelerate_towards(src: f32, target: f32, accel: f32, decel: f32) -> f32 {
  if accel == 0.0 || src == target {
    return src;
  }

  // if we are going *faster* than the target, use decel;
  // otherwise use accel.
  // the problem is figuring out what "faster" means.
  // we know if the target is pointing positive, negative, or zero...
  let target_delta = target - src;
  let acc = if target.abs() > src.abs() {
    accel.min(target_delta.abs())
  } else {
    decel.min(target_delta.abs())
  };

  src + acc * target_delta.signum()
}

#[test]
fn acc() {
  let mut v = 10.0;
  for _ in 0..30 {
    v = accelerate_towards(v, -5.0, 0.5, 1.0);
    println!("{:?}", v);
  }
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
