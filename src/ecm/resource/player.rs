//! https://gmtk.itch.io/platformer-toolkit/devlog/395523/behind-the-code

use aglet::{CoordVec, Direction8};
use palkia::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    controls::ControlState,
    ecm::component::{KinematicState, Positioned, Velocitized},
};

const WALK_TERMINAL_VEL: f32 = 12.0 * 8.0;
/// Can express these in terms of "seconds reqd to get to/from terminal vel."
const WALK_ACCEL: f32 = WALK_TERMINAL_VEL / 0.3;
/// "stop moving in 2 frames"
const WALK_FRICTION: f32 = WALK_TERMINAL_VEL * 60.0 / 2.0;
const WALK_TURN_ACCEL: f32 = WALK_TERMINAL_VEL * 60.0;

const JUMP_HEIGHT: f32 = 40.0;
const TIME_TO_JUMP_APEX: f32 = 0.4;
/// Derived from kinematics
const JUMP_IMPULSE_VEL: f32 = 2.0 * JUMP_HEIGHT / TIME_TO_JUMP_APEX;
/// gravity when rising from a jump
const JUMP_GRAVITY: f32 = JUMP_IMPULSE_VEL / TIME_TO_JUMP_APEX;
/// gravity when rising from a jump but not holding jump
const JUMP_RELEASE_GRAVITY: f32 = JUMP_GRAVITY * 3.0;
/// normal falling gravity
const FALLING_GRAVITY: f32 = JUMP_GRAVITY * 2.5;
const FALL_TERMINAL_VEL: f32 = 300.0;
const PLUMMET_TERMINAL_VEL: f32 = 400.0;
const FALLING_GRAVITY_VERT_VEL_THRESHOLD: f32 = 16.0;

const COYOTE_TIME: f32 = 0.09;
const JUMP_BUFFER_LEN: f32 = 0.1;

#[derive(Serialize, Deserialize)]
pub struct PlayerController {
    entity: Entity,

    is_jumping: bool,
    coyote_countdown: f32,
    was_on_ground: bool,

    was_pressing_jump: bool,
    jump_buffer_countdown: f32,
}

impl PlayerController {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            is_jumping: false,
            coyote_countdown: 0.0,
            was_on_ground: false,
            was_pressing_jump: false,
            jump_buffer_countdown: 0.0,
        }
    }

    pub fn update_from_controls(
        &mut self,
        controls: ControlState,
        world: &World,
        dt: f32,
    ) {
        let mut player_vel =
            world.query::<&mut Velocitized>(self.entity).unwrap();
        let ks = world.query::<&KinematicState>(self.entity).unwrap();

        let on_ground = ks.touching.contains(Direction8::South);

        // Running
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

        // Jumping
        if on_ground {
            self.is_jumping = false;
        }

        if !on_ground && self.was_on_ground {
            self.coyote_countdown = COYOTE_TIME;
        }

        let gravity =
            if player_vel.vel.y < -FALLING_GRAVITY_VERT_VEL_THRESHOLD * dt {
                if self.is_jumping && controls.jump {
                    JUMP_GRAVITY
                } else {
                    JUMP_RELEASE_GRAVITY
                }
            } else {
                FALLING_GRAVITY
            };

        let jump_rising_edge = controls.jump && !self.was_pressing_jump;
        if jump_rising_edge {
            self.jump_buffer_countdown = JUMP_BUFFER_LEN;
        }

        let can_jump =
            !self.is_jumping && (on_ground || self.coyote_countdown > 0.0);
        if can_jump && self.jump_buffer_countdown > 0.0 {
            player_vel.vel.y = -JUMP_IMPULSE_VEL;
            self.is_jumping = true;
        }

        let terminal_vel = if controls.movement.y > 0.0 {
            PLUMMET_TERMINAL_VEL
        } else {
            FALL_TERMINAL_VEL
        };
        player_vel.vel.y =
            move_towards(player_vel.vel.y, terminal_vel, gravity * dt);
        if player_vel.vel.y > 0.0 && on_ground {
            player_vel.vel.y = 0.0;
        }

        if !on_ground {
            self.coyote_countdown = (self.coyote_countdown - dt).max(0.0);
        }
        self.jump_buffer_countdown = (self.jump_buffer_countdown - dt).max(0.0);

        self.was_on_ground = on_ground;
        self.was_pressing_jump = controls.jump;

        if controls.reset {
            let mut pos = world.query::<&mut Positioned>(self.entity).unwrap();
            pos.pos = CoordVec::new(0, 0);
        }
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
