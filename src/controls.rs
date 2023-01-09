use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct ControlState {
    pub movement: Vec2,
    pub jump: bool,
    pub reset: bool,
}

impl ControlState {
    pub fn calculate() -> Self {
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
        let reset = is_key_down(KeyCode::R);

        Self {
            movement,
            jump,
            reset,
        }
    }
}
