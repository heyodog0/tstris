use std::collections::HashMap;
use std::time::{Duration, Instant};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

use crate::input::direction::{InputDirection, DirectionState};
use crate::constants::KEY_TIMEOUT;

pub struct InputState {
    pub directions: HashMap<InputDirection, DirectionState>,
    pub last_horizontal_dir: Option<InputDirection>,
    pub keyboard_enhancement_active: bool,
}

impl InputState {
    pub fn new() -> Self {
        let mut directions = HashMap::new();
        directions.insert(InputDirection::Left, DirectionState::new());
        directions.insert(InputDirection::Right, DirectionState::new());
        directions.insert(InputDirection::Down, DirectionState::new());
        
        Self { 
            directions,
            last_horizontal_dir: None,
            keyboard_enhancement_active: false,
        }
    }

    pub fn press_direction(&mut self, dir: InputDirection) {
        match dir {
            InputDirection::Left => {
                self.release_direction(InputDirection::Right);
                self.last_horizontal_dir = Some(InputDirection::Left);
            }
            InputDirection::Right => {
                self.release_direction(InputDirection::Left);
                self.last_horizontal_dir = Some(InputDirection::Right);
            }
            _ => {}
        }

        if let Some(state) = self.directions.get_mut(&dir) {
            state.press();
        }
    }

    pub fn release_direction(&mut self, dir: InputDirection) {
        if let Some(state) = self.directions.get_mut(&dir) {
            state.release();
        }

        if self.last_horizontal_dir == Some(dir) {
            self.last_horizontal_dir = None;
        }
    }

    pub fn is_pressed(&self, dir: InputDirection) -> bool {
        self.directions.get(&dir).map_or(false, |s| s.pressed)
    }

    pub fn reset_das_states(&mut self) {
        for state in self.directions.values_mut() {
            state.reset_das();
        }
    }

    pub fn check_timeouts(&mut self) {
        if !self.keyboard_enhancement_active {
            let now = Instant::now();
            for state in self.directions.values_mut() {
                if state.pressed && now.duration_since(state.last_update) > Duration::from_millis(KEY_TIMEOUT) {
                    state.release();
                }
            }
        }
    }

    pub fn update_key_activity(&mut self, dir: InputDirection) {
        if let Some(state) = self.directions.get_mut(&dir) {
            state.last_update = Instant::now();
        }
    }
}

pub fn handle_input(game: &mut crate::game::Game, key_code: KeyCode, kind: KeyEventKind, modifiers: KeyModifiers) {
    match kind {
        KeyEventKind::Press | KeyEventKind::Repeat => {
            match key_code {
                KeyCode::Left => {
                    if !game.input_state.is_pressed(InputDirection::Left) {
                        game.input_state.press_direction(InputDirection::Left);
                    } else {
                        game.input_state.update_key_activity(InputDirection::Left);
                    }
                }
                KeyCode::Right => {
                    if !game.input_state.is_pressed(InputDirection::Right) {
                        game.input_state.press_direction(InputDirection::Right);
                    } else {
                        game.input_state.update_key_activity(InputDirection::Right);
                    }
                }
                KeyCode::Down => {
                    if !game.input_state.is_pressed(InputDirection::Down) {
                        game.input_state.press_direction(InputDirection::Down);
                    } else {
                        game.input_state.update_key_activity(InputDirection::Down);
                    }
                }
                KeyCode::Up => {
                    game.rotate_piece(); // Rotate right (clockwise)
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    game.rotate_piece_left(); // Rotate left (counter-clockwise)
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    game.rotate_piece_180();
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    game.hard_drop();
                }
                KeyCode::Char(' ') => {
                    match game.game_state {
                        crate::game::state::GameState::Ready => {
                            game.start_countdown();
                        }
                        crate::game::state::GameState::Playing => {
                            game.hard_drop();
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('h') | KeyCode::Char('H') => {
                    game.hold_piece();
                }
                _ => {
                    // Handle left shift for hold
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        game.hold_piece();
                    }
                }
            }
        }
        KeyEventKind::Release => {
            match key_code {
                KeyCode::Left => {
                    game.input_state.release_direction(InputDirection::Left);
                }
                KeyCode::Right => {
                    game.input_state.release_direction(InputDirection::Right);
                }
                KeyCode::Down => {
                    game.input_state.release_direction(InputDirection::Down);
                }
                _ => {}
            }
        }
    }
}