use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use std::{
    io::stdout,
    time::{Duration, Instant},
};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

// DAS and ARR settings (in milliseconds)
const DAS_DELAY: u64 = 167;  // Delayed Auto Shift - delay before repeating
const ARR_DELAY: u64 = 33;   // Auto Repeat Rate - delay between repeats
const SOFT_DROP_DELAY: u64 = 50; // Soft drop speed
const KEY_TIMEOUT: u64 = 100; // Timeout for key release detection fallback

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell {
    Empty,
    Filled(Color),
}

#[derive(Clone, Copy, Debug)]
enum PieceType {
    I, O, T, S, Z, J, L
}

#[derive(Clone, Debug)]
struct Piece {
    piece_type: PieceType,
    shape: Vec<Vec<bool>>,
    x: i32,
    y: i32,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InputDirection {
    Left,
    Right,
    Down,
}

#[derive(Debug)]
struct DirectionState {
    pressed: bool,
    das_timer: Instant,
    arr_timer: Instant,
    das_charged: bool,
    initial_move_done: bool,
    last_update: Instant, // For timeout-based release detection
}

impl DirectionState {
    fn new() -> Self {
        Self {
            pressed: false,
            das_timer: Instant::now(),
            arr_timer: Instant::now(),
            das_charged: false,
            initial_move_done: false,
            last_update: Instant::now(),
        }
    }

    fn press(&mut self) {
        self.pressed = true;
        let now = Instant::now();
        self.das_timer = now;
        self.arr_timer = now;
        self.das_charged = false;
        self.initial_move_done = false;
        self.last_update = now;
    }

    fn release(&mut self) {
        self.pressed = false;
        self.das_charged = false;
        self.initial_move_done = false;
        self.last_update = Instant::now();
    }

    fn reset_das(&mut self) {
        // Reset DAS charging but keep pressed state
        if self.pressed {
            let now = Instant::now();
            self.das_timer = now;
            self.arr_timer = now;
            self.das_charged = false;
            self.initial_move_done = false;
            self.last_update = now;
        }
    }
}

struct InputState {
    directions: std::collections::HashMap<InputDirection, DirectionState>,
    last_horizontal_dir: Option<InputDirection>, // Track last pressed horizontal direction
    keyboard_enhancement_active: bool,
}

impl InputState {
    fn new() -> Self {
        let mut directions = std::collections::HashMap::new();
        directions.insert(InputDirection::Left, DirectionState::new());
        directions.insert(InputDirection::Right, DirectionState::new());
        directions.insert(InputDirection::Down, DirectionState::new());
        
        Self { 
            directions,
            last_horizontal_dir: None,
            keyboard_enhancement_active: false,
        }
    }

    fn press_direction(&mut self, dir: InputDirection) {
        // Handle direction priority - if pressing left/right, release the opposite
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

    fn release_direction(&mut self, dir: InputDirection) {
        if let Some(state) = self.directions.get_mut(&dir) {
            state.release();
        }

        // Clear last horizontal direction if it matches
        if self.last_horizontal_dir == Some(dir) {
            self.last_horizontal_dir = None;
        }
    }

    fn is_pressed(&self, dir: InputDirection) -> bool {
        self.directions.get(&dir).map_or(false, |s| s.pressed)
    }

    fn reset_das_states(&mut self) {
        // Reset DAS for all directions but keep pressed states
        for state in self.directions.values_mut() {
            state.reset_das();
        }
    }

    fn check_timeouts(&mut self) {
        // Fallback: release keys that haven't been updated recently
        // Only use this if keyboard enhancement is not active
        if !self.keyboard_enhancement_active {
            let now = Instant::now();
            for state in self.directions.values_mut() {
                if state.pressed && now.duration_since(state.last_update) > Duration::from_millis(KEY_TIMEOUT) {
                    state.release();
                }
            }
        }
    }

    fn update_key_activity(&mut self, dir: InputDirection) {
        // Update the last activity time for a key
        if let Some(state) = self.directions.get_mut(&dir) {
            state.last_update = Instant::now();
        }
    }
}

struct Game {
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Option<Piece>,
    next_piece: Piece,
    score: u32,
    lines_cleared: u32,
    level: u32,
    drop_timer: Instant,
    input_state: InputState,
    game_over: bool,
}

impl Piece {
    fn new(piece_type: PieceType) -> Self {
        let (shape, color) = match piece_type {
            PieceType::I => (vec![
                vec![false, false, false, false],
                vec![true, true, true, true],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ], Color::Cyan),
            PieceType::O => (vec![
                vec![true, true],
                vec![true, true],
            ], Color::Yellow),
            PieceType::T => (vec![
                vec![false, true, false],
                vec![true, true, true],
                vec![false, false, false],
            ], Color::Magenta),
            PieceType::S => (vec![
                vec![false, true, true],
                vec![true, true, false],
                vec![false, false, false],
            ], Color::Green),
            PieceType::Z => (vec![
                vec![true, true, false],
                vec![false, true, true],
                vec![false, false, false],
            ], Color::Red),
            PieceType::J => (vec![
                vec![true, false, false],
                vec![true, true, true],
                vec![false, false, false],
            ], Color::Blue),
            PieceType::L => (vec![
                vec![false, false, true],
                vec![true, true, true],
                vec![false, false, false],
            ], Color::LightYellow),
        };

        Self {
            piece_type,
            shape,
            x: (BOARD_WIDTH as i32 - 4) / 2,
            y: 0,
            color,
        }
    }

