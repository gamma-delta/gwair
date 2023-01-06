use std::cmp;

use aglet::CoordVec;
use macroquad::prelude as mq;
use palkia::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;

use crate::{
    ecm::{message::MsgDraw, resource::Camera},
    resources::Resources,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColoredHitbox {
    #[serde(serialize_with = "ser_hexcol")]
    #[serde(deserialize_with = "de_hexcol")]
    color: mq::Color,
}

impl ColoredHitbox {
    pub fn new(color: mq::Color) -> Self {
        Self { color }
    }
}

impl Component for ColoredHitbox {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_read(|this, msg: MsgDraw, _, access| {
            if let (Some(pos), Some((w, h))) = (msg.pos, msg.dims) {
                let cam = access.read_resource::<Camera>().unwrap();

                let corner = pos - CoordVec::new(w / 2, h / 2) - cam.center;
                mq::draw_rectangle(
                    corner.x as f32,
                    corner.y as f32,
                    w as f32,
                    h as f32,
                    this.color,
                );
            }

            msg
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DrawTexture {
    tex: SmolStr,
}

impl Component for DrawTexture {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder.handle_read(|this, msg: MsgDraw, _, access| {
            if let (Some(pos), Some((w, h))) = (msg.pos, msg.dims) {
                let cam = access.read_resource::<Camera>().unwrap();

                let corner = pos - CoordVec::new(w / 2, h / 2) - cam.center;

                let assets = Resources::get();
                let tex = assets.get_texture(&this.tex);
                mq::draw_texture(
                    tex,
                    corner.x as f32,
                    corner.y as f32,
                    mq::WHITE,
                );
            }

            msg
        })
    }
}

/// Data component determining what gets rendered on top. Higher = more on top.
/// `None` gets rendered under everything.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZLevel {
    pub level: u32,
}

impl ZLevel {
    pub fn sort(a: Option<u32>, b: Option<u32>) -> cmp::Ordering {
        use cmp::Ordering;
        match (a, b) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(&b),
        }
    }
}

impl Component for ZLevel {
    fn register_handlers(builder: HandlerBuilder<Self>) -> HandlerBuilder<Self>
    where
        Self: Sized,
    {
        builder
    }
}

fn ser_hexcol<S>(col: &mq::Color, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let code = (((col.r * 255.0) as u32) << 24)
        | (((col.g * 255.0) as u32) << 16)
        | (((col.b * 255.0) as u32) << 8)
        | ((col.a * 255.0) as u32);
    ser.serialize_u32(code)
}

fn de_hexcol<'de, D>(de: D) -> Result<mq::Color, D::Error>
where
    D: Deserializer<'de>,
{
    let col: u32 = Deserialize::deserialize(de)?;
    let r = (col & 0xff000000) >> 24;
    let g = (col & 0xff0000) >> 16;
    let b = (col & 0xff00) >> 8;
    let a = col & 0xff;
    Ok(mq::Color::from_rgba(r as u8, g as u8, b as u8, a as u8))
}
