use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::game::{Game, Cell};
use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT};

pub fn ui(f: &mut Frame, game: &Game) {
    let size = f.size();
    
    // Calculate center position for the game board
    let board_height = 22; // 20 rows + 2 borders
    let board_width = 22;  // 20 cols (2 chars per block) + 2 borders
    
    // Create a centered layout
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),          // Flexible top space
            Constraint::Length(board_height), // Game board height
            Constraint::Min(1),          // Flexible bottom space
        ])
        .split(size);
    
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),          // Left margin
            Constraint::Length(15),      // Left info panel
            Constraint::Length(board_width), // Game board
            Constraint::Length(15),      // Right info panel  
            Constraint::Min(1),          // Right margin
        ])
        .split(vertical_chunks[1]);
    
    let left_info_area = horizontal_chunks[1];
    let board_area = horizontal_chunks[2];
    let right_info_area = horizontal_chunks[3];
    
    // Render components
    render_board(f, game, board_area);
    render_left_info(f, game, left_info_area);
    render_right_info(f, game, right_info_area);
    
    // Render countdown or game state overlays
    match game.game_state {
        crate::game::state::GameState::Ready => {
            render_ready_overlay(f, board_area);
        }
        crate::game::state::GameState::Countdown(count) => {
            render_countdown_overlay(f, count, board_area);
        }
        crate::game::state::GameState::Finished => {
            render_finished_overlay(f, game, board_area);
        }
        _ => {}
    }
}

fn render_board(f: &mut Frame, game: &Game, area: Rect) {
    let mut render_board = game.board;
    
    // Render ghost piece first (so it appears behind the current piece)
    if let Some(ghost) = game.get_ghost_piece() {
        for (x, y) in ghost.get_blocks() {
            if x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32 {
                if render_board[y as usize][x as usize] == Cell::Empty {
                    render_board[y as usize][x as usize] = Cell::Ghost(ghost.color);
                }
            }
        }
    }
    
    // Render current piece on top
    if let Some(piece) = &game.current_piece {
        for (x, y) in piece.get_blocks() {
            if x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32 {
                render_board[y as usize][x as usize] = Cell::Filled(piece.color);
            }
        }
    }
    
    let mut board_lines = Vec::new();
    
    for y in 0..BOARD_HEIGHT {
        let mut line_spans = Vec::new();
        for x in 0..BOARD_WIDTH {
            match render_board[y][x] {
                Cell::Empty => {
                    // Restore checkerboard pattern
                    if (x + y) % 2 == 0 {
                        line_spans.push(Span::styled("░░", Style::default().fg(Color::DarkGray)));
                    } else {
                        line_spans.push(Span::styled("  ", Style::default()));
                    }
                }
                Cell::Filled(color) => {
                    line_spans.push(Span::styled("██", Style::default().fg(color)));
                }
                Cell::Ghost(color) => {
                    // Ghost piece with dimmed color and outline
                    line_spans.push(Span::styled("▒▒", Style::default().fg(color)));
                }
            }
        }
        board_lines.push(Line::from(line_spans));
    }
    
    let board_widget = Paragraph::new(board_lines)
        .block(Block::default()
               .borders(Borders::ALL)
               .title("tstris"));
    
    f.render_widget(board_widget, area);
}

fn render_left_info(f: &mut Frame, game: &Game, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Hold piece
            Constraint::Length(8),  // Stats
            Constraint::Min(1),     // Empty space
        ])
        .split(area);
    
    render_hold_piece(f, game, chunks[0]);
    render_stats(f, game, chunks[1]);
}

fn render_right_info(f: &mut Frame, game: &Game, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(16), // Next pieces
            Constraint::Min(1),     // Empty space
        ])
        .split(area);
    
    render_next_piece(f, game, chunks[0]);
}

fn render_stats(f: &mut Frame, game: &Game, area: Rect) {
    let time_text = if let Some(duration) = game.get_current_time() {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        "0.00s".to_string()
    };
    
    let stats_text = vec![
        Line::from(vec![Span::styled("40L", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(time_text)]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(format!("{}/40", game.lines_cleared))]),
    ];
    
    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    
    f.render_widget(stats_widget, area);
}

