// Engine states?
use crate::battle::turn_preparation::TurnPreparationState;
use crate::battle::ActionType;
use crate::battle::Actor;
use crate::battle::MacroBattleStates;
use crate::Assets;
use crate::BattleScene;
use tetra::graphics::text::Text;
use tetra::graphics::DrawParams;
use tetra::input::{is_key_pressed, Key};
use tetra::math::Vec2;
use tetra::Context;

pub struct AllyActionRecord {
    pub id: usize,
    pub registered_speed: u16,
    pub action_type: ActionType,
}

pub enum CharacterTurnDecisionState {
    Menu(Menu),
    Bash(BashTargetSelection),
    //
    // SpecialMoveSelection(u8),
    // SpecialTargetSelection,
    //
    // ItemSelection,
    // ItemTargetSelection,
    //
}

impl CharacterTurnDecisionState {
    pub fn new_turn(characters: &[Actor]) -> Option<CharacterTurnDecisionState> {
        if let Some((i, _)) = characters
            .iter()
            .enumerate()
            .find(|(_, c)| c.hp.current_value > 0)
        {
            return Some(CharacterTurnDecisionState::Menu(Menu {
                shared: Breadcrumbs {
                    current_character: i,
                    current_item: 0,
                },
            }));
        }
        None
    }

    fn next_character(_characters: &[Actor], current: usize) -> MacroBattleStates {
        MacroBattleStates::CharacterTurnDecision(CharacterTurnDecisionState::Menu(Menu {
            // TODO Determine end of action (here or somewhere else, I guess)
            shared: Breadcrumbs {
                current_character: current + 1,
                current_item: 0,
            },
        }))
    }

    pub fn update(scene: &mut BattleScene, ctx: &Context) {
        if let MacroBattleStates::CharacterTurnDecision(sub_state) = &mut scene.state {
            let result = match sub_state {
                CharacterTurnDecisionState::Menu(menu) => menu.update(&scene.characters, ctx),
                CharacterTurnDecisionState::Bash(bash) => {
                    bash.update(ctx, &scene.characters, &scene.enemies)
                }
            };

            match result {
                Transition::None => (),
                Transition::SwitchTo(new_state) => {
                    scene.state = MacroBattleStates::CharacterTurnDecision(new_state)
                }
                Transition::Validate(action) => {
                    if action.id == scene.characters.len() - 1 {
                        // TODO Whole turn system
                        scene.state = MacroBattleStates::TurnPreparation(TurnPreparationState {});
                    } else {
                        // TODO Whole turn system and action structure passing.
                        scene.state = CharacterTurnDecisionState::next_character(
                            &scene.characters,
                            action.id,
                        );
                        scene.allies_actions.push(action);
                    }
                }
            }
        }
    }

    pub fn draw(scene: &BattleScene, ctx: &mut Context, assets: &Assets) {
        if let MacroBattleStates::CharacterTurnDecision(sub_state) = &scene.state {
            match sub_state {
                CharacterTurnDecisionState::Menu(menu) => menu.draw(
                    ctx,
                    assets,
                    &scene.characters[menu.shared.current_character],
                ),
                CharacterTurnDecisionState::Bash(bash) => {
                    bash.draw(ctx, assets, &scene.enemies);
                }
            }
        }
    }
}

enum Transition {
    None,
    Validate(AllyActionRecord), // TODO content
    SwitchTo(CharacterTurnDecisionState),
}

#[derive(Copy, Clone)]
struct Breadcrumbs {
    current_item: usize,
    current_character: usize,
}

pub struct Menu {
    shared: Breadcrumbs,
}

impl Menu {
    const MENU_NAMES: &'static [&'static str] = &["Bash", "PSI", "Item", "Guard", "Flee"];

    fn update(&mut self, characters: &[Actor], ctx: &Context) -> Transition {
        if is_key_pressed(ctx, Key::Left) {
            if self.shared.current_item > 0 {
                self.shared.current_item -= 1;
            }
        }
        if is_key_pressed(ctx, Key::Right) {
            // TODO determine hud state from current character
            if self.shared.current_item < 4 {
                self.shared.current_item += 1;
            }
        }
        if is_key_pressed(ctx, Key::PageUp) {
            self.shared.current_item = 0;
        }
        if is_key_pressed(ctx, Key::PageDown) {
            self.shared.current_item = 4;
        }
        if is_key_pressed(ctx, Key::Enter) {
            // TODO announce substate change
            if self.shared.current_item == 0 {
                return Transition::SwitchTo(CharacterTurnDecisionState::Bash(
                    BashTargetSelection {
                        shared: self.shared,
                        selected: 0,
                    },
                ));
            }
            if self.shared.current_item == 4 {
                return Transition::Validate(AllyActionRecord {
                    id: self.shared.current_character,
                    registered_speed: characters[self.shared.current_character].speed.multiplied(),
                    action_type: ActionType::Guard,
                });
            }
        }
        if is_key_pressed(ctx, Key::Backspace) && self.shared.current_character > 0 {
            // TODO announce substate change
            return Transition::SwitchTo(CharacterTurnDecisionState::Menu(Menu {
                shared: Breadcrumbs {
                    current_character: self.shared.current_character - 1,
                    current_item: 0,
                },
            }));
        }
        Transition::None
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, character: &Actor) {
        let mut debug_text = Text::new("--Turn Selection--\n", assets.headupdaisy.clone());
        debug_text.push_str(&format!("Char: {}\n", character.name));
        debug_text.push_str(&format!(
            "Selected: {}\n",
            Menu::MENU_NAMES[self.shared.current_item]
        ));
        debug_text.draw(ctx, DrawParams::new().position(Vec2::new(16., 360.)));
    }
}

pub struct BashTargetSelection {
    shared: Breadcrumbs,
    selected: usize,
}

impl BashTargetSelection {
    fn update(&mut self, ctx: &Context, allies: &[Actor], enemies: &[Actor]) -> Transition {
        if is_key_pressed(ctx, Key::Left) {
            if self.selected > 0 {
                self.selected -= 1;
            }
        }
        if is_key_pressed(ctx, Key::Right) {
            // TODO determine hud state from current character
            if self.selected < enemies.len() - 1 {
                self.selected += 1;
            }
        }

        if is_key_pressed(ctx, Key::Backspace) {
            return Transition::SwitchTo(CharacterTurnDecisionState::Menu(Menu {
                shared: self.shared,
            }));
        }

        if is_key_pressed(ctx, Key::Enter) {
            return Transition::Validate(AllyActionRecord {
                id: self.shared.current_character,
                // TODO passing target id
                action_type: ActionType::Bash,
                registered_speed: allies[self.shared.current_character].speed.multiplied(),
            });
        }

        Transition::None
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, enemies: &[Actor]) {
        let enemy = &enemies[self.selected];
        let mut debug_text = Text::new("--Bash selection--\n", assets.headupdaisy.clone());
        debug_text.push_str(&format!("Char: {} ({})\n", enemy.name, self.selected));
        debug_text.draw(ctx, DrawParams::new().position(Vec2::new(16., 360.)));
    }
}
