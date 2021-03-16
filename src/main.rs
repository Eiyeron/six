mod puppet;

use puppet::{GameState, Scene, Transition};
use tetra::graphics::{self, Color, DrawParams, Rectangle, Texture};
use tetra::math::Vec2;
use tetra::time::{self, Timestep};
use tetra::{Context, ContextBuilder};

//-- Game handling structures --

pub struct Assets {
    pub robot: Texture,
    pub blowharder: Texture,
    pub white: Texture,
}

impl Assets {
    fn load(ctx: &mut Context) -> Assets {
        Assets {
            robot: Texture::new(ctx, "res/textures/robot.png").unwrap(),
            blowharder: Texture::new(ctx, "res/textures/blowharder.png").unwrap(),
            white: Texture::from_rgba(ctx, 1, 1, &[0xff, 0xff, 0xff, 0xff]).unwrap(),
        }
    }
}

//-- Some scenes --

pub struct TestScene {
    color: Color,
    time: f32,
}

impl TestScene {
    pub fn new() -> TestScene {
        TestScene {
            color: Color::rgb(0.2, 0.4, 0.6),
            time: 0.0,
        }
    }
}
fn cell_from_index(
    id: usize,
    cell_width: usize,
    cell_height: usize,
    width: usize,
    height: usize,
) -> Rectangle {
    let cx = width / cell_width;
    let cy = height / cell_height;
    let x = (id % cx) * cell_width;
    let y = (id / cx) * cell_height;
    Rectangle {
        x: x as f32,
        y: y as f32,
        width: cell_width as f32,
        height: cell_height as f32,
    }
}

const FRAME_ANIMATION: [usize; 6] = [0, 1, 2, 2, 1, 0];

impl Scene for TestScene {
    fn update(&mut self, ctx: &mut Context, _assets: &Assets) -> tetra::Result<Transition> {
        self.time += time::get_delta_time(ctx).as_secs_f32();
        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> tetra::Result<()> {
        graphics::clear(ctx, self.color);
        for x in 0..16 {
            for y in 0..16 {
                let x = x as f32 * 16.;
                let y = y as f32 * 16.;
                let position = Vec2::new(x, y);
                let frame = cell_from_index(1, 16, 16, 1264, 512);
                assets
                    .blowharder
                    .draw_region(ctx, frame, DrawParams::new().position(position));
            }
        }
        let frame_index = FRAME_ANIMATION[(self.time * 4.0) as usize % FRAME_ANIMATION.len()];
        let frame = cell_from_index(frame_index, 64, 64, 256, 256);
        let x = 320. - 32.;
        let y = 240. + f32::sin(self.time) * 8. - 32.;
        assets
            .robot
            .draw_region(ctx, frame, DrawParams::new().position(Vec2::new(x, y)));
        Ok(())
    }
}

//-- Entry point and loop --

fn main() -> tetra::Result {
    println!("Hello, world!");

    ContextBuilder::new("Hello World!", 640 * 2, 480 * 2)
        .timestep(Timestep::Fixed(60.0))
        .build()?
        .run(GameState::new)
}
