#![warn(clippy::unwrap_used)]

use std::{
    thread,
    time::{Duration, Instant},
    sync::mpsc,
};
use ratatui::{
    widgets::*,
    layout::{Layout, Constraint, Direction},
    style::{Color, Style},
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

mod app;
use app::{AppState, EditingMode};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let config: Config = Config::load()?;
    let mut app_state: AppState = AppState::new();
    
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.show_cursor()?;

    // Set up event handler
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        // Set last_tick to current time
        let mut last_tick = Instant::now();
        loop {
            // timeout = tick_rate - time since last_tick (aka, time left before next tick)
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            // Check if there is an event to read
            if event::poll(timeout).expect("error polling crossterm events") {
                // Attempt to read event
                if let Ok(CEvent::Key(key)) = event::read() {
                    // Send event
                    tx.send(Event::Input(key)).expect("could not send input event to main thread");
                }
            }

            // Check if elapsed time is greater than tick rate
            if last_tick.elapsed() >= tick_rate {
                // Attempt to send Tick event
                if tx.send(Event::Tick).is_ok() {
                    // Reset last_tick to current time
                    last_tick = Instant::now();
                }
            }
        }
    });

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

            // Set cursor to correct position
            // TODO: how to deal with text wrapping?
            let input_box_width = f.size().width - 2;
            let cursor_line = app_state.cursor_position() / input_box_width;
            let cursor_column = app_state.cursor_position() % input_box_width;
            f.set_cursor(cursor_column + 1, layout[1].y + 1 + cursor_line);

            // Input and help box, depending on editing mode
            match app_state.editing_mode() {
                EditingMode::Normal => {
                    let input_box = Paragraph::new(app_state.input_text())
                        .wrap(Wrap{ trim: false })
                        .style(Style::default())
                        .block(Block::default().borders(Borders::ALL).title("Input"));
                    f.render_widget(input_box, layout[1]);

                    let help_box = Paragraph::new("<q: quit> <i: insert mode>")
                        .block(Block::default().borders(Borders::ALL).title("Help"));
                    f.render_widget(help_box, layout[2]);
                },
                EditingMode::Insert => {
                    let input_box = Paragraph::new(app_state.input_text())
                        .wrap(Wrap{ trim: false })
                        .style(Style::default().fg(Color::Red))
                        .block(Block::default().borders(Borders::ALL).title("Input"));
                    f.render_widget(input_box, layout[1]);

                    let help_box = Paragraph::new("<esc: normal mode>")
                        .block(Block::default().borders(Borders::ALL).title("Help"));
                    f.render_widget(help_box, layout[2]);
                },
                EditingMode::Visual => {},
            }
        })?;

        // Handle user event, depending on current editing editing_mode
        match app_state.editing_mode() {
            EditingMode::Normal => {
                match rx.recv().expect("recv error") {
                    Event::Input(event) => match event.code {
                        // Quit application
                        KeyCode::Char('q') => {
                            break;
                        },
                        // Switch editing modes
                        KeyCode::Char('i') => {
                            app_state.switch_editing_mode(EditingMode::Insert);
                        },
                        KeyCode::Char('v') => {
                            app_state.switch_editing_mode(EditingMode::Visual);
                        },
                        // Remove character under cursor
                        KeyCode::Char('x') => {
                            app_state.remove_char();
                        },
                        // TODO: Navigate up/down through history
                        KeyCode::Char('j') => {
                        },
                        KeyCode::Char('k') => {
                        },
                        // Navigate cursor left/right
                        KeyCode::Char('h') => {
                            app_state.move_cursor_left();
                        },
                        KeyCode::Char('l') => {
                            app_state.move_cursor_right();
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
                            app_state.insert_char(c);
                        },
                        // TODO: Remove letter before cursor
                        KeyCode::Backspace => {
                            app_state.backspace();
                        },
                        // Remove letter under cursor
                        KeyCode::Delete => {
                            app_state.remove_char();
                        }
                        KeyCode::Esc => {
                            app_state.switch_editing_mode(EditingMode::Normal);
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
                            app_state.switch_editing_mode(EditingMode::Normal);
                        },
                        _ => {},
                    },
                    Event::Tick => {},
                }
            },
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
