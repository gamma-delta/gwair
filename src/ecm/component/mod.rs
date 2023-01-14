mod gfx;
mod phys;
mod player;
mod swinging;

pub use gfx::*;
pub use phys::*;
pub use player::*;
pub use swinging::*;

use palkia::prelude::*;
use serde::{Deserialize, Serialize};

use super::message::MsgTick;

/// Despawns after the given number of frames.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LimitedTimeOffer {
    time_remaining: u32,
}

impl Component for LimitedTimeOffer {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_write(|this, msg: MsgTick, me, access| {
            if this.time_remaining == 0 {
                access.lazy_despawn(me);
            } else {
                this.time_remaining -= 1;
            }

            msg
        })
    }
}

/// Keeps track of its age.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgeTracker {
    #[serde(default)]
    age: u64,
}

impl Component for AgeTracker {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_write(|this, msg: MsgTick, _, _| {
            this.age += 1;

            msg
        })
    }
}
