use std::time::{Duration, Instant};
use rand::Rng;

use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT, TARGET_LINES, GROUND_TIME};
use crate::game::board::{Board, Cell, empty_board};
use crate::game::piece::{Piece, PieceType};
use crate::input::handler::InputState;
use crate::input::direction::InputDirection;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameState {
    Ready,
    Countdown(u32), // Countdown number (3, 2, 1)
    Playing,
    Finished,
}

pub struct Game {
    pub board: Board,
    pub current_piece: Option<Piece>,
    pub next_pieces: Vec<Piece>,  // Queue of next 5 pieces
    pub hold_piece: Option<Piece>,
    pub can_hold: bool,
    pub lines_cleared: u32,
    pub lines_remaining: u32,
    pub drop_timer: Instant,
    pub input_state: InputState,
    pub game_state: GameState,
    pub countdown_timer: Instant,
    pub game_timer: Option<Instant>,
    pub final_time: Option<Duration>,
    pub ground_timer: Option<Instant>, // Timer for piece on ground
    pub piece_bag: Vec<PieceType>,     // 7-bag randomizer
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            board: empty_board(),
            current_piece: None,
            next_pieces: Vec::new(),
            hold_piece: None,
            can_hold: true,
            lines_cleared: 0,
            lines_remaining: TARGET_LINES,
            drop_timer: Instant::now(),
            input_state: InputState::new(),
            game_state: GameState::Ready,
            countdown_timer: Instant::now(),
            game_timer: None,
            final_time: None,
            ground_timer: None,
            piece_bag: Vec::new(),
        };
        
        // Initialize the next pieces queue with 5 pieces
        game.fill_next_pieces();
        game
    }

    fn fill_bag(&mut self) {
        // Create a new bag with all 7 piece types
        self.piece_bag = vec![
            PieceType::I, PieceType::O, PieceType::T, 
            PieceType::S, PieceType::Z, PieceType::J, PieceType::L
        ];
        
        // Shuffle the bag using Fisher-Yates shuffle
        let mut rng = rand::thread_rng();
        for i in (1..self.piece_bag.len()).rev() {
            let j = rng.gen_range(0..=i);
            self.piece_bag.swap(i, j);
        }
    }
    
    fn get_next_piece_type(&mut self) -> PieceType {
        if self.piece_bag.is_empty() {
            self.fill_bag();
        }
        self.piece_bag.pop().unwrap()
    }
    
    fn fill_next_pieces(&mut self) {
        while self.next_pieces.len() < 5 {
            let piece_type = self.get_next_piece_type();
            self.next_pieces.push(Piece::new(piece_type));
        }
    }

    pub fn start_countdown(&mut self) {
        if self.game_state == GameState::Ready {
            self.game_state = GameState::Countdown(3);
            self.countdown_timer = Instant::now();
        }
    }

    pub fn start_game(&mut self) {
        self.game_state = GameState::Playing;
        self.game_timer = Some(Instant::now());
        self.spawn_piece();
    }

    pub fn spawn_piece(&mut self) {
        if self.game_state != GameState::Playing {
            return;
        }
        
        // Get the next piece from the queue
        if !self.next_pieces.is_empty() {
            self.current_piece = Some(self.next_pieces.remove(0));
            
            // Refill the queue to maintain 5 pieces
            self.fill_next_pieces();
        }
        
        self.can_hold = true; // Reset hold ability when spawning new piece
        self.ground_timer = None; // Reset ground timer
        
        if let Some(ref piece) = self.current_piece {
            if !self.is_valid_position(piece) {
                self.game_state = GameState::Finished;
                if let Some(start_time) = self.game_timer {
                    self.final_time = Some(start_time.elapsed());
                }
            }
        }
    }

    pub fn is_valid_position(&self, piece: &Piece) -> bool {
        for (x, y) in piece.get_blocks() {
            if x < 0 || x >= BOARD_WIDTH as i32 || y >= BOARD_HEIGHT as i32 {
                return false;
            }
            if y >= 0 && self.board[y as usize][x as usize] != Cell::Empty {
                return false;
            }
        }
        true
    }

    pub fn get_ghost_piece(&self) -> Option<Piece> {
        if let Some(ref current_piece) = self.current_piece {
            let mut ghost = current_piece.clone();
            
            // Drop the ghost piece as far down as possible
            while self.is_valid_position(&ghost) {
                ghost.y += 1;
            }
            ghost.y -= 1; // Back up one position to the last valid position
            
            // Only return ghost if it's different from current piece position
            if ghost.y != current_piece.y {
                Some(ghost)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if self.game_state != GameState::Playing {
            return false;
        }
        
        if let Some(piece) = self.current_piece.take() {
            let mut test_piece = piece.clone();
            test_piece.x += dx;
            test_piece.y += dy;
            
            if self.is_valid_position(&test_piece) {
                self.current_piece = Some(test_piece);
                
                // Reset ground timer if moving horizontally while on ground
                if dx != 0 {
                    self.ground_timer = None;
                }
                
                return true;
            } else {
                self.current_piece = Some(piece);
                
                // If moving down failed, start ground timer
                if dy > 0 && self.ground_timer.is_none() {
                    self.ground_timer = Some(Instant::now());
                }
            }
        }
        false
    }

    pub fn rotate_piece(&mut self) -> bool {
        if let Some(ref piece) = self.current_piece {
            let rotated = piece.rotate_clockwise();
            
            // Try basic rotation
            if self.is_valid_position(&rotated) {
                self.current_piece = Some(rotated);
                return true;
            }
            
            // Try wall kicks
            let kicks = match piece.piece_type {
                PieceType::I => vec![(1, 0), (-1, 0), (2, 0), (-2, 0), (0, -1)],
                _ => vec![(1, 0), (-1, 0), (0, -1), (1, -1), (-1, -1)],
            };
            
            for (kick_x, kick_y) in kicks {
                let mut kicked = rotated.clone();
                kicked.x += kick_x;
                kicked.y += kick_y;
                
                if self.is_valid_position(&kicked) {
                    self.current_piece = Some(kicked);
                    return true;
                }
            }
        }
        false
    }

    pub fn rotate_piece_left(&mut self) -> bool {
        if let Some(ref piece) = self.current_piece {
            let rotated = piece.rotate_counter_clockwise();
            
            // Try basic rotation
            if self.is_valid_position(&rotated) {
                self.current_piece = Some(rotated);
                return true;
            }
            
            // Try wall kicks
            let kicks = match piece.piece_type {
                PieceType::I => vec![(1, 0), (-1, 0), (2, 0), (-2, 0), (0, -1)],
                _ => vec![(1, 0), (-1, 0), (0, -1), (1, -1), (-1, -1)],
            };
            
            for (kick_x, kick_y) in kicks {
                let mut kicked = rotated.clone();
                kicked.x += kick_x;
                kicked.y += kick_y;
                
                if self.is_valid_position(&kicked) {
                    self.current_piece = Some(kicked);
                    return true;
                }
            }
        }
        false
    }

    pub fn rotate_piece_180(&mut self) -> bool {
        if let Some(ref piece) = self.current_piece {
            let rotated = piece.rotate_180();
            
            // Try basic rotation
            if self.is_valid_position(&rotated) {
                self.current_piece = Some(rotated);
                return true;
            }
            
            // Try wall kicks (same as regular rotation)
            let kicks = match piece.piece_type {
                PieceType::I => vec![(1, 0), (-1, 0), (2, 0), (-2, 0), (0, -1)],
                _ => vec![(1, 0), (-1, 0), (0, -1), (1, -1), (-1, -1)],
            };
            
            for (kick_x, kick_y) in kicks {
                let mut kicked = rotated.clone();
                kicked.x += kick_x;
                kicked.y += kick_y;
                
                if self.is_valid_position(&kicked) {
                    self.current_piece = Some(kicked);
                    return true;
                }
            }
        }
        false
    }

    pub fn hold_piece(&mut self) {
        if !self.can_hold || self.game_state != GameState::Playing {
            return;
        }
        
        if let Some(current) = self.current_piece.take() {
            if let Some(held) = self.hold_piece.take() {
                // Swap current with held piece
                self.current_piece = Some(held);
            } else {
                // First time holding, get next piece from queue
                if !self.next_pieces.is_empty() {
                    self.current_piece = Some(self.next_pieces.remove(0));
                    self.fill_next_pieces();
                }
            }
            
            // Reset the held piece to spawn position
            let mut held_piece = current;
            held_piece.x = (BOARD_WIDTH as i32 - 4) / 2;
            held_piece.y = 0;
            self.hold_piece = Some(held_piece);
            
            self.can_hold = false; // Can't hold again until next spawn
            
            // Check if new current piece is valid
            if let Some(ref piece) = self.current_piece {
                if !self.is_valid_position(piece) {
                    self.game_state = GameState::Finished;
                    if let Some(start_time) = self.game_timer {
                        self.final_time = Some(start_time.elapsed());
                    }
                }
            }
        }
    }

    pub fn hard_drop(&mut self) {
        while self.move_piece(0, 1) {}
        self.lock_piece();
    }

    pub fn lock_piece(&mut self) {
        if let Some(ref piece) = self.current_piece {
            for (x, y) in piece.get_blocks() {
                if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                    self.board[y as usize][x as usize] = Cell::Filled(piece.color);
                }
            }
        }
        
        self.current_piece = None;
        let lines = self.clear_lines();
        self.update_lines(lines);
        
        // Reset DAS states when piece locks to prevent new piece from flying away
        self.input_state.reset_das_states();
        
        self.spawn_piece();
        self.drop_timer = Instant::now();
    }

    fn clear_lines(&mut self) -> u32 {
        let mut lines_cleared = 0;
        let mut write_row = BOARD_HEIGHT - 1;
        
        // Start from bottom and work up, copying non-full rows down
        for read_row in (0..BOARD_HEIGHT).rev() {
            if !self.board[read_row].iter().all(|&cell| cell != Cell::Empty) {
                // This row is not full, keep it
                if read_row != write_row {
                    self.board[write_row] = self.board[read_row];
                }
                if write_row > 0 {
                    write_row -= 1;
                }
            } else {
                // This row is full, skip it (clear it)
                lines_cleared += 1;
            }
        }
        
        // Fill remaining top rows with empty
        for row in 0..=write_row {
            self.board[row] = [Cell::Empty; BOARD_WIDTH];
        }
        
        lines_cleared
    }

    fn update_lines(&mut self, lines: u32) {
        self.lines_cleared += lines;
        self.lines_remaining = self.lines_remaining.saturating_sub(lines);
        
        // Check if 40L sprint is complete
        if self.lines_remaining == 0 {
            self.game_state = GameState::Finished;
            if let Some(start_time) = self.game_timer {
                self.final_time = Some(start_time.elapsed());
            }
        }
    }

    fn get_drop_delay(&self) -> Duration {
        Duration::from_millis(1000) // Fixed 1 second drop delay for 40L sprint
    }

    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Instant::now();
        
        match self.game_state {
            GameState::Ready => {
                // Waiting for user to start
                return Ok(());
            }
            GameState::Countdown(count) => {
                if now.duration_since(self.countdown_timer) >= Duration::from_millis(1000) {
                    if count > 1 {
                        self.game_state = GameState::Countdown(count - 1);
                        self.countdown_timer = now;
                    } else {
                        self.start_game();
                    }
                }
                return Ok(());
            }
            GameState::Finished => {
                return Ok(());
            }
            GameState::Playing => {
                // Continue with normal game logic
            }
        }

        self.input_state.check_timeouts();

        self.handle_movement(InputDirection::Left, -1, 0, now);
        self.handle_movement(InputDirection::Right, 1, 0, now);
        self.handle_soft_drop(now);

        // Check ground timer for piece locking
        if let Some(ground_time) = self.ground_timer {
            if now.duration_since(ground_time) >= Duration::from_millis(GROUND_TIME) {
                self.lock_piece();
                return Ok(());
            }
        }

        // Handle gravity drop
        if now.duration_since(self.drop_timer) >= self.get_drop_delay() {
            self.drop_timer = now;
            if !self.move_piece(0, 1) {
                // Start ground timer if not already started
                if self.ground_timer.is_none() {
                    self.ground_timer = Some(now);
                }
            }
        }

        Ok(())
    }

    fn handle_movement(&mut self, direction: InputDirection, dx: i32, dy: i32, now: Instant) {
        if let Some(state) = self.input_state.directions.get_mut(&direction) {
            if state.pressed {
                let mut should_move = false;
                
                if !state.initial_move_done {
                    should_move = true;
                    state.initial_move_done = true;
                } else if !state.das_charged {
                    if now.duration_since(state.das_timer) >= Duration::from_millis(crate::constants::DAS_DELAY) {
                        state.das_charged = true;
                        state.arr_timer = now;
                        should_move = true;
                    }
                } else {
                    if now.duration_since(state.arr_timer) >= Duration::from_millis(crate::constants::ARR_DELAY) {
                        state.arr_timer = now;
                        should_move = true;
                    }
                }
                
                if should_move {
                    self.move_piece(dx, dy);
                }
            }
        }
    }

    fn handle_soft_drop(&mut self, now: Instant) {
        if let Some(down_state) = self.input_state.directions.get_mut(&InputDirection::Down) {
            if down_state.pressed {
                let mut should_move = false;
                
                if !down_state.initial_move_done {
                    should_move = true;
                    down_state.initial_move_done = true;
                } else if now.duration_since(down_state.arr_timer) >= Duration::from_millis(crate::constants::SOFT_DROP_DELAY) {
                    down_state.arr_timer = now;
                    should_move = true;
                }
                
                if should_move {
                    if !self.move_piece(0, 1) {
                        // Don't immediately lock - let ground timer handle it
                        if self.ground_timer.is_none() {
                            self.ground_timer = Some(now);
                        }
                    }
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.board = empty_board();
        self.current_piece = None;
        self.next_pieces.clear();
        self.piece_bag.clear();
        self.hold_piece = None;
        self.can_hold = true;
        self.lines_cleared = 0;
        self.lines_remaining = TARGET_LINES;
        self.drop_timer = Instant::now();
        self.input_state = InputState::new();
        self.game_timer = None;
        self.final_time = None;
        self.ground_timer = None;
        
        // Refill the next pieces queue
        self.fill_next_pieces();
        
        // Auto-start countdown
        self.game_state = GameState::Countdown(3);
        self.countdown_timer = Instant::now();
    }
    
    pub fn get_current_time(&self) -> Option<Duration> {
        if let Some(start_time) = self.game_timer {
            match self.game_state {
                GameState::Playing => Some(start_time.elapsed()),
                GameState::Finished => self.final_time,
                _ => None,
            }
        } else {
            None
        }
    }
}