use crate::battle::action_decision::CharacterTurnDecisionState;
use crate::battle::ActionType;
use crate::battle::MacroBattleStates;
use crate::battle::Team;
use crate::battle::TurnAction;
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
enum SubStateTransition {
    None,
    NextSubState(TurnSubState),
    EndOfTurn,
}

struct Announce {
    time: f32,
}
impl Announce {
    pub fn new() -> Announce {
        Announce { time: 0. }
    }

    pub fn update(&mut self, ctx: &Context) -> SubStateTransition {
        self.time += tetra::time::get_delta_time(ctx).as_secs_f32();
        if self.time >= 1. {
            println!("Going to Act!");
            return SubStateTransition::NextSubState(TurnSubState::DoIt(DoIt::new()));
        }
        SubStateTransition::None
    }
}

struct DoIt {
    time: f32,
}

impl DoIt {
    pub fn new() -> DoIt {
        DoIt { time: 0. }
    }

    pub fn update(&mut self, ctx: &Context) -> SubStateTransition {
        self.time += tetra::time::get_delta_time(ctx).as_secs_f32();
        if self.time >= 1. {
            println!("Pow!");
            return SubStateTransition::NextSubState(TurnSubState::NextAction);
        }
        SubStateTransition::None
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

    // TODO ?
    fn process_ally_action(scene: &BattleScene, action: TurnAction) -> SubStateTransition {
        let ally = &scene.characters[action.id_in_team];
        if ally.hp.current_and_max().0 == 0 {
            println!("Skipping action because K.O.");
            return SubStateTransition::NextSubState(TurnSubState::NextAction);
        }
        let action = match scene
            .allies_actions
            .iter()
            .find(|rec| rec.id_in_team == action.id_in_team)
        {
            Some(a) => a,
            _ => unreachable!("[ERROR] An action record should always involve a character."),
        };
        let action_str = match action.action_type {
            ActionType::Bash => "Bash",
            ActionType::Psi => "PSI",
            ActionType::Item => "Item",
            ActionType::Guard => "Guard",
        };
        println!(
            "→ {} ({}) will act ({})",
            ally.name, action.id_in_team, action_str
        );
        SubStateTransition::NextSubState(TurnSubState::Announce(Announce::new()))
    }

    fn end_of_turn(scene: &mut BattleScene) -> SubStateTransition {
        println!("- End of Turn -");
        scene.allies_actions.clear();

        SubStateTransition::EndOfTurn
    }

    fn next_action(scene: &mut BattleScene) -> SubStateTransition {
        let next_action = scene.turn_order.pop_front().unwrap();
        match next_action.team {
            Team::Ally => TurnUnrollState::process_ally_action(scene, next_action),
            // TODO Enemy AI decision
            Team::Enemy => {
                let enemy = &scene.enemies[next_action.id_in_team];
                println!("→ {} ({}) will do", enemy.name, next_action.id_in_team);
                SubStateTransition::NextSubState(TurnSubState::Announce(Announce::new()))
            }
        }
    }

    pub fn update(scene: &mut BattleScene, ctx: &Context) {
        if let MacroBattleStates::TurnUnroll(state) = &mut scene.state {
            let transition = match &mut state.sub_state {
                TurnSubState::NextAction => {
                    if scene.turn_order.is_empty() {
                        TurnUnrollState::end_of_turn(scene)
                    } else {
                        TurnUnrollState::next_action(scene)
                    }
                }
                // TODO Some animations and stuff?
                TurnSubState::Announce(announce) => announce.update(ctx),
                // TODO Pass around the action data
                // TODO determine what the AI should do in their turn
                // TODO Apply damage
                TurnSubState::DoIt(do_it) => do_it.update(ctx),
            };

            match transition {
                SubStateTransition::EndOfTurn => {
                    scene.state = MacroBattleStates::CharacterTurnDecision(
                        match CharacterTurnDecisionState::new_turn(&scene.characters) {
                            Some(a) => a,
                            _ => unreachable!(
                                "Tried to transition into a new turn with all characters K.O."
                            ),
                        },
                    );
                }
                SubStateTransition::NextSubState(sub_state) => {
                    scene.state = MacroBattleStates::TurnUnroll(TurnUnrollState { sub_state });
                }
                SubStateTransition::None => (),
            };
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
