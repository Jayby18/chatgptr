#![warn(clippy::unwrap_used)]

use ratatui::{
    widgets::*,
    layout::{Layout, Constraint, Direction},
    style::{Color, Style},
};
use crossterm::{
    event::KeyCode,
    execute,
};

mod io;
use io::config::Config;

mod app;
use app::{AppState, EditingMode};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = Config::load()?;
    let mut state: AppState = AppState::default();
    
    // Set up terminal
    let (mut terminal, rx) = stdr::setup_terminal!();

    // Render loop
    loop {
        // Draw terminal
        terminal.draw(|f| {
            // Layout
            let size = f.size();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(20),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            // Update input box
            if state.editing_mode() == EditingMode::Insert {
                let text_box = Paragraph::new(state.input_text().clone())
                    .wrap(Wrap { trim: true })
                    .style(
                        Style::default()
                            .fg(Color::Red)
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Send a message")
                    );
                f.render_widget(text_box, layout[1]);

                // Help
                let help = Paragraph::new("<esc: exit insert editing_mode> <enter: send message>")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Help")
                    );
                f.render_widget(help, layout[2]);
            } else if state.editing_mode() == EditingMode::Normal {
                let text_box = Paragraph::new(state.input_text().clone())
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Send a message")
                    );
                f.render_widget(text_box, layout[1]);

                // Help
                let help = Paragraph::new("<q: quit> <i: insert editing_mode>")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Help")
                    );
                f.render_widget(help, layout[2]);
            }
        })?;

        // Handle user event, depending on current editing editing_mode
        match state.editing_mode() {
            EditingMode::Normal => {
                match rx.recv().expect("recv error") {
                    Event::Input(event) => match event.code {
                        // Quit application
                        KeyCode::Char('q') => {
                            break;
                        },
                        // Switch editing modes
                        KeyCode::Char('i') => {
                            state.switch_editing_mode(EditingMode::Insert);
                        },
                        KeyCode::Char('v') => {
                            state.switch_editing_mode(EditingMode::Visual);
                        },
                        // TODO: Undo
                        KeyCode::Char('u') => {
                        },
                        // TODO: Remove character under cursor
                        KeyCode::Char('x') => {
                        },
                        // TODO: Navigate up/down through history
                        KeyCode::Char('j') => {
                        },
                        KeyCode::Char('k') => {
                        },
                        // Navigate cursor left/right
                        KeyCode::Char('h') => {
                            state.move_cursor_left();
                        },
                        KeyCode::Char('l') => {
                            state.move_cursor_right();
                        },
                        _ => {},
                    },
                    Event::Tick => {},
                }
            },
            EditingMode::Insert => {
                match rx.recv().expect("recv error") {
                    Event::Input(event) => match event.code {
                        // Insert letter
                        KeyCode::Char(c) => {
                            state.push_input_text(c);
                        },
                        // TODO: Remove letter before cursor
                        KeyCode::Backspace => {
                        },
                        // TODO: Remove letter under cursor
                        KeyCode::Delete => {}
                        // Escape back to normal mode
                        KeyCode::Esc => {
                            state.switch_editing_mode(EditingMode::Normal);
                        },
                        // TODO: Send message
                        KeyCode::Enter => {
                        }
                        _ => {},
                    },
                    Event::Tick => {},
                }
            },
            EditingMode::Visual => {
                match rx.recv().expect("recv error") {
                    Event::Input(event) => match event.code {
                        KeyCode::Esc => {
                            state.switch_editing_mode(EditingMode::Normal);
                        },
                        _ => {},
                    },
                    Event::Tick => {},
                }
            },
        }
    }

    // Restore terminal
    stdr::restore_terminal!(terminal);

    Ok(())
}
