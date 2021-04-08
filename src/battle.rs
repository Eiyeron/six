// Structs

mod action;
mod action_decision;
mod stat;
mod turn;
mod turn_preparation;

use crate::battle::action_decision::{AllyActionRecord, CharacterTurnDecisionState};
use crate::battle::stat::{ActorStats, Stat};
use crate::battle::turn::TurnUnrollState;
use crate::battle::turn_preparation::TurnPreparationState;
use crate::meters::{InstantMeter, Meter, RollingMeter};
use crate::Assets;
use crate::Scene;
use crate::Transition;
use std::collections::VecDeque;
use tetra::graphics::text::Text;
use tetra::graphics::{self, Color, DrawParams};
use tetra::math::Vec2;
use tetra::time;
use tetra::Context;

trait CharacterKoSignal {
    fn on_character_ko(&mut self, id: ActorIdentifier);
}

pub struct Actor {
    pub name: String,
    pub hp: Meter,
    pub pp: Meter,
    pub stats: ActorStats,
}

impl Actor {
    // Enemies use instant meters.
    fn enemy_from_stats(
        name: &str,
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
            name: String::from(name),
            hp: Meter::Instant(InstantMeter::new(hp, max_hp)),
            pp: Meter::Instant(InstantMeter::new(pp, max_pp)),
            stats: ActorStats {
                offense: Stat::new(offense),
                defense: Stat::new(defense),
                speed: Stat::new(speed),
                iq: Stat::new(iq),
            },
        }
    }

    // Characters use rolling meters.
    fn character_from_stats(
        name: &str,
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
            name: String::from(name),
            hp: Meter::Rolling(RollingMeter::new(hp, max_hp)),
            pp: Meter::Rolling(RollingMeter::new(pp, max_pp)),
            stats: ActorStats {
                offense: Stat::new(offense),
                defense: Stat::new(defense),
                speed: Stat::new(speed),
                iq: Stat::new(iq),
            },
        }
    }
    fn update_meters(&mut self, dt: f32) {
        if let Meter::Rolling(meter) = &mut self.hp {
            meter.update(dt)
        }
        if let Meter::Rolling(meter) = &mut self.pp {
            meter.update(dt)
        }
    }
}

// Turn structure idea

enum UIAction {
    Up,
    Down,
    Left,
    Right,
    PagePrev, // For paginated ui
    PageNext,
    Validate,
    Cancel, // Also works as back
}

trait Drawable {
    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> tetra::Result<()>;
}

enum MacroBattleStates {
    CharacterTurnDecision(CharacterTurnDecisionState),
    TurnPreparation(TurnPreparationState),
    TurnUnroll(TurnUnrollState),
    // Out of the loop
    Intro,
    Win,
    GameOver,
    // Overriding state transitions
    CharacterFalls,
}

// TODO Replace with Action instead?
pub enum ActionType {
    Bash(Target),
    Psi,
    Item,
    Guard,
    // ???
}

// Epiphany : the action could determine itself the target instead of hard-coding it.
// Even better : the AI would decice when it's their turn
// Update : I hate myself for inflicting this upon my poor soul.

// Scene?

#[derive(Clone)]
pub enum Team {
    Ally,
    Enemy,
}

type ActorIdentifier = (Team, usize);

#[derive(Clone)]
// TODO ?
pub enum Target {
    Single(ActorIdentifier),
    WholeTeam(Team),
}

pub struct TurnAction {
    id_in_team: usize,
    team: Team,
    speed: u16,
}

pub struct BattleScene {
    pub characters: Vec<Actor>,
    pub enemies: Vec<Actor>,
    // Test
    allies_actions: Vec<AllyActionRecord>,
    turn_order: VecDeque<TurnAction>,
    // Stack?
    state: MacroBattleStates,
}

impl BattleScene {
    pub fn dummy() -> BattleScene {
        let characters = vec![
            Actor::character_from_stats("One", 98, 98, 46, 46, 45, 22, 16, 10),
            Actor::character_from_stats("Two", 115, 115, 0, 0, 35, 27, 12, 21),
            Actor::character_from_stats("Three", 82, 82, 73, 73, 28, 29, 20, 16),
            Actor::character_from_stats("Four", 67, 67, 0, 0, 32, 20, 9, 23),
        ];
        BattleScene {
            enemies: vec![
                Actor::enemy_from_stats("Robot", 53, 53, 0, 0, 35, 10, 17, 8),
                Actor::enemy_from_stats("Robot", 53, 53, 0, 0, 35, 10, 17, 8),
            ],
            allies_actions: vec![],
            turn_order: VecDeque::new(),
            state: MacroBattleStates::CharacterTurnDecision(
                CharacterTurnDecisionState::new_turn(&characters).unwrap(),
            ),
            characters,
        }
    }

