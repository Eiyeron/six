// Structs

use crate::Assets;
use crate::Scene;
use crate::Transition;
use rand::Rng;
use tetra::graphics;
use tetra::graphics::text::Text;
use tetra::graphics::{Color, DrawParams};
use tetra::math::Vec2;
use tetra::time;
use tetra::Context;

/// One of the two not-so-unique selling points of this battle engine
pub struct RollingMeter {
    pub current_value: u16,
    pub target_value: u16,
    pub max: u16,
    pub accumulator: f32,
    // Speed? Rate?
}

impl RollingMeter {
    fn new(current: u16, max: u16) -> RollingMeter {
        RollingMeter {
            current_value: current,
            max,
            target_value: current,
            accumulator: 0.,
        }
    }

    fn update(&mut self, dt: f32) {
        if self.current_value == self.target_value {
            return;
        }

        self.accumulator += dt;
        // TODO Speed? Rate?
        // TODO While >= 1.
        if self.accumulator >= 1. {
            self.current_value = {
                if self.current_value < self.target_value {
                    self.current_value + 1
                } else {
                    self.current_value - 1
                }
            };
            self.accumulator = self.accumulator.fract();
        }
    }
}

pub struct Stat {
    pub base: u16,
    pub modifier: i16,
}

impl Stat {
    fn new(base: u16) -> Stat {
        Stat { base, modifier: 0 }
    }
    fn multiplied(&self) -> u16 {
        let mult: f32 = get_stat_multiplier(self.modifier) * f32::from(self.base);
        mult as u16
    }

    // Returns the stat difference after buff
    fn buff(&mut self, levels: i16) -> i16 {
        let current_value = self.multiplied() as i16;
        self.modifier = i16::clamp(self.modifier + levels, -3, 3);
        let new_value = self.multiplied() as i16;

        new_value - current_value
    }
}

pub struct Actor {
    pub hp: RollingMeter,
    pub pp: RollingMeter,
    pub offense: Stat,
    pub defense: Stat,
    pub speed: Stat,
    pub iq: Stat,
    // TODO guts
}

impl Actor {
    fn from_stats(
        hp: u16,
        max_hp: u16,
        pp: u16,
        max_pp: u16,
        offense: u16,
        defense: u16,
        speed: u16,
        iq: u16,
    ) -> Actor {
        Actor {
            hp: RollingMeter::new(hp, max_hp),
            pp: RollingMeter::new(pp, max_pp),
            offense: Stat::new(offense),
            defense: Stat::new(defense),
            speed: Stat::new(speed),
            iq: Stat::new(iq),
        }
    }
}

// Logic code decorrelated from structs

fn get_stat_multiplier(modifier: i16) -> f32 {
    match modifier {
        -3 => 0.125,
        -2 => 0.25,
        -1 => 0.5,
        0 => 1.,
        1 => 1.5,
        2 => 1.75,
        3 => 2.,
        _ => panic!("Invalid modifier"),
    }
}

fn damage(offense: u16, attack_level: u16, defense: u16) -> u16 {
    let mut rng = rand::thread_rng();
    let random_multiplier = rng.gen_range(0.75..1.25);
    let base = attack_level * offense - defense;
    ((base as f32) * random_multiplier) as u16
}

// Turn structure idea

// struct TurnAction<'a, 'b> {
//     caster: &'a Actor,
//     targets: Vec<&'b Actor>,
// }

// Engine states?

enum MacroBattleStates {
    CharacterTurnDecision,
    AiTurnDecision,
    TurnAction,
    // Out of the loop
    Intro,
    Win,
    GameOver,
    // Overriding state transitions
    CharacterFalls,
}

enum CharacterTurnSelectionStates {
    Menu,
    BashTargetSelection,
    //
    SpecialMoveSelection(u8), // TODO Class
    SpecialTargetSelection,
    //
    ItemSelection,
    ItemTargetSelection,
    //
}

// Transition? We have either a small stack based FSM or a slightly more developped state machine here.

// Scene?

pub struct BattleScene {
    pub characters: Vec<Actor>,
    pub enemies: Vec<Actor>,
}

impl BattleScene {
    pub fn dummy() -> BattleScene {
        BattleScene {
            characters: vec![
                Actor::from_stats(98, 98, 46, 46, 45, 22, 16, 10),
                Actor::from_stats(115, 115, 0, 0, 35, 27, 12, 21),
                Actor::from_stats(82, 82, 73, 73, 28, 29, 20, 16),
                Actor::from_stats(67, 67, 0, 0, 32, 20, 9, 23),
            ],
            enemies: vec![],
        }
    }

    fn compute_hud_table(title: &str, actors: &[Actor]) -> String {
        let mut actor_summary = {
            if actors.len() > 0 {
                format!("{}\n─────────┬───────\n", title)
            } else {
                format!("{}\n─────────────────\n", title)
            }
        };
        for actor in actors.iter() {
            let actor_line = format!(
                "- {:3}/{:3}|{:3}/{:3}\n",
                actor.hp.current_value, actor.hp.max, actor.pp.current_value, actor.pp.max,
            );
            actor_summary.push_str(&actor_line);
        }
        actor_summary
    }

    fn draw_debug_hud(&self, ctx: &mut Context, assets: &Assets) {
        let character_summary = BattleScene::compute_hud_table("Characters", &self.characters);
        let mut text = Text::new(character_summary, assets.headupdaisy.clone());
        text.draw(ctx, DrawParams::new().position(Vec2::new(16., 16.)));

        let enemy_summary = BattleScene::compute_hud_table("Enemies", &self.enemies);
        let mut text = Text::new(enemy_summary, assets.headupdaisy.clone());
        text.draw(ctx, DrawParams::new().position(Vec2::new(336., 16.)));
    }
}

impl Scene for BattleScene {
    fn update(&mut self, ctx: &mut Context, _assets: &Assets) -> tetra::Result<Transition> {
        let dt = time::get_delta_time(ctx).as_secs_f32();
        for character in self.characters.iter_mut() {
            character.hp.update(dt);
            character.pp.update(dt);
        }
        for enemy in self.enemies.iter_mut() {
            enemy.hp.update(dt);
            enemy.pp.update(dt);
        }
        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> tetra::Result<()> {
        graphics::clear(ctx, Color::BLUE);
        self.draw_debug_hud(ctx, assets);
        Ok(())
    }
}
