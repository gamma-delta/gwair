use std::cmp;

use aglet::CoordVec;
use macroquad::prelude as mq;
use palkia::prelude::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{
  ecm::{message::MsgDraw, resource::Camera},
  gfx::{de_hexcol, ser_hexcol},
  resources::Resources,
};

use super::{HasDims, Positioned};

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
    builder.handle_read(|this, msg: MsgDraw, me, access| {
      let pos = access.query::<&Positioned>(me).unwrap();
      let dims = access.query::<&HasDims>(me).unwrap();
      let cam = access.read_resource::<Camera>().unwrap();

      let corner =
        pos.pos - CoordVec::new(dims.w / 2, dims.h / 2) - cam.center();
      mq::draw_rectangle(
        corner.x as f32,
        corner.y as f32,
        dims.w as f32,
        dims.h as f32,
        this.color,
      );

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
    builder.handle_read(|this, msg: MsgDraw, me, access| {
      let pos = access.query::<&Positioned>(me).unwrap();
      let dims = access.query::<&HasDims>(me).unwrap();
      let cam = access.read_resource::<Camera>().unwrap();

      let corner =
        pos.pos - CoordVec::new(dims.w / 2, dims.h / 2) - cam.center();
      let assets = Resources::get();
      let tex = assets.get_texture(&this.tex);
      mq::draw_texture(tex, corner.x as f32, corner.y as f32, mq::WHITE);

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