    fn compute_hud_table(title: &str, actors: &[Actor]) -> String {
        let mut actor_summary = {
            if actors.is_empty() {
                format!("{}\n────────────────\n", title)
            } else {
                format!("{}\n────────┬───────\n", title)
            }
        };
        for actor in actors.iter() {
            let (hp, max_hp) = actor.hp.current_and_max();
            let (pp, max_pp) = actor.pp.current_and_max();
            let actor_line = format!(
                "{:8}|\n {:3}/{:3}|{:3}/{:3}\n",
                actor.name, hp, max_hp, pp, max_pp,
            );
            actor_summary.push_str(&actor_line);
        }
        actor_summary
    }

    fn draw_debug_hud(&self, ctx: &mut Context, assets: &Assets) {
        let character_summary = BattleScene::compute_hud_table("Characters", &self.characters);
        let mut text = Text::new(character_summary, assets.headupdaisy.clone());
        text.draw(
            ctx,
            DrawParams::new()
                .color(Color::rgb8(0xeb, 0xdb, 0xb2))
                .position(Vec2::new(16., 16.)),
        );

        let enemy_summary = BattleScene::compute_hud_table("Enemies", &self.enemies);
        let mut text = Text::new(enemy_summary, assets.headupdaisy.clone());
        text.draw(
            ctx,
            DrawParams::new()
                .color(Color::rgb8(0xeb, 0xdb, 0xb2))
                .position(Vec2::new(336., 16.)),
        );
    }

    pub fn all_ko(arr: &[Actor]) -> bool {
        arr.iter().all(|e| e.hp.current_and_max().0 == 0)
    }

    pub fn end_of_fight(&self) -> bool {
        BattleScene::all_ko(&self.enemies) || BattleScene::all_ko(&self.characters)
    }
    pub fn get_end_state(&self) -> Option<MacroBattleStates> {
        if BattleScene::all_ko(&self.enemies) {
            return Some(MacroBattleStates::Win);
        } else if BattleScene::all_ko(&self.characters) {
            return Some(MacroBattleStates::GameOver);
        }

        None
    }
}

impl Scene for BattleScene {
    fn update(&mut self, ctx: &mut Context, _assets: &Assets) -> tetra::Result<Transition> {
        let dt = time::get_delta_time(ctx).as_secs_f32();
        let update_meters: bool = !self.end_of_fight();
        if update_meters {
            for enemy in self.enemies.iter_mut() {
                enemy.update_meters(dt);
            }
        }
        for (id, character) in self.characters.iter_mut().enumerate() {
            let (previous_hp, _) = character.hp.current_and_max();
            if update_meters {
                character.update_meters(dt);
                if previous_hp > 0 && character.hp.current_and_max().0 == 0 {
                    println!("{} is K.O.", character.name);

                    // TODO finish death signaling
                    match &mut self.state {
                        MacroBattleStates::CharacterTurnDecision(decision_state) => {
                            decision_state.on_character_ko((Team::Ally, id));
                        }
                        MacroBattleStates::TurnPreparation(_) => {} // Shouldn't be necessary
                        MacroBattleStates::TurnUnroll(_) => {
                            // TODO signal ko during unroll
                        }
                        _ => (),
                    }
                }
            }
        }

        match &self.state {
            MacroBattleStates::CharacterTurnDecision(_) => {
                CharacterTurnDecisionState::update(self, ctx)
            }
            MacroBattleStates::TurnPreparation(_) => TurnPreparationState::update(self, ctx),
            MacroBattleStates::TurnUnroll(_) => TurnUnrollState::update(self, ctx),
            _ => (),
        }

        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> tetra::Result<()> {
        graphics::clear(ctx, Color::rgb8(0x28, 0x28, 0x28));
        self.draw_debug_hud(ctx, assets);

        match &self.state {
            MacroBattleStates::CharacterTurnDecision(_) => {
                CharacterTurnDecisionState::draw(&self, ctx, assets)
            }
            MacroBattleStates::TurnPreparation(_) => TurnPreparationState::draw(&self, ctx, assets),
            MacroBattleStates::TurnUnroll(_) => TurnUnrollState::draw(&self, ctx, assets),
            MacroBattleStates::Win => {
                let mut debug_text = Text::new("--Victory!--\n", assets.headupdaisy.clone());
                debug_text.draw(
                    ctx,
                    DrawParams::new()
                        .color(Color::rgb8(0xeb, 0xdb, 0xb2))
                        .position(Vec2::new(16., 360.)),
                );
            }
            MacroBattleStates::GameOver => {
                let mut debug_text = Text::new("--Game over!--\n", assets.headupdaisy.clone());
                debug_text.draw(
                    ctx,
                    DrawParams::new()
                        .color(Color::rgb8(0xeb, 0xdb, 0xb2))
                        .position(Vec2::new(16., 360.)),
                );
            }
            _ => (),
        }
        Ok(())
    }
}
