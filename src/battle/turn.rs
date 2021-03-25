use crate::battle::action_decision::CharacterTurnDecisionState;
use crate::battle::MacroBattleStates;
use crate::battle::Team;
use crate::Assets;
use crate::BattleScene;
use tetra::graphics::text::Text;
use tetra::graphics::DrawParams;
use tetra::math::Vec2;
use tetra::Context;

// TODO Maybe merge with TurnSubState
pub struct TurnUnrollState {
    sub_state: TurnSubState,
}

struct Announce {
    time: f32,
}
impl Announce {
    pub fn new() -> Announce {
        Announce { time: 0. }
    }
}

struct DoIt {
    time: f32,
}

impl DoIt {
    pub fn new() -> DoIt {
        DoIt { time: 0. }
    }
}

enum TurnSubState {
    // Also handles the AI decision
    NextAction,
    // The blink + sound jingle
    Announce(Announce),
    //
    DoIt(DoIt),
}

impl TurnUnrollState {
    pub fn new() -> TurnUnrollState {
        println!("- New Turn -");
        TurnUnrollState {
            sub_state: TurnSubState::NextAction,
        }
    }
    pub fn update(scene: &mut BattleScene, ctx: &Context) {
        if scene.turn_order.is_empty() {
            println!("- End of Turn -");
            scene.state = MacroBattleStates::CharacterTurnDecision(
                CharacterTurnDecisionState::new_turn(&scene.characters).unwrap(),
            );
            return;
        }
        if let MacroBattleStates::TurnUnroll(state) = &mut scene.state {
            match &mut state.sub_state {
                TurnSubState::NextAction => {
                    println!("Next action");
                    let next_action = scene.turn_order.pop_front().unwrap();
                    match next_action.team {
                        Team::Ally => (),
                        // TODO Enemy AI decision
                        Team::Enemy => (),
                    }
                    scene.state = MacroBattleStates::TurnUnroll(TurnUnrollState {
                        sub_state: TurnSubState::Announce(Announce::new()),
                    });
                }
                // TODO Some animations and stuff?
                TurnSubState::Announce(announce) => {
                    announce.time += tetra::time::get_delta_time(ctx).as_secs_f32();
                    if announce.time >= 1. {
                        println!("Going to Act!");
                        scene.state = MacroBattleStates::TurnUnroll(TurnUnrollState {
                            sub_state: TurnSubState::DoIt(DoIt::new()),
                        });
                    }
                }
                // TODO Pass around the action data
                // TODO determine what the AI should do in their turn
                // TODO Apply damage
                TurnSubState::DoIt(do_it) => {
                    do_it.time += tetra::time::get_delta_time(ctx).as_secs_f32();
                    if do_it.time >= 1. {
                        println!("Pow!");
                        scene.state = MacroBattleStates::TurnUnroll(TurnUnrollState {
                            sub_state: TurnSubState::NextAction,
                        });
                    }
                }
            }
        }
    }
    pub fn draw(scene: &BattleScene, ctx: &mut Context, assets: &Assets) {
        let mut debug_text = Text::new("--Turn--\n", assets.headupdaisy.clone());
        if let MacroBattleStates::TurnUnroll(state) = &scene.state {
            match state.sub_state {
                TurnSubState::NextAction => debug_text.push_str("Next action"),
                TurnSubState::Announce(_) => debug_text.push_str("Announcing attack"),
                TurnSubState::DoIt(_) => debug_text.push_str("Action happens"),
            }
        }
        debug_text.draw(ctx, DrawParams::new().position(Vec2::new(16., 360.)));
    }
}
