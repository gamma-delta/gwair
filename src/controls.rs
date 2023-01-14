use aglet::CoordVec;
use glam::Vec2;
use macroquad::prelude::*;

use crate::gfx::{width_height_deficit, GAME_HEIGHT, GAME_WIDTH};

#[derive(Debug, Clone, Copy)]
pub struct ControlState {
    pub movement: Vec2,
    pub pointing: Vec2,
    pub jump: bool,
    pub swing: bool,

    pub reset: bool,
}

impl ControlState {
    pub fn calculate(player_pos: CoordVec, cam_pos: CoordVec) -> Self {
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

        let screen_space_player_pos = player_pos - cam_pos
            + CoordVec::new(GAME_WIDTH as i32 / 2, GAME_HEIGHT as i32 / 2);
        let mpp = mouse_position_pixel();
        let delta = CoordVec::new(mpp.0.round() as _, mpp.1.round() as _)
            - screen_space_player_pos;
        let pointing = vec2(delta.x as _, delta.y as _).normalize_or_zero();

        let jump =
            is_key_down(KeyCode::Space) || is_key_down(KeyCode::RightBracket);
        let swing = is_mouse_button_down(MouseButton::Left);

        let reset = is_key_down(KeyCode::R);

        Self {
            movement,
            pointing,
            jump,
            swing,
            reset,
        }
    }
}

pub fn mouse_position_pixel() -> (f32, f32) {
    let (mx, my) = mouse_position();
    let (wd, hd) = width_height_deficit();
    let mx = (mx - wd / 2.0) / ((screen_width() - wd) / GAME_WIDTH);
    let my = (my - hd / 2.0) / ((screen_height() - hd) / GAME_HEIGHT);
    (mx, my)
}
