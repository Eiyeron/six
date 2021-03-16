use crate::Assets;
use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Color};
use tetra::window;
use tetra::{self, Context, State};

// Allow for scene control from a Scene to the GameState
pub enum Transition {
    None,
    Push(Box<dyn Scene>),
    Switch(Box<dyn Scene>), // Pop + push
    Pop,
}

pub trait Scene {
    fn update(&mut self, ctx: &mut Context, assets: &Assets) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> tetra::Result<()>;
}

pub struct GameState {
    scenes: Vec<Box<dyn Scene>>,
    scaler: ScreenScaler,
    assets: Assets,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let assets = Assets::load(ctx);
        Ok(GameState {
            scenes: vec![Box::new(crate::TestScene::new())],
            scaler: ScreenScaler::with_window_size(
                ctx,
                640,
                480,
                ScalingMode::ShowAllPixelPerfect,
            )?,
            assets,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx, &self.assets)? {
                Transition::None => {}
                Transition::Push(s) => {
                    self.scenes.push(s);
                }
                Transition::Switch(s) => {
                    self.scenes.pop();
                    self.scenes.push(s);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
            },
            None => window::quit(ctx),
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());

        match self.scenes.last_mut() {
            Some(active_scene) => active_scene.draw(ctx, &self.assets)?,
            None => (),
        }

        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);

        self.scaler.draw(ctx);

        Ok(())
    }
}
