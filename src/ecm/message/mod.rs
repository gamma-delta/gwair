use aglet::{CoordVec, Direction8};

use palkia::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct MsgTick;
impl Message for MsgTick {}

#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPhysicsTick {
    dt: f32,
}
impl MsgPhysicsTick {
    pub fn new(dt: f32) -> Self {
        Self { dt }
    }
    pub fn dt(&self) -> f32 {
        self.dt
    }
}
impl Message for MsgPhysicsTick {}

#[derive(Debug, Clone, Default)]
pub struct MsgDraw {
    pub pos: Option<CoordVec>,
    pub dims: Option<(i32, i32)>,
}
impl Message for MsgDraw {}

/// Sent to colliders when an entity hits it.
///
/// The normal is the direction it is getting hit from,
/// so it points from `bonker` to the entitty getting this message.
#[derive(Debug)]
pub struct MsgRecvHit {
    bonker: Entity,
    normal: Direction8,
}
impl Message for MsgRecvHit {}
impl MsgRecvHit {
    pub fn new(bonker: Entity, normal: Direction8) -> Self {
        Self { bonker, normal }
    }

    pub fn bonker(&self) -> Entity {
        self.bonker
    }

    /// Directions can be orthagonal or cornered; hence direction8
    pub fn normal(&self) -> Direction8 {
        self.normal
    }
}

/// Sent to movers when it hits a collider.
///
/// The normal is the direction it is hitting in, so it points from the
/// entity getting this message to `bonkee`.
#[derive(Debug)]
pub struct MsgSendHit {
    bonkee: Entity,
    normal: Direction8,
}
impl Message for MsgSendHit {}
impl MsgSendHit {
    pub fn new(bonkee: Entity, normal: Direction8) -> Self {
        Self { bonkee, normal }
    }

    pub fn bonkee(&self) -> Entity {
        self.bonkee
    }

    pub fn normal(&self) -> Direction8 {
        self.normal
    }
}
