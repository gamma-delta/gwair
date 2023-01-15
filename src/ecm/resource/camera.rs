use aglet::CoordVec;
use palkia::prelude::*;

/// Where the world is viewed from
#[derive(Debug)]
pub struct Camera {
    x: ViewAxis,
    y: ViewAxis,
}
impl Resource for Camera {}

impl Camera {
    pub fn update(&mut self, player_pos: CoordVec) {
        self.x.update(player_pos.x);
        self.y.update(player_pos.y);
    }

    pub fn center(&self) -> CoordVec {
        CoordVec::new(self.x.get(), self.y.get())
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: ViewAxis::Static(0),
            y: ViewAxis::FollowEased { here: 0 },
        }
    }
}

#[derive(Debug)]
enum ViewAxis {
    Static(i32),
    /// Follow the player
    FollowEased {
        here: i32,
    },
}

impl ViewAxis {
    const EASING_AMOUNT: i32 = 4;

    fn update(&mut self, pos: i32) {
        match self {
            ViewAxis::Static(_) => {}
            ViewAxis::FollowEased { ref mut here } => {
                let delta = *here - pos;
                *here = pos + delta / Self::EASING_AMOUNT;
            }
        }
    }

    fn get(&self) -> i32 {
        match self {
            ViewAxis::FollowEased { here } => *here,
            ViewAxis::Static(here) => *here,
        }
    }
}
