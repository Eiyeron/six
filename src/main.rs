mod battle;
mod meters;
mod puppet;

use crate::battle::BattleScene;
use puppet::{GameState, Scene, Transition};
use tetra::graphics::text::Font;
use tetra::graphics::Texture;
use tetra::time::Timestep;
use tetra::{Context, ContextBuilder};

//-- Game handling structures --

pub struct Assets {
    pub robot: Texture,
    pub blowharder: Texture,
    pub white: Texture,

    pub gridgazer: Font,
    pub headupdaisy: Font,
}

impl Assets {
    fn load(ctx: &mut Context) -> Assets {
        Assets {
            robot: Texture::new(ctx, "res/textures/robot.png").unwrap(),
            blowharder: Texture::new(ctx, "res/textures/blowharder.png").unwrap(),
            white: Texture::from_rgba(ctx, 1, 1, &[0xff, 0xff, 0xff, 0xff]).unwrap(),

            gridgazer: Font::vector(ctx, "res/fonts/x16y32pxGridGazer.ttf", 32.).unwrap(),
            headupdaisy: Font::vector(ctx, "res/fonts/x14y24pxHeadUpDaisy.ttf", 24.).unwrap(),
        }
    }
}

//-- Entry point and loop --

fn main() -> tetra::Result {
    println!("Hello, world!");
    ContextBuilder::new("Hello World!", 640 * 2, 480 * 2)
        .timestep(Timestep::Fixed(60.0))
        .build()?
        .run(|ctx| GameState::new(ctx, BattleScene::dummy()))
}
