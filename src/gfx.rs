use macroquad::prelude as mq;

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