    fn rotate_clockwise(&self) -> Self {
        let mut rotated = self.clone();
        let size = self.shape.len();
        let mut new_shape = vec![vec![false; size]; size];
        
        for i in 0..size {
            for j in 0..size {
                new_shape[j][size - 1 - i] = self.shape[i][j];
            }
        }
        
        rotated.shape = new_shape;
        rotated
    }

    fn get_blocks(&self) -> Vec<(i32, i32)> {
        let mut blocks = Vec::new();
        for (i, row) in self.shape.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                if cell {
                    blocks.push((self.x + j as i32, self.y + i as i32));
                }
            }
        }
        blocks
    }
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            board: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            next_piece: Self::random_piece(),
            score: 0,
            lines_cleared: 0,
            level: 1,
            drop_timer: Instant::now(),
            input_state: InputState::new(),
            game_over: false,
        };
        game.spawn_piece();
        game
    }

    fn random_piece() -> Piece {
        let mut rng = rand::thread_rng();
        let piece_types = [PieceType::I, PieceType::O, PieceType::T, PieceType::S, PieceType::Z, PieceType::J, PieceType::L];
        let piece_type = piece_types[rng.gen_range(0..piece_types.len())];
        Piece::new(piece_type)
    }

    fn spawn_piece(&mut self) {
        self.current_piece = Some(self.next_piece.clone());
        self.next_piece = Self::random_piece();
        
        if let Some(ref piece) = self.current_piece {
            if !self.is_valid_position(piece) {
                self.game_over = true;
            }
        }
    }

    fn is_valid_position(&self, piece: &Piece) -> bool {
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

    fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if let Some(piece) = self.current_piece.take() {
            let mut test_piece = piece.clone();
            test_piece.x += dx;
            test_piece.y += dy;
            
            if self.is_valid_position(&test_piece) {
                self.current_piece = Some(test_piece);
                return true;
            } else {
                self.current_piece = Some(piece);
            }
        }
        false
    }

    fn rotate_piece(&mut self) -> bool {
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

    fn hard_drop(&mut self) {
        while self.move_piece(0, 1) {}
        self.lock_piece();
    }

    fn lock_piece(&mut self) {
        if let Some(ref piece) = self.current_piece {
            for (x, y) in piece.get_blocks() {
                if y >= 0 && y < BOARD_HEIGHT as i32 && x >= 0 && x < BOARD_WIDTH as i32 {
                    self.board[y as usize][x as usize] = Cell::Filled(piece.color);
                }
            }
        }
        
        self.current_piece = None;
        let lines = self.clear_lines();
        self.update_score(lines);
        
        // Reset DAS states when piece locks to prevent new piece from flying away
        self.input_state.reset_das_states();
        
        self.spawn_piece();
        self.drop_timer = Instant::now();
    }

    fn clear_lines(&mut self) -> u32 {
        let mut lines_to_clear = Vec::new();
        
        for y in 0..BOARD_HEIGHT {
            if self.board[y].iter().all(|&cell| cell != Cell::Empty) {
                lines_to_clear.push(y);
            }
        }
        
        for &y in lines_to_clear.iter().rev() {
            for row in (1..=y).rev() {
                self.board[row] = self.board[row - 1];
            }
            self.board[0] = [Cell::Empty; BOARD_WIDTH];
        }
        
        lines_to_clear.len() as u32
    }

    fn update_score(&mut self, lines: u32) {
        let points = match lines {
            1 => 100 * self.level,
            2 => 300 * self.level,
            3 => 500 * self.level,
            4 => 800 * self.level,
            _ => 0,
        };
        
        self.score += points;
        self.lines_cleared += lines;
        self.level = (self.lines_cleared / 10) + 1;
    }

    fn get_drop_delay(&self) -> Duration {
        let base_delay = 1000 - ((self.level - 1) * 50).min(950);
        Duration::from_millis(base_delay as u64)
    }

    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.game_over {
            return Ok(());
        }

        // Check for timeout-based key releases (fallback)
        self.input_state.check_timeouts();

        let now = Instant::now();

        // Handle left movement with proper DAS/ARR
        if let Some(left_state) = self.input_state.directions.get_mut(&InputDirection::Left) {
            if left_state.pressed {
                let mut should_move = false;
                
                if !left_state.initial_move_done {
                    // Initial move on key press
                    should_move = true;
                    left_state.initial_move_done = true;
                } else if !left_state.das_charged {
                    // Check if DAS delay has passed
                    if now.duration_since(left_state.das_timer) >= Duration::from_millis(DAS_DELAY) {
                        left_state.das_charged = true;
                        left_state.arr_timer = now;
                        should_move = true;
                    }
                } else {
                    // ARR phase - move at regular intervals
                    if now.duration_since(left_state.arr_timer) >= Duration::from_millis(ARR_DELAY) {
                        left_state.arr_timer = now;
                        should_move = true;
                    }
                }
                
                if should_move {
                    self.move_piece(-1, 0);
                }
            }
        }

        // Handle right movement with proper DAS/ARR
        if let Some(right_state) = self.input_state.directions.get_mut(&InputDirection::Right) {
            if right_state.pressed {
                let mut should_move = false;
                
                if !right_state.initial_move_done {
                    // Initial move on key press
                    should_move = true;
                    right_state.initial_move_done = true;
                } else if !right_state.das_charged {
                    // Check if DAS delay has passed
                    if now.duration_since(right_state.das_timer) >= Duration::from_millis(DAS_DELAY) {
                        right_state.das_charged = true;
                        right_state.arr_timer = now;
                        should_move = true;
                    }
                } else {
                    // ARR phase - move at regular intervals
                    if now.duration_since(right_state.arr_timer) >= Duration::from_millis(ARR_DELAY) {
                        right_state.arr_timer = now;
                        should_move = true;
                    }
                }
                
                if should_move {
                    self.move_piece(1, 0);
                }
            }
        }

        // Handle soft drop (down movement)
        if let Some(down_state) = self.input_state.directions.get_mut(&InputDirection::Down) {
            if down_state.pressed {
                let mut should_move = false;
                
                if !down_state.initial_move_done {
                    // Initial move on key press
                    should_move = true;
                    down_state.initial_move_done = true;
                } else if now.duration_since(down_state.arr_timer) >= Duration::from_millis(SOFT_DROP_DELAY) {
                    down_state.arr_timer = now;
                    should_move = true;
                }
                
                if should_move {
                    if !self.move_piece(0, 1) {
                        self.lock_piece();
                    }
                }
            }
        }

        // Handle gravity drop
        if now.duration_since(self.drop_timer) >= self.get_drop_delay() {
            self.drop_timer = now;
            if !self.move_piece(0, 1) {
                self.lock_piece();
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key_code: KeyCode, kind: KeyEventKind) {
        match kind {
            KeyEventKind::Press | KeyEventKind::Repeat => {
                match key_code {
                    KeyCode::Left => {
                        if !self.input_state.is_pressed(InputDirection::Left) {
                            self.input_state.press_direction(InputDirection::Left);
                        } else {
                            // Update activity for repeat events
                            self.input_state.update_key_activity(InputDirection::Left);
                        }
                    }
                    KeyCode::Right => {
                        if !self.input_state.is_pressed(InputDirection::Right) {
                            self.input_state.press_direction(InputDirection::Right);
                        } else {
                            // Update activity for repeat events
                            self.input_state.update_key_activity(InputDirection::Right);
                        }
                    }
                    KeyCode::Down => {
                        if !self.input_state.is_pressed(InputDirection::Down) {
                            self.input_state.press_direction(InputDirection::Down);
                        } else {
                            // Update activity for repeat events
                            self.input_state.update_key_activity(InputDirection::Down);
                        }
                    }
                    KeyCode::Up => {
                        self.rotate_piece();
                    }
                    KeyCode::Char(' ') => {
                        self.hard_drop();
                    }
                    _ => {}
                }
            }
            KeyEventKind::Release => {
                match key_code {
                    KeyCode::Left => {
                        self.input_state.release_direction(InputDirection::Left);
                    }
                    KeyCode::Right => {
                        self.input_state.release_direction(InputDirection::Right);
                    }
                    KeyCode::Down => {
                        self.input_state.release_direction(InputDirection::Down);
                    }
                    _ => {}
                }
            }
        }
    }

    fn reset(&mut self) {
        self.board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        self.current_piece = None;
        self.next_piece = Self::random_piece();
        self.score = 0;
        self.lines_cleared = 0;
        self.level = 1;
        self.drop_timer = Instant::now();
        self.input_state = InputState::new();
        self.game_over = false;
        self.spawn_piece();
    }
}

fn ui(f: &mut Frame, game: &Game) {
    let size = f.size();
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(20)].as_ref())
        .split(size);
    
    // Game board area
    let board_area = chunks[0];
    let info_area = chunks[1];
    
    // Create board with current piece
    let mut render_board = game.board;
    if let Some(ref piece) = game.current_piece {
        for (x, y) in piece.get_blocks() {
            if x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32 {
                render_board[y as usize][x as usize] = Cell::Filled(piece.color);
            }
        }
    }
    
    // Build the game board display
    let mut board_lines = Vec::new();
    
    for y in 0..BOARD_HEIGHT {
        let mut line_spans = Vec::new();
        for x in 0..BOARD_WIDTH {
            match render_board[y][x] {
                Cell::Empty => {
                    line_spans.push(Span::styled("  ", Style::default()));
                }
                Cell::Filled(color) => {
                    line_spans.push(Span::styled("██", Style::default().fg(color)));
                }
            }
        }
        board_lines.push(Line::from(line_spans));
    }
    
    let board_widget = Paragraph::new(board_lines)
        .block(Block::default()
               .borders(Borders::ALL)
               .title("Jstris Clone"));
    
    f.render_widget(board_widget, board_area);
    
    // Info panel layout
    let info_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Stats
            Constraint::Length(8),  // Next piece
            Constraint::Length(10), // Controls
            Constraint::Min(5),     // Debug info
        ].as_ref())
        .split(info_area);
    
    // Stats panel
    let stats_text = vec![
        Line::from(vec![Span::raw(format!("Score: {}", game.score))]),
        Line::from(vec![Span::raw(format!("Lines: {}", game.lines_cleared))]),
        Line::from(vec![Span::raw(format!("Level: {}", game.level))]),
    ];
    
    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Stats"))
        .alignment(Alignment::Left);
    
    f.render_widget(stats_widget, info_chunks[0]);
    
    // Next piece panel
    let mut next_lines = Vec::new();
    for i in 0..4 {
        let mut line_spans = Vec::new();
        if i < game.next_piece.shape.len() {
            for j in 0..4 {
                if j < game.next_piece.shape[i].len() && game.next_piece.shape[i][j] {
                    line_spans.push(Span::styled("██", Style::default().fg(game.next_piece.color)));
                } else {
                    line_spans.push(Span::raw("  "));
                }
            }
        } else {
            line_spans.push(Span::raw("        "));
        }
        next_lines.push(Line::from(line_spans));
    }
    
    let next_widget = Paragraph::new(next_lines)
        .block(Block::default().borders(Borders::ALL).title("Next"))
        .alignment(Alignment::Center);
    
    f.render_widget(next_widget, info_chunks[1]);
    
    // Controls panel
    let controls_text = vec![
        Line::from(vec![Span::raw("← → Move")]),
        Line::from(vec![Span::raw("↓ Soft drop")]),
        Line::from(vec![Span::raw("↑ Rotate")]),
        Line::from(vec![Span::raw("Space Hard drop")]),
        Line::from(vec![Span::raw("R Restart")]),
        Line::from(vec![Span::raw("Q Quit")]),
    ];
    
    let controls_widget = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(Alignment::Left);
    
    f.render_widget(controls_widget, info_chunks[2]);
    
    // Debug info panel
    let mut debug_lines = vec![];
    
    // Show input states
    let left_state = if game.input_state.is_pressed(InputDirection::Left) {
        let state = game.input_state.directions.get(&InputDirection::Left).unwrap();
        if state.das_charged { "ARR" } else { "DAS" }
    } else { "---" };
    
    let right_state = if game.input_state.is_pressed(InputDirection::Right) {
        let state = game.input_state.directions.get(&InputDirection::Right).unwrap();
        if state.das_charged { "ARR" } else { "DAS" }
    } else { "---" };
    
    let down_state = if game.input_state.is_pressed(InputDirection::Down) { "ON" } else { "OFF" };
    
    debug_lines.push(Line::from(vec![
        Span::raw("Left: "),
        Span::styled(left_state, if left_state != "---" { Style::default().fg(Color::Green) } else { Style::default() }),
    ]));
    debug_lines.push(Line::from(vec![
        Span::raw("Right: "),
        Span::styled(right_state, if right_state != "---" { Style::default().fg(Color::Green) } else { Style::default() }),
    ]));
    debug_lines.push(Line::from(vec![
        Span::raw("Down: "),
        Span::styled(down_state, if down_state == "ON" { Style::default().fg(Color::Green) } else { Style::default() }),
    ]));
    
    let enhancement_status = if game.input_state.keyboard_enhancement_active { "Active" } else { "Inactive" };
    debug_lines.push(Line::from(vec![Span::raw("")]));
    debug_lines.push(Line::from(vec![
        Span::raw("Key Release: "),
        Span::styled(enhancement_status, 
            if game.input_state.keyboard_enhancement_active { 
                Style::default().fg(Color::Green) 
            } else { 
                Style::default().fg(Color::Yellow) 
            }),
    ]));
    
    let debug_widget = Paragraph::new(debug_lines)
        .block(Block::default().borders(Borders::ALL).title("Input Debug"))
        .alignment(Alignment::Left);
    
    f.render_widget(debug_widget, info_chunks[3]);
    
    // Game over overlay
    if game.game_over {
        let popup_area = centered_rect(50, 30, size);
        f.render_widget(Clear, popup_area);
        
        let game_over_text = vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled("GAME OVER!", Style::default().fg(Color::Red))]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw(format!("Final Score: {}", game.score))]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::raw("Press R to restart")]),
            Line::from(vec![Span::raw("Press Q to quit")]),
        ];
        
        let game_over_widget = Paragraph::new(game_over_text)
            .block(Block::default().borders(Borders::ALL).title("Game Over"))
            .alignment(Alignment::Center);
            
        f.render_widget(game_over_widget, popup_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // Try to enable keyboard enhancement for better key release detection
    let keyboard_enhancement_active = matches!(
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        ),
        Ok(())
    );
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut game = Game::new();
    game.input_state.keyboard_enhancement_active = keyboard_enhancement_active;
    
    // Game loop
    loop {
        // Render
        terminal.draw(|f| ui(f, &game))?;
        
        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(KeyEvent { code, kind, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if kind == KeyEventKind::Press {
                            break;
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if kind == KeyEventKind::Press && game.game_over {
                            game.reset();
                            game.input_state.keyboard_enhancement_active = keyboard_enhancement_active;
                        }
                    }
                    _ => {
                        if !game.game_over {
                            game.handle_input(code, kind);
                        }
                    }
                }
            }
        }
        
        // Update game state
        game.update()?;
    }

    // Cleanup
    if keyboard_enhancement_active {
        execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
    }
    execute!(terminal.backend_mut(), DisableMouseCapture)?;
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    Ok(())
}