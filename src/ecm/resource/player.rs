use palkia::prelude::*;

use crate::{
    controls::ControlState,
    ecm::component::{KinematicState, Velocitized},
};

const WALK_ACC: f32 = 32.0;
const WALK_SPEED: f32 = 128.0;

pub struct PlayerController {
    entity: Entity,
}

enum PlayerState {
    Grounded,
    JumpingUp,
    Falling,
}

impl PlayerController {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn update_from_controls(
        &mut self,
        controls: ControlState,
        world: &World,
        dt: f32,
    ) {
        // temp
        let mut player_vel =
            world.query::<&mut Velocitized>(self.entity).unwrap();
        player_vel.impulse(controls.movement * WALK_ACC, WALK_SPEED, dt);

        if controls.jump {
            let ks = world.query::<&KinematicState>(self.entity).unwrap();
            println!("{}", ks.touching);
        }
    }
}

impl Resource for PlayerController {}
