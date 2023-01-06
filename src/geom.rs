use aglet::CoordVec;
use bitflags::bitflags;
use broccoli::{
    aabb::{Aabb, ManySwap},
    axgeom::{Rect, Vec2 as BrocVec2},
};
use palkia::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Hitbox(
    #[serde(serialize_with = "ser_irect")]
    #[serde(deserialize_with = "de_irect")]
    pub Rect<i32>,
);

impl Hitbox {
    /// Create a new hitbox centered on the given point
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Hitbox {
        let x = x - w / 2;
        let y = y - h / 2;
        Hitbox(Rect::new(x, x + w, y, y + h))
    }

    pub fn x(&self) -> i32 {
        self.0.x.start
    }
    pub fn y(&self) -> i32 {
        self.0.y.start
    }
    pub fn w(&self) -> i32 {
        self.0.x.end - self.0.x.start
    }
    pub fn h(&self) -> i32 {
        self.0.y.end - self.0.y.start
    }

    pub fn top_left(&self) -> CoordVec {
        CoordVec::new(self.x(), self.y())
    }
    pub fn center(&self) -> CoordVec {
        CoordVec::new(self.x() + self.w() / 2, self.y() + self.h() / 2)
    }

    pub fn shifted_by(&self, dx: i32, dy: i32) -> Hitbox {
        Hitbox::new(self.x() + dx, self.y() + dy, self.w(), self.h())
    }
}

/// Make the rect serializable
#[derive(Debug, Serialize, Deserialize)]
struct ErsatzRect {
    x1: i32,
    x2: i32,
    y1: i32,
    y2: i32,
}
fn ser_irect<S>(rect: &Rect<i32>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ErsatzRect {
        x1: rect.x.start,
        x2: rect.x.end,
        y1: rect.y.start,
        y2: rect.y.end,
    }
    .serialize(ser)
}

fn de_irect<'de, D>(de: D) -> Result<Rect<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let rect = ErsatzRect::deserialize(de)?;
    Ok(Rect::new(rect.x1, rect.x2, rect.y1, rect.y2))
}

/// Broccoli-demanded impl for entity profile.
///
/// We do Broccoli calculations with *floats*, not ints internally. This is so we can do diagonal raycasts.
#[derive(Debug, Clone, Copy)]
pub struct EntityAABB {
    pub e: Entity,
    pub rect: Rect<f64>,
}

impl EntityAABB {
    pub fn new(e: Entity, hb: Hitbox) -> Self {
        Self {
            e,
            rect: hb.0.inner_as(),
        }
    }

    pub fn hb(&self) -> Hitbox {
        let rect = self.rect.inner_as();
        Hitbox(rect)
    }
}

impl Aabb for EntityAABB {
    type Num = f64;

    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

impl ManySwap for EntityAABB {}

/// Convert a [`CoordVec`] to a Broccoli vec2.
pub fn brocfv2(vec: CoordVec) -> BrocVec2<f64> {
    BrocVec2 {
        x: vec.x as _,
        y: vec.y as _,
    }
}
