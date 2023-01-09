//! https://gmtk.itch.io/platformer-toolkit/devlog/395523/behind-the-code

use aglet::Direction8;
use palkia::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    controls::ControlState,
    ecm::component::{KinematicState, Velocitized},
};

const WALK_ACCEL: f32 = 40.0;
const WALK_FRICTION: f32 = 60.0;
const WALK_TURN_SPEED: f32 = 120.0;

const WALK_TERMINAL_VEL: f32 = 200.0;

#[derive(Serialize, Deserialize)]
pub struct PlayerController {
    entity: Entity,
}

impl PlayerController {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn new(entity: Entity) -> Self {
        Self { entity }
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
        let walk_turn = WALK_TURN_SPEED;

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

        player_vel.vel.x = move_towards(player_vel.vel.x, target_vel_x, acc);

        if controls.jump {
            println!("{}", player_vel.vel.x);
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

#[test]
fn aaa() {
    let mut vel = 0.0;
    for _ in 0..20 {
        vel = move_towards(vel, 20.0 / 60.0, 0.5 / 60.0);
        println!("{}", vel);
    }
}
