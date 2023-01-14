use aglet::{CoordVec, Direction8};
use ahash::AHashMap;
use dialga::factory::ComponentFactory;
use kdl::KdlNode;
use macroquad::prelude::Vec2;
use palkia::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    ecm::{
        message::{MsgDraw, MsgPhysicsTick, MsgSendHit},
        resource::HitboxTracker,
    },
    fabctx::FabCtx,
    geom::Hitbox,
};

/// Indicates this is placed in the world.
///
/// Local `(0, 0)` is the center.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Positioned {
    pub pos: CoordVec,
}

impl Positioned {
    pub fn new(pos: CoordVec) -> Self {
        Self { pos }
    }

    pub fn from_vec(vec: Vec2) -> Self {
        Self {
            pos: CoordVec::new(vec.x.round() as _, vec.y.round() as _),
        }
    }

    /// Create a hitbox at the center point
    pub fn make_hitbox(&self, dims: HasDims) -> Hitbox {
        Hitbox::new(self.pos.x, self.pos.y, dims.w, dims.h)
    }
}

impl Component for Positioned {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder
            .handle_read(|this, mut msg: MsgDraw, _, _| {
                msg.pos = Some(this.pos);
                msg
            })
            .register_create_callback(|_, me, access| {
                if access.query::<&HasDims>(me).is_some() {
                    let mut tracker =
                        access.write_resource::<HitboxTracker>().unwrap();
                    tracker.on_create(me);
                }
            })
            .register_remove_callback(|_, me, access| {
                // We don't query for HasDims because the entity will be dead right now
                // so we might try to remove some things that aren't in the tracker, oh well
                let mut tracker =
                    access.write_resource::<HitboxTracker>().unwrap();
                tracker.on_remove(me);
            })
    }
}

/// Indicates this has a width and height.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HasDims {
    pub w: i32,
    pub h: i32,
}

impl HasDims {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }

    pub fn make_hitbox(&self, pos: Positioned) -> Hitbox {
        pos.make_hitbox(*self)
    }

    fn on_draw(
        &self,
        mut msg: MsgDraw,
        _me: Entity,
        _access: &ListenerWorldAccess,
    ) -> MsgDraw {
        msg.dims = Some((self.w, self.h));
        msg
    }
}

impl Component for HasDims {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_read(Self::on_draw)
    }
}

/// Data component that indicates this has the ability to move.
///
/// https://maddythorson.medium.com/celeste-and-towerfall-physics-d24bd2ae0fc5
#[derive(Debug, Serialize, Deserialize)]
pub struct Mover {
    pub remainder: Vec2,
}

impl Mover {
    pub fn new() -> Self {
        Self {
            remainder: Vec2::ZERO,
        }
    }

    pub fn move_by(&mut self, dv: Vec2) {
        self.remainder += dv;
    }
}

impl Component for Mover {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder
    }
}

/// Indicates this uses velocity-based movement.
#[derive(Debug, Serialize, Deserialize)]
pub struct Velocitized {
    pub vel: Vec2,
}

impl Velocitized {
    pub fn new(vel: Vec2) -> Self {
        Self { vel }
    }

    pub fn still() -> Self {
        Self::new(Vec2::ZERO)
    }

    pub fn impulse(&mut self, dv: Vec2, terminal_vel: f32, dt: f32) {
        if dv.x != 0.0 || dv.y != 0.0 {
            let target_vel = self.vel + dv * dt;
            let target_vel =
                if target_vel.length_squared() >= terminal_vel * terminal_vel {
                    target_vel.normalize() * terminal_vel
                } else {
                    target_vel
                };
            self.vel = target_vel;
        }
    }
}
impl Component for Velocitized {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_write(|this, msg: MsgPhysicsTick, me, access| {
            // if this.vel.length_squared() < 1.0 * msg.dt() {
            //     this.vel = Vec2::ZERO;
            // }

            let mut mover = access.query::<&mut Mover>(me).unwrap();
            mover.move_by(this.vel * msg.dt());

            msg
        })
    }
}

