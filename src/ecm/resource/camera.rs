use aglet::{Area, CoordVec};
use palkia::prelude::*;

use crate::gfx::{GAME_HEIGHT, GAME_WIDTH};

/// Where the world is viewed from
#[derive(Debug)]
pub struct Camera {
    current: CoordVec,

    bb_corner: CoordVec,
    bb_size: CoordVec,
}
impl Resource for Camera {}

impl Camera {
    const EASING_AMOUNT: i32 = 4;

    pub fn new() -> Self {
        Self {
            current: CoordVec::new(0, 0),
            bb_corner: CoordVec::new(-160, -10_000),
            bb_size: CoordVec::new(320, 20_000),
        }
    }

    pub fn update(&mut self, player_pos: CoordVec) {
        for (slot, player, corner, size, window_size) in [
            (
                &mut self.current.x,
                player_pos.x,
                self.bb_corner.x,
                self.bb_size.x,
                GAME_WIDTH as i32,
            ),
            (
                &mut self.current.y,
                player_pos.y,
                self.bb_corner.y,
                self.bb_size.y,
                GAME_HEIGHT as i32,
            ),
        ] {
            let delta = *slot - player;
            let ideal_pos = player + delta / Self::EASING_AMOUNT;
            *slot = ideal_pos.clamp(
                corner + window_size / 2,
                corner + size - window_size / 2,
            );
        }
    }

    pub fn center(&self) -> CoordVec {
        self.current
    }
}
