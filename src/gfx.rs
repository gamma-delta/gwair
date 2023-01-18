use macroquad::prelude as mq;
use serde::{Deserialize, Deserializer, Serializer};

pub const GAME_WIDTH: f32 = 320.0;
pub const GAME_HEIGHT: f32 = 180.0;
pub const ASPECT_RATIO: f32 = GAME_WIDTH / GAME_HEIGHT;

pub fn width_height_deficit() -> (f32, f32) {
  if (mq::screen_width() / mq::screen_height()) > ASPECT_RATIO {
    // it's too wide! put bars on the sides!
    // the height becomes the authority on how wide to draw
    let expected_width = mq::screen_height() * ASPECT_RATIO;
    (mq::screen_width() - expected_width, 0.0f32)
  } else {
    // it's too tall! put bars on the ends!
    // the width is the authority
    let expected_height = mq::screen_width() / ASPECT_RATIO;
    (0.0f32, mq::screen_height() - expected_height)
  }
}

pub fn hexcol(code: u32) -> mq::Color {
  let r = (code & 0xff000000) >> 24;
  let g = (code & 0xff0000) >> 16;
  let b = (code & 0xff00) >> 8;
  let a = code & 0xff;
  mq::Color::from_rgba(r as u8, g as u8, b as u8, a as u8)
}
pub fn ser_hexcol<S>(col: &mq::Color, ser: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let code = (((col.r * 255.0) as u32) << 24)
    | (((col.g * 255.0) as u32) << 16)
    | (((col.b * 255.0) as u32) << 8)
    | ((col.a * 255.0) as u32);
  ser.serialize_u32(code)
}
pub fn de_hexcol<'de, D>(de: D) -> Result<mq::Color, D::Error>
where
  D: Deserializer<'de>,
{
  let col: u32 = Deserialize::deserialize(de)?;
  Ok(hexcol(col))
}