fn render_next_piece(f: &mut Frame, game: &Game, area: Rect) {
    let mut next_lines = Vec::new();
    
    // Show 5 next pieces compactly
    for (piece_idx, piece) in game.next_pieces.iter().take(5).enumerate() {
        // Find the bounding box of the piece shape
        let mut min_row = piece.shape.len();
        let mut max_row = 0;
        
        for (i, row) in piece.shape.iter().enumerate() {
            if row.iter().any(|&cell| cell) {
                min_row = min_row.min(i);
                max_row = max_row.max(i);
            }
        }
        
        // Render only the rows that contain blocks
        for i in min_row..=max_row {
            let mut line_spans = Vec::new();
            for j in 0..4 {
                if j < piece.shape[i].len() && piece.shape[i][j] {
                    line_spans.push(Span::styled("██", Style::default().fg(piece.color)));
                } else {
                    line_spans.push(Span::raw("  "));
                }
            }
            next_lines.push(Line::from(line_spans));
        }
        
        // Add spacing between pieces
        if piece_idx < 4 {
            next_lines.push(Line::from(vec![Span::raw("")]));
        }
    }
    
    let next_widget = Paragraph::new(next_lines)
        .block(Block::default().borders(Borders::ALL).title("Next"))
        .alignment(Alignment::Center);
    
    f.render_widget(next_widget, area);
}

fn render_hold_piece(f: &mut Frame, game: &Game, area: Rect) {
    let mut hold_lines = Vec::new();
    
    hold_lines.push(Line::from(vec![Span::raw("")])); // Padding
    
    if let Some(ref hold_piece) = game.hold_piece {
        // Find the bounding box of the piece shape
        let mut min_row = hold_piece.shape.len();
        let mut max_row = 0;
        
        for (i, row) in hold_piece.shape.iter().enumerate() {
            if row.iter().any(|&cell| cell) {
                min_row = min_row.min(i);
                max_row = max_row.max(i);
            }
        }
        
        // Render only the rows that contain blocks
        for i in min_row..=max_row {
            let mut line_spans = Vec::new();
            for j in 0..4 {
                if j < hold_piece.shape[i].len() && hold_piece.shape[i][j] {
                    let color = if game.can_hold { hold_piece.color } else { Color::DarkGray };
                    line_spans.push(Span::styled("██", Style::default().fg(color)));
                } else {
                    line_spans.push(Span::raw("  "));
                }
            }
            hold_lines.push(Line::from(line_spans));
        }
    } else {
        hold_lines.push(Line::from(vec![Span::raw("        ")]));
        hold_lines.push(Line::from(vec![Span::raw("        ")]));
    }
    
    let hold_widget = Paragraph::new(hold_lines)
        .block(Block::default().borders(Borders::ALL).title("Hold"))
        .alignment(Alignment::Center);
    
    f.render_widget(hold_widget, area);
}


// Old render_game_over function removed - replaced with render_finished_overlay

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

fn render_ready_overlay(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 35, area);
    f.render_widget(Clear, popup_area);
    
    let ready_text = vec![
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("40L SPRINT", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press SPACE to start")]),
        Line::from(vec![Span::raw("")]),
    ];
    
    let ready_widget = Paragraph::new(ready_text)
        .block(Block::default().borders(Borders::ALL).title("Ready"))
        .alignment(Alignment::Center);
        
    f.render_widget(ready_widget, popup_area);
}

fn render_countdown_overlay(f: &mut Frame, count: u32, area: Rect) {
    let popup_area = centered_rect(40, 20, area);
    f.render_widget(Clear, popup_area);
    
    let countdown_text = if count == 1 {
        "GO!"
    } else {
        match count {
            2 => "Ready", 
            _ => "",
        }
    };
    
    let color = if count == 1 { Color::Green } else { Color::Yellow };
    
    let text = vec![
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(countdown_text, Style::default().fg(color))]),
        Line::from(vec![Span::raw("")]),
    ];
    
    let countdown_widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
        
    f.render_widget(countdown_widget, popup_area);
}

fn render_finished_overlay(f: &mut Frame, game: &Game, area: Rect) {
    let popup_area = centered_rect(50, 40, area);
    f.render_widget(Clear, popup_area);
    
    let time_text = if let Some(duration) = game.final_time {
        format!("{:.3}s", duration.as_secs_f64())
    } else {
        "N/A".to_string()
    };
    
    let finished_text = vec![
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("40L COMPLETE!", Style::default().fg(Color::Green))]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(format!("Final Time: {}", time_text))]),
        Line::from(vec![Span::raw(format!("Lines Cleared: {}", game.lines_cleared))]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press R to restart")]),
        Line::from(vec![Span::raw("Press Q to quit")]),
    ];
    
    let finished_widget = Paragraph::new(finished_text)
        .block(Block::default().borders(Borders::ALL).title("Finished"))
        .alignment(Alignment::Center);
        
    f.render_widget(finished_widget, popup_area);
}