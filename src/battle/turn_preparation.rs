use crate::battle::MacroBattleStates;
use crate::battle::MacroBattleStates::TurnUnroll;
use crate::battle::Team;
use crate::battle::TurnAction;
use crate::battle::TurnUnrollState;
use crate::Assets;
use crate::BattleScene;
use tetra::graphics::text::Text;
use tetra::graphics::DrawParams;
use tetra::math::Vec2;
use tetra::Context;

pub struct TurnPreparationState;

impl TurnPreparationState {
    pub fn update(scene: &mut BattleScene, _ctx: &Context) {
        if let MacroBattleStates::TurnPreparation(_) = &mut scene.state {
            scene.turn_order.clear();
            for action in scene.allies_actions.iter() {
                scene.turn_order.push_back(TurnAction {
                    team: Team::Ally,
                    speed: action.registered_speed,
                    id_in_team: action.id,
                })
            }
            for (id, enemy) in scene.enemies.iter().enumerate() {
                if enemy.hp.current_value > 0 {
                    scene.turn_order.push_back(TurnAction {
                        team: Team::Enemy,
                        speed: enemy.speed.multiplied(),
                        id_in_team: id,
                    })
                }
            }
            scene
                .turn_order
                .make_contiguous()
                .sort_by(|a, b| b.speed.cmp(&a.speed)); // Reverse order
            scene.turn_order.make_contiguous().reverse();
            // TODO Consume actions (unroll turn)
            scene.allies_actions.clear();
            scene.state = TurnUnroll(TurnUnrollState::new());
        }
    }
    pub fn draw(_scene: &BattleScene, ctx: &mut Context, assets: &Assets) {
        let mut debug_text = Text::new("--Turn Preparation--\n", assets.headupdaisy.clone());
        debug_text.draw(ctx, DrawParams::new().position(Vec2::new(16., 360.)));
    }
}