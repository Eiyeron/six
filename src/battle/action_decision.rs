// Engine states?
use crate::battle::Actor;
use crate::battle::MacroBattleStates;
use crate::Assets;
use crate::BattleScene;
use tetra::graphics::text::Text;
use tetra::graphics::DrawParams;
use tetra::input::{is_key_pressed, Key};
use tetra::math::Vec2;
use tetra::Context;

pub enum CharacterTurnDecisionState {
    Menu(Menu),
    Bash(BashTargetSelection),
    //
    SpecialMoveSelection(u8),
    SpecialTargetSelection,
    //
    ItemSelection,
    ItemTargetSelection,
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

    fn next_character(characters: &[Actor], current: usize) -> MacroBattleStates {
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
                CharacterTurnDecisionState::Menu(menu) => menu.update(ctx),
                CharacterTurnDecisionState::Bash(bash) => bash.update(ctx, &scene.enemies),
                _ => CharacterTurnDecisionTransition::None,
            };

            match result {
                CharacterTurnDecisionTransition::None => (),
                CharacterTurnDecisionTransition::SwitchTo(new_state) => {
                    scene.state = MacroBattleStates::CharacterTurnDecision(new_state)
                }
                CharacterTurnDecisionTransition::Validate(action) => {
                    if action.character == scene.characters.len() - 1 {
                        // TODO Whole turn system
                        scene.state = MacroBattleStates::CharacterTurnDecision(
                            CharacterTurnDecisionState::new_turn(&scene.characters).unwrap(),
                        );
                    } else {
                        scene.state = CharacterTurnDecisionState::next_character(
                            &scene.characters,
                            action.character,
                        );
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
                _ => (),
            }
        }
    }
}

struct ValidatedAction {
    character: usize,
}

enum CharacterTurnDecisionTransition {
    None,
    Validate(ValidatedAction), // TODO content
    SwitchTo(CharacterTurnDecisionState),
}

#[derive(Copy, Clone)]
struct Breadcrumbs {
    current_item: usize,
    current_character: usize,
}

struct Menu {
    shared: Breadcrumbs,
}

impl Menu {
    fn update(&mut self, ctx: &Context) -> CharacterTurnDecisionTransition {
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
            return CharacterTurnDecisionTransition::SwitchTo(CharacterTurnDecisionState::Bash(
                BashTargetSelection {
                    shared: self.shared,
                    selected: 0,
                },
            ));
        }
        if is_key_pressed(ctx, Key::Backspace) && self.shared.current_character > 0 {
            // TODO announce substate change
            return CharacterTurnDecisionTransition::SwitchTo(CharacterTurnDecisionState::Menu(
                Menu {
                    shared: Breadcrumbs {
                        current_character: self.shared.current_character - 1,
                        current_item: 0,
                    },
                },
            ));
        }
        CharacterTurnDecisionTransition::None
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, character: &Actor) {
        let mut debug_text = Text::new("--Turn Selection--\n", assets.headupdaisy.clone());
        debug_text.push_str(&format!("Char: {}\n", character.name));
        debug_text.push_str(&format!("Selected: {}\n", self.shared.current_item));
        debug_text.draw(ctx, DrawParams::new().position(Vec2::new(16., 360.)));
    }
}

struct BashTargetSelection {
    shared: Breadcrumbs,
    selected: usize,
}

impl BashTargetSelection {
    fn update(&mut self, ctx: &Context, enemies: &[Actor]) -> CharacterTurnDecisionTransition {
        if is_key_pressed(ctx, Key::Left) {
            if self.selected > 0 {
                self.selected -= 1;
            }
        }
        if is_key_pressed(ctx, Key::Right) {
            // TODO determine hud state from current character
            if self.selected < enemies.len() {
                self.selected += 1;
            }
        }

        if is_key_pressed(ctx, Key::Backspace) {
            return CharacterTurnDecisionTransition::SwitchTo(CharacterTurnDecisionState::Menu(
                Menu {
                    shared: self.shared,
                },
            ));
        }

        if is_key_pressed(ctx, Key::Enter) {
            return CharacterTurnDecisionTransition::Validate(ValidatedAction {
                character: self.shared.current_character,
            });
        }

        CharacterTurnDecisionTransition::None
    }

    fn draw(&self, ctx: &mut Context, assets: &Assets, enemies: &[Actor]) {
        let enemy = &enemies[self.selected];
        let mut debug_text = Text::new("--Bash selection--\n", assets.headupdaisy.clone());
        debug_text.push_str(&format!("Char: {} ({})\n", enemy.name, self.selected));
        debug_text.draw(ctx, DrawParams::new().position(Vec2::new(16., 360.)));
    }
}
