use std::sync::Mutex;

use gilrs::{GamepadId, Gilrs, GilrsBuilder};
use glam::{vec2, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct ControlState {
  pub movement: Vec2,
  pub jump: bool,
  pub swing: bool,

  pub reset: bool,
}

impl ControlState {
  pub fn calculate() -> Self {
    let base = Self::calculate_from_kb();

    let mut lock = THE_GILRS.lock().unwrap();
    let gilrs = lock.as_mut().unwrap();
    if let Some(gamepad_state) = gilrs.calculate_controls() {
      return base.merge(&gamepad_state);
    }

    base
  }

  fn calculate_from_kb() -> Self {
    use macroquad::prelude::*;

    let mut dv = Vec2::ZERO;
    if is_key_down(KeyCode::W) {
      dv.y -= 1.0;
    }
    if is_key_down(KeyCode::S) {
      dv.y += 1.0;
    }
    if is_key_down(KeyCode::A) {
      dv.x -= 1.0;
    }
    if is_key_down(KeyCode::D) {
      dv.x += 1.0;
    }
    let movement = dv.normalize_or_zero();

    let jump =
      is_key_down(KeyCode::Space) || is_key_down(KeyCode::RightBracket);
    let swing = is_key_down(KeyCode::J);

    let reset = is_key_down(KeyCode::R);

    Self {
      movement,
      jump,
      swing,
      reset,
    }
  }

  pub fn merge(&self, other: &Self) -> Self {
    let movement = (self.movement + other.movement).normalize_or_zero();
    let jump = self.jump || other.jump;
    let swing = self.swing || other.swing;
    let reset = self.reset || other.reset;

    ControlState {
      movement,
      jump,
      swing,
      reset,
    }
  }
}

/// Yes it's a global womp womp fight me this is the kind of thing that wants
/// to be global.
pub struct GilrsState {
  gilrs: Gilrs,
  gamepad_id: Option<GamepadId>,
}

impl GilrsState {
  pub const DEADZONE: f32 = 0.15;
  pub const TRIGGER_DEPTH: f32 = 0.3;

  pub fn init() {
    let builder = GilrsBuilder::new().set_update_state(false);
    let gilrs = match builder.build() {
      Ok(it) => it,
      Err(gilrs::Error::NotImplemented(dummy)) => {
        eprintln!("gilrs is not supported, using dummy impl");
        dummy
      }
      Err(ono) => panic!("{}", ono),
    };

    let gamepads = gilrs
      .gamepads()
      .inspect(|(id, gp)| {
        println!("{:?}: {:?}", id, gp.name());
      })
      .collect::<Vec<_>>();
    let id = gamepads.get(0).map(|(id, gp)| {
      println!("using gamepad {:?}", id);

      println!("rt: {:?}", gp.button_code(gilrs::Button::RightTrigger2));

      *id
    });

    let state = GilrsState {
      gilrs,
      gamepad_id: id,
    };

    let mut lock = THE_GILRS.lock().unwrap();
    *lock = Some(state);
  }

  fn calculate_controls(&mut self) -> Option<ControlState> {
    use gilrs::{Axis, Button};

    while let Some(ev) = self.gilrs.next_event() {
      self.gilrs.update(&ev);
    }

    let id = self.gamepad_id?;
    let gp = self.gilrs.connected_gamepad(id)?;

    let dx = gp.value(Axis::LeftStickX);
    let dy = -gp.value(Axis::LeftStickY);
    let dx = if dx * dx < Self::DEADZONE * Self::DEADZONE {
      0.0
    } else {
      dx
    };
    let dy = if dy * dy < Self::DEADZONE * Self::DEADZONE {
      0.0
    } else {
      dy
    };
    let movement = vec2(dx, dy).normalize_or_zero();

    let jump = gp.is_pressed(Button::South);
    let swing = gp.value(Axis::RightZ) >= Self::TRIGGER_DEPTH;

    let reset = gp.is_pressed(Button::Start);

    Some(ControlState {
      movement,
      jump,
      swing,
      reset,
    })
  }
}

static THE_GILRS: Mutex<Option<GilrsState>> = Mutex::new(None);
