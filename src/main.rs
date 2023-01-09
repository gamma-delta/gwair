use dialga::EntityFabricator;
use fabctx::FabCtx;
use gfx::{width_height_deficit, GAME_HEIGHT, GAME_WIDTH};
use macroquad::prelude::*;
use resources::Resources;
use states::StateGameplay;

mod controls;
mod ecm;
mod fabctx;
mod geom;
mod gfx;
mod resources;
mod states;

fn conf() -> Conf {
    Conf {
        window_title: String::from("Gymnast With An Immovable Rod"),
        window_width: GAME_WIDTH as i32 * 2,
        window_height: GAME_HEIGHT as i32 * 2,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let resources = Resources::load().unwrap();
    Resources::swap(resources);

    let canvas = render_target(GAME_WIDTH as u32, GAME_HEIGHT as u32);
    canvas.texture.set_filter(FilterMode::Nearest);
    let mut app = App {
        canvas,
        state: StateGameplay::new(),
    };

    loop {
        app.update();
        app.draw();

        next_frame().await
    }
}

struct App {
    canvas: RenderTarget,

    state: StateGameplay,
}
impl App {
    fn update(&mut self) {
        self.state.on_update();
    }

    fn draw(&self) {
        push_camera_state();
        set_camera(&Camera2D {
            render_target: Some(self.canvas),
            zoom: vec2(
                (GAME_WIDTH as f32).recip() * 2.0,
                (GAME_HEIGHT as f32).recip() * 2.0,
            ),
            // target: vec2(GAME_WIDTH as f32 / 2.0, GAME_HEIGHT as f32 / 2.0),
            target: vec2(0.0, 0.0),
            ..Default::default()
        });

        clear_background(WHITE);
        self.state.on_draw();

        // Done rendering to the canvas; go back to our normal camera
        // to size the canvas
        pop_camera_state();

        clear_background(BLACK);

        // Figure out the drawbox.
        // these are how much wider/taller the window is than the content
        let (width_deficit, height_deficit) = width_height_deficit();
        draw_texture_ex(
            self.canvas.texture,
            width_deficit / 2.0,
            height_deficit / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    screen_width() - width_deficit,
                    screen_height() - height_deficit,
                )),
                ..Default::default()
            },
        );
    }
}

type EntityFab = EntityFabricator<FabCtx>;
