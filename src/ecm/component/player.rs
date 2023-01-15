//! https://gmtk.itch.io/platformer-toolkit/devlog/395523/behind-the-code

use std::f32::consts::{PI, TAU};

use aglet::{CoordVec, Direction8};
use dialga::factory::ComponentFactory;
use glam::{vec2, Vec2};
use kdl::KdlNode;
use palkia::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    controls::ControlState,
    ecm::{
        component::{KinematicState, Positioned, Velocitized},
        message::{MsgPhysicsTick, MsgTick},
        resource::{Camera, FabCtxHolder},
    },
    fabctx::FabCtx,
    resources::Resources,
};

const WALK_TERMINAL_VEL: f32 = 12.0 * 8.0;
/// Can express these in terms of "seconds reqd to get to/from terminal vel."
const WALK_ACCEL: f32 = WALK_TERMINAL_VEL / 0.3;
/// "stop moving in 2 frames"
const WALK_FRICTION: f32 = WALK_TERMINAL_VEL * 60.0 / 2.0;
const WALK_TURN_ACCEL: f32 = WALK_TERMINAL_VEL * 60.0;

const JUMP_HEIGHT: f32 = 40.0;
const TIME_TO_JUMP_APEX: f32 = 0.45;
/// Derived from kinematics
const JUMP_IMPULSE_VEL: f32 = 2.0 * JUMP_HEIGHT / TIME_TO_JUMP_APEX;
/// gravity when rising from a jump
const JUMP_GRAVITY: f32 = JUMP_IMPULSE_VEL / TIME_TO_JUMP_APEX;
/// gravity when rising from a jump but not holding jump
const JUMP_RELEASE_GRAVITY: f32 = JUMP_GRAVITY * 3.0;
/// normal falling gravity
const FALLING_GRAVITY: f32 = JUMP_GRAVITY * 2.5;
/// Grace-period gravity when falling off a ledge
const COYOTE_GRAVITY: f32 = FALLING_GRAVITY * 0.5;

const FALL_TERMINAL_VEL: f32 = 300.0;
const PLUMMET_TERMINAL_VEL: f32 = 400.0;

const COYOTE_TIME: f32 = 0.05;
const JUMP_BUFFER_LEN: f32 = 0.1;

const ROD_ANCHOR_DIST: f32 = 12.0;
const VEL_TO_SWING_VEL_RATE: f32 = 0.05;
const SWING_GRAVITY: f32 = 5.0;
const SWING_FRICTION: f32 = 0.05;
const SWING_TOO_FAR_ANGLE: f32 = TAU / 4.0;
const SWING_TOO_FAR_GRAVITY: f32 = 10.0;

const PLAYER_SWING_ACC: f32 = 4.0;

const SWING_TERMINAL_VEL: f32 = 10.0;
const SWING_VEL_TO_VEL_RATE: f32 = 2.15;
/// If the angle is over horizontal, cheat in favor of the player
/// and make it a little smaller.
const ANGLE_TO_CHEAT_VEL_AT: f32 = TAU * 0.225;
const ANGLE_VEL_CHEAT_FACTOR: f32 = 2.0;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerController {
    was_pressing_jump: bool,
    jump_buffer_countdown: f32,

    state: PlayerState,
}

