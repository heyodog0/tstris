use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    io::stdout,
    time::Duration,
};

mod constants;
mod game;
mod input;
mod ui;

use game::Game;
use input::handle_input;
use ui::ui;

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
            if let Event::Key(KeyEvent { code, kind, modifiers, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if kind == KeyEventKind::Press {
                            break;
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if kind == KeyEventKind::Press {
                            game.reset();
                            game.input_state.keyboard_enhancement_active = keyboard_enhancement_active;
                        }
                    }
                    _ => {
                        handle_input(&mut game, code, kind, modifiers);
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