/// Tracks the directions this is touching something in.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KinematicState {
    /// A "countdown" for each direction.
    touching: AHashMap<Direction8, u8>,
}

impl KinematicState {
    const TOUCH_COUNTDOWN: u8 = 3;

    pub fn touching(&self, direction: Direction8) -> bool {
        match self.touching.get(&direction) {
            Some(countdown) => *countdown > 0,
            None => false,
        }
    }

    pub fn touching_any(&self) -> bool {
        self.touching.values().any(|v| *v > 0)
    }
}

impl Component for KinematicState {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder
            .handle_write(|this, msg: MsgSendHit, _, _| {
                this.touching
                    .insert(msg.normal(), KinematicState::TOUCH_COUNTDOWN);
                msg
            })
            .handle_write(|this, msg: MsgPhysicsTick, _, _| {
                for v in this.touching.values_mut() {
                    *v = v.saturating_sub(1);
                }
                msg
            })
    }
}

/// If this bonks against something remove all velocity in that direction
#[derive(Debug, Serialize, Deserialize)]
pub struct Bonker;

impl Component for Bonker {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_read(|_, msg: MsgSendHit, me, access| {
            let ks = access.query::<&KinematicState>(me).unwrap();
            let mut vel = access.query::<&mut Velocitized>(me).unwrap();

            for horz_dir in [
                Direction8::NorthWest,
                Direction8::West,
                Direction8::SouthWest,
                Direction8::NorthEast,
                Direction8::East,
                Direction8::SouthEast,
            ] {
                if ks.touching(horz_dir) {
                    vel.vel.x = 0.0;
                    break;
                }
            }

            for vert_dir in [
                Direction8::NorthWest,
                Direction8::North,
                Direction8::NorthEast,
                Direction8::SouthWest,
                Direction8::South,
                Direction8::SouthEast,
            ] {
                if ks.touching(vert_dir) {
                    vel.vel.y = 0.0;
                    break;
                }
            }

            msg
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrictionHaver {
    pub friction: f32,
}

impl Component for FrictionHaver {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_write(|this, msg: MsgPhysicsTick, me, access| {
            let mut vel = access.query::<&mut Velocitized>(me).unwrap();

            let fric_coeff = 1.0 - (this.friction * msg.dt());
            if fric_coeff < 0.0 {
                vel.vel = Vec2::ZERO;
            } else {
                vel.vel *= fric_coeff;
            }

            msg
        })
    }
}

/// Marker component for things that should block movement.
#[derive(Debug, Serialize, Deserialize)]
pub struct Collider;
impl Component for Collider {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder
    }
}

// FACTORIES

/// Factory for [`HasDims`], [`Mover`], [`Velocitized`], and [`KinematicState`].
pub struct PhysicFactory;

impl ComponentFactory<FabCtx> for PhysicFactory {
    fn assemble<'a, 'w>(
        &self,
        mut builder: EntityBuilder<'a, 'w>,
        node: &KdlNode,
        _ctx: &FabCtx,
    ) -> eyre::Result<EntityBuilder<'a, 'w>> {
        #[derive(Deserialize)]
        struct Raw {
            width: i32,
            height: i32,
        }

        let raw: Raw = knurdy::deserialize_node(node)?;

        builder.insert(HasDims {
            w: raw.width,
            h: raw.height,
        });
        builder.insert(Mover::new());
        builder.insert(Velocitized::still());
        builder.insert(KinematicState::default());
        Ok(builder)
    }
}

pub struct FrictionFactory;

impl ComponentFactory<FabCtx> for FrictionFactory {
    fn assemble<'a, 'w>(
        &self,
        mut builder: EntityBuilder<'a, 'w>,
        node: &KdlNode,
        _ctx: &FabCtx,
    ) -> eyre::Result<EntityBuilder<'a, 'w>> {
        #[derive(Deserialize)]
        struct Raw(f32);
        let raw: Raw = knurdy::deserialize_node(node)?;
        builder.insert(FrictionHaver { friction: raw.0 });
        Ok(builder)
    }
}
