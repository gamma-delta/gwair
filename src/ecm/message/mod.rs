use aglet::CoordVec;
use macroquad::prelude::Vec2;
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
#[derive(Debug)]
pub struct MsgRecvHit {
    bonker: Entity,
    normal: Vec2,
}
impl Message for MsgRecvHit {}
impl MsgRecvHit {
    pub fn new(bonker: Entity, normal: Vec2) -> Self {
        Self { bonker, normal }
    }

    pub fn bonker(&self) -> Entity {
        self.bonker
    }

    pub fn normal(&self) -> Vec2 {
        self.normal
    }
}

/// Sent to movers when it hits a collider.
///
/// The normal is the direction it bounces against (so the normal of the face of the collider).
#[derive(Debug)]
pub struct MsgSendHit {
    bonkee: Entity,
    normal: Vec2,
}
impl Message for MsgSendHit {}
impl MsgSendHit {
    pub fn new(bonkee: Entity, normal: Vec2) -> Self {
        Self { bonkee, normal }
    }

    pub fn bonkee(&self) -> Entity {
        self.bonkee
    }

    pub fn normal(&self) -> Vec2 {
        self.normal
    }
}
