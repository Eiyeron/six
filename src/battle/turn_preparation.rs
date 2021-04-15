use crate::battle::MacroBattleStates;
use crate::battle::MacroBattleStates::TurnUnroll;
use crate::battle::{BattleState, BattleStateTransition};
use crate::battle::{Team, TurnAction, TurnUnrollState};
use crate::{Assets, BattleScene};
use tetra::graphics::text::Text;
use tetra::graphics::{Color, DrawParams};
use tetra::math::Vec2;
use tetra::Context;

pub struct TurnPreparationState;

impl BattleState for TurnPreparationState {
    fn update(scene: &mut BattleScene, _ctx: &Context) -> BattleStateTransition {
        if let MacroBattleStates::TurnPreparation(_) = &mut scene.state {
            scene.turn_order.clear();
            for action in scene.allies_actions.iter() {
                scene.turn_order.push_back(TurnAction {
                    team: Team::Ally,
                    speed: action.registered_speed,
                    id_in_team: action.id_in_team,
                })
            }
            for (id, enemy) in scene.enemies.iter().enumerate() {
                let (current_hp, _) = enemy.hp.current_and_max();
                if current_hp > 0 {
                    scene.turn_order.push_back(TurnAction {
                        id_in_team: id,
                        team: Team::Enemy,
                        // TODO randomized speed
                        speed: enemy.stats.speed.multiplied(),
                    })
                }
            }
            scene
                .turn_order
                .make_contiguous()
                .sort_by(|a, b| b.speed.cmp(&a.speed)); // Reverse order
            scene.turn_order.make_contiguous().reverse();
            // TODO Consume actions (unroll turn)

            return Some(TurnUnroll(TurnUnrollState::new()));
        }
        None
    }
}

impl TurnPreparationState {
    pub fn draw(_scene: &BattleScene, ctx: &mut Context, assets: &Assets) {
        let mut debug_text = Text::new("--Turn Preparation--\n", assets.headupdaisy.clone());
        debug_text.draw(
            ctx,
            DrawParams::new()
                .color(Color::rgb8(0xeb, 0xdb, 0xb2))
                .position(Vec2::new(16., 360.)),
        );
    }
}