impl Component for PlayerController {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_write(|this, msg: MsgPhysicsTick, me, access| {
            let controls = ControlState::calculate();
            this.update_from_controls(me, msg.dt(), controls, access);
            msg
        })
    }
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            was_pressing_jump: false,
            jump_buffer_countdown: 0.0,

            state: PlayerState::default(),
        }
    }

    pub fn update_from_controls(
        &mut self,
        entity: Entity,
        dt: f32,
        controls: ControlState,
        access: &ListenerWorldAccess,
    ) {
        if let PlayerState::Normal(n) = &mut self.state {
            if controls.swing {
                let can_swing = match n.state {
                    NormalState::OnGround => false,
                    NormalState::FallingFromLedge { .. }
                    | NormalState::JumpingUp
                    | NormalState::Falling => true,
                };
                if !n.was_swinging && can_swing {
                    let player_vel =
                        access.query::<&Velocitized>(entity).unwrap();
                    let player_pos =
                        access.query::<&Positioned>(entity).unwrap();

                    let anchor_delta =
                        if controls.movement.length_squared() < 0.0001 {
                            Vec2::new(0.0, -1.0)
                        } else {
                            controls.movement.normalize()
                        };
                    // how much in common does the player vel have with
                    // orthagonal to the anchor delta?
                    dbg!(player_vel.vel, anchor_delta);
                    // vector rejection, but with a sign also
                    let rej = player_vel
                        .vel
                        .reject_from_normalized(anchor_delta)
                        .length();
                    let perp_dot = player_vel.vel.perp_dot(anchor_delta);
                    let vel = rej * perp_dot.signum() * VEL_TO_SWING_VEL_RATE;

                    // We consider an angle of 0 to be straight down, so we
                    // need the angle between down.
                    let angle = vec2(0.0, -1.0).angle_between(anchor_delta);
                    println!("initial: {} {}", vel, angle);

                    let anchor_pos =
                        vec2(player_pos.pos.x as f32, player_pos.pos.y as f32)
                            + anchor_delta * ROD_ANCHOR_DIST;

                    // so we can see it
                    let rod = {
                        let res = Resources::get();
                        let ctx =
                            access.read_resource::<FabCtxHolder>().unwrap();
                        res.fabber()
                            .instantiate(
                                "rod",
                                access
                                    .lazy_spawn()
                                    .with(Positioned::from_vec(anchor_pos)),
                                &ctx.0,
                            )
                            .unwrap()
                    };

                    self.state = PlayerState::Swinging(Swinging {
                        angle,
                        vel,
                        anchor_pos,
                        swingee: rod,
                    });
                }
            } else {
                n.was_swinging = false;
            }
        }

        match self.state {
            PlayerState::Normal(..) => {
                self.normal_movement(entity, dt, controls, access);
            }
            PlayerState::Swinging(ref mut swinging) => {
                self.swinging_movement(access, entity, controls, dt)
            }
        }

        let jump_rising_edge = controls.jump && !self.was_pressing_jump;
        if jump_rising_edge {
            self.jump_buffer_countdown = JUMP_BUFFER_LEN;
        }
        self.jump_buffer_countdown = (self.jump_buffer_countdown - dt).max(0.0);

        self.was_pressing_jump = controls.jump;

        if controls.reset {
            let mut pos = access.query::<&mut Positioned>(entity).unwrap();
            pos.pos = CoordVec::new(0, 0);
        }
    }

    fn swinging_movement(
        &mut self,
        access: &ListenerWorldAccess,
        entity: Entity,
        controls: ControlState,
        dt: f32,
    ) {
        let swinging = match self.state {
            PlayerState::Swinging(ref mut it) => it,
            _ => unreachable!(),
        };

        let ks = access.query::<&KinematicState>(entity).unwrap();

        swinging.angle = (swinging.angle + PI).rem_euclid(TAU) - PI;

        let gravity = if swinging.angle.abs() > SWING_TOO_FAR_ANGLE {
            SWING_TOO_FAR_GRAVITY
        } else {
            SWING_GRAVITY
        };
        let control = controls.movement.x.signum();
        let acc = -gravity * swinging.angle.sin() + -control * PLAYER_SWING_ACC;
        let friction = (swinging.vel * swinging.vel)
            * SWING_FRICTION
            * swinging.vel.signum();

        swinging.vel += (acc * dt - friction * dt)
            .clamp(-SWING_TERMINAL_VEL, SWING_TERMINAL_VEL);
        swinging.angle += swinging.vel * dt;

        println!("{} -> {}", swinging.vel, swinging.angle);
        let player_pos = access.query::<&Positioned>(entity).unwrap();
        let mut player_vel = access.query::<&mut Velocitized>(entity).unwrap();
        let ideal_player_loc = swinging.anchor_pos
            - Vec2::from_angle(swinging.angle - TAU / 4.0) * ROD_ANCHOR_DIST;
        let vel = ideal_player_loc
            - vec2(player_pos.pos.x as _, player_pos.pos.y as _);
        player_vel.vel = vel / dt;

        if !controls.swing || ks.touching_any() {
            access.lazy_despawn(swinging.swingee);

            let cheated_angle = if swinging.angle.abs() > ANGLE_TO_CHEAT_VEL_AT
            {
                let reduced_extra = (swinging.angle.abs()
                    - ANGLE_TO_CHEAT_VEL_AT)
                    / ANGLE_VEL_CHEAT_FACTOR;
                swinging.angle.signum()
                    * (ANGLE_TO_CHEAT_VEL_AT + reduced_extra)
            } else {
                swinging.angle
            };
            let launch_vel = -Vec2::from_angle(cheated_angle)
                * swinging.vel
                * ROD_ANCHOR_DIST
                * SWING_VEL_TO_VEL_RATE;

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
        let normal = match self.state {
            PlayerState::Normal(ref mut it) => it,
            _ => unreachable!(),
        };

        let mut player_vel = access.query::<&mut Velocitized>(entity).unwrap();
        let ks = access.query::<&KinematicState>(entity).unwrap();
        let on_ground = ks.touching(Direction8::South);

        let walk_acc = WALK_ACCEL;
        let walk_dec = WALK_FRICTION;
        let walk_turn = WALK_TURN_ACCEL;
        let target_vel_x = controls.movement.x * WALK_TERMINAL_VEL;
        let acc = if controls.movement.x == 0.0 {
            walk_dec
        } else if player_vel.vel.x == 0.0
            || controls.movement.x.signum() == player_vel.vel.x.signum()
        {
            walk_acc
        } else {
            walk_turn
        };
        player_vel.vel.x =
            move_towards(player_vel.vel.x, target_vel_x, acc * dt);

        let state2 = match normal.state {
            NormalState::OnGround => {
                if !on_ground {
                    Some(NormalState::FallingFromLedge {
                        coyote_countdown: COYOTE_TIME,
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
                } else if coyote_countdown > 0.0
                    && self.jump_buffer_countdown > 0.0
                {
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
                player_vel.vel.y = -JUMP_IMPULSE_VEL;
            }
            println!("changing to {:?}", &state2);
            normal.state = state2;
        }

        let gravity = match normal.state {
            NormalState::OnGround => FALLING_GRAVITY,
            NormalState::FallingFromLedge { coyote_countdown } => {
                if coyote_countdown > 0.0 {
                    COYOTE_GRAVITY
                } else {
                    FALLING_GRAVITY
                }
            }
            NormalState::JumpingUp => {
                if controls.jump {
                    JUMP_GRAVITY
                } else {
                    JUMP_RELEASE_GRAVITY
                }
            }
            NormalState::Falling => FALLING_GRAVITY,
        };
        let terminal_vel = if controls.movement.y > 0.0 {
            PLUMMET_TERMINAL_VEL
        } else {
            FALL_TERMINAL_VEL
        };
        player_vel.vel.y =
            move_towards(player_vel.vel.y, terminal_vel, gravity * dt);
    }
}

impl Resource for PlayerController {}

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
        _node: &KdlNode,
        _ctx: &FabCtx,
    ) -> eyre::Result<EntityBuilder<'a, 'w>> {
        builder.insert(PlayerController::new());
        Ok(builder)
    }
}
