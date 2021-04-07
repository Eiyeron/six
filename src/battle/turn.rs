use std::rc::Rc;

use crate::battle::action_decision::CharacterTurnDecisionState;
use crate::battle::ActionType;
use crate::battle::Actor;
use crate::battle::ActorIdentifier;
use crate::battle::MacroBattleStates;
use crate::battle::Team;
use crate::battle::TurnAction;
use crate::Assets;
use crate::BattleScene;
use tetra::graphics::text::Text;
use tetra::graphics::DrawParams;
use tetra::math::Vec2;
use tetra::Context;

use super::{damage, ActionTarget};

pub trait Action {
    fn go(&mut self, caster: &Actor, targets: &mut [Actor], ctx: &Context) -> SubStateTransition;
}

// TODO Maybe merge with TurnSubState
pub struct TurnUnrollState {
    sub_state: TurnSubState,
}
pub enum SubStateTransition {
    None,
    NextSubState(TurnSubState),
    EndOfTurn,
}

// There's probably a better solution than a shared pointer to store the Action.
struct Announce {
    time: f32,
    announced_action: Rc<dyn Action>,
    caster: ActorIdentifier,
    // TODO Keep track of target
    target: ActionTarget,
}
impl Announce {
    pub fn new(
        announced_action: Rc<dyn Action>,
        caster: ActorIdentifier,
        target: ActionTarget,
    ) -> Announce {
        Announce {
            time: 0.,
            announced_action,
            caster,
            target,
        }
    }
    pub fn update(&mut self, ctx: &Context) -> SubStateTransition {
        self.time += tetra::time::get_delta_time(ctx).as_secs_f32();
        if self.time >= 1. {
            println!("Going to Act!");
            return SubStateTransition::NextSubState(TurnSubState::DoIt(DoIt::new(
                self.announced_action.clone(),
                self.caster.clone(),
                // TODO Keep track of target
                self.target.clone(),
            )));
        }
        SubStateTransition::None
    }
}

struct Bash {
    time: f32,
}

impl Bash {
    fn new() -> Bash {
        Bash { time: 0. }
    }
}

impl Action for Bash {
    fn go(&mut self, caster: &Actor, targets: &mut [Actor], ctx: &Context) -> SubStateTransition {
        self.time += tetra::time::get_delta_time(ctx).as_secs_f32();
        if self.time < 1. {
            return SubStateTransition::None;
        }
        let caster_offense = caster.offense.multiplied();

        for target in targets.iter_mut() {
            let dmg = damage(caster_offense, 1, target.defense.multiplied());
            target.hp.hit(dmg);
        }
        SubStateTransition::NextSubState(TurnSubState::NextAction)
    }
}

struct DoIt {
    action: Rc<dyn Action>,
    caster: ActorIdentifier,
    target: ActionTarget,
}

impl DoIt {
    pub fn new(action: Rc<dyn Action>, caster: ActorIdentifier, target: ActionTarget) -> DoIt {
        DoIt {
            action,
            caster,
            target,
        }
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
        let action_record = match scene
            .allies_actions
            .iter()
            .find(|rec| rec.id_in_team == action.id_in_team)
        {
            Some(a) => a,
            _ => unreachable!("[ERROR] An action record should always involve a character."),
        };
        let action_str = match action_record.action_type {
            ActionType::Bash => "Bash",
            ActionType::Psi => "PSI",
            ActionType::Item => "Item",
            ActionType::Guard => "Guard",
        };
        println!(
            "→ {} ({}) will act ({})",
            ally.name, action_record.id_in_team, action_str
        );
        let action = match action_record.action_type {
            ActionType::Bash => Rc::new(Bash::new()),
            _ => unimplemented!(),
        };
        let caster = (Team::Ally, action_record.id_in_team);
        // TODO Keep track of target
        let target = ActionTarget::Single((Team::Enemy, 0));

        SubStateTransition::NextSubState(TurnSubState::Announce(Announce::new(
            action, caster, target,
        )))
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
                let action = Rc::new(Bash::new());
                SubStateTransition::NextSubState(TurnSubState::Announce(Announce::new(
                    action,
                    (Team::Enemy, next_action.id_in_team),
                    ActionTarget::Single((Team::Ally, 0)),
                )))
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
                TurnSubState::DoIt(do_it) => {
                    // Cloning the caster allows for broad selection.
                    // Think as a snapshot. Sadly it also copies the name for now.
                    // TODO avoid cloning the name
                    let caster = match do_it.caster.0 {
                        Team::Ally => scene.characters[do_it.caster.1].clone(),
                        Team::Enemy => scene.enemies[do_it.caster.1].clone(),
                    };

                    let characters = &mut scene.characters;
                    let enemies = &mut scene.enemies;

                    let targets = match &do_it.target {
                        ActionTarget::Single((team, id)) => {
                            let v = match team {
                                Team::Ally => characters,
                                Team::Enemy => enemies,
                            };
                            &mut v[*id..*id + 1]
                        }
                        ActionTarget::WholeTeam(team) => match team {
                            Team::Ally => characters,
                            Team::Enemy => enemies,
                        },
                    };

                    Rc::get_mut(&mut do_it.action)
                        .unwrap()
                        .go(&caster, targets, ctx)
                }
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
