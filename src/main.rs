#![warn(clippy::unwrap_used)]

use std::{
    thread,
    time::{Duration, Instant},
    sync::mpsc,
};
use ratatui::{
    widgets::*,
    layout::{Layout, Constraint, Direction},
    style::{Color, Style, Modifier},
    backend::CrosstermBackend,
    Terminal,
    text::Line,
};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use chatgpt::prelude::{ ChatGPT, ChatMessage, Conversation };
use chatgpt::types::Role;

mod app;
use app::{AppState, EditingMode};
mod io;
use io::config::Config;

enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = Config::load();
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

    // FOR TESTING PURPOSES
    let client: ChatGPT = ChatGPT::new(config.api_key().unwrap())?;
    let mut conversation: Conversation = Conversation::new_with_history(client, vec![]);

    // Render loop
    loop {
        // Draw terminal
        terminal.draw(|f| {
            // Sizes
            // FIXME: program panics when sizes don't fit
            let size = f.size();
            let inner_width: u16 = f.size().width - 2;
            let input_box_height: u16 = app_state.input_text().len() as u16 / inner_width + 3;
            let help_box_height: u16 = 3;
            let history_box_height: u16 = f.size().height - input_box_height - help_box_height;

            // Layout
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(history_box_height),
                    Constraint::Length(input_box_height),
                    Constraint::Length(help_box_height),
                ])
                .split(size);

            // Set cursor to correct position
            let cursor_line = app_state.cursor_position() / inner_width;
            let cursor_column = app_state.cursor_position() % inner_width;
            f.set_cursor(cursor_column + 1, layout[1].y + 1 + cursor_line);

            // Display message history
            // TODO: make user messages italic and assistant messages regular
            // FIXME: wrap messages
            let messages: Vec<ListItem> = app_state.history().iter()
                .map(|msg| {
                    ListItem::new(msg.as_str())
                })
                .collect();
            let history_box = List::new(messages)
                .block(Block::default().title("History").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol("> ");
            // TODO: keep history_state in app_state?
            let mut history_state = ListState::default();
            history_state.select(app_state.selected_message());
            f.render_stateful_widget(history_box, layout[0], &mut history_state);

            // Input and help box, depending on editing mode
            match app_state.editing_mode() {
                EditingMode::Normal => {
                    let lines: Vec<Line> = app_state.input_text().chars()
                        .collect::<Vec<char>>()
                        .chunks(inner_width as usize)
                        .map(|chunk| Line::from(chunk.iter().collect::<String>()))
                        .collect();

                    let input_box = Paragraph::new(lines)
                        .wrap(Wrap{ trim: false })
                        .style(Style::default())
                        .block(Block::default().borders(Borders::ALL).title("Input"));
                    f.render_widget(input_box, layout[1]);

                    let help_box = Paragraph::new("<q: quit> <i: insert mode>")
                        .block(Block::default().borders(Borders::ALL).title("Help"));
                    f.render_widget(help_box, layout[2]);
                },
                EditingMode::Insert => {
                    let lines: Vec<Line> = app_state.input_text().chars()
                        .collect::<Vec<char>>()
                        .chunks(inner_width as usize)
                        .map(|chunk| Line::from(chunk.iter().collect::<String>()))
                        .collect();

                    let input_box = Paragraph::new(lines)
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
                        // Append with 'a'
                        // Remove character under cursor
                        KeyCode::Char('x') => {
                            app_state.remove_char();
                        },
                        // Navigate up/down through history
                        KeyCode::Char('j') => {
                            app_state.select_next_msg();
                        },
                        KeyCode::Char('k') => {
                            app_state.select_prev_msg();
                        },
                        // Navigate cursor left/right
                        KeyCode::Char('h') => {
                            app_state.move_cursor_left();
                        },
                        KeyCode::Char('l') => {
                            app_state.move_cursor_right();
                        },
                        // Yank selected message
                        KeyCode::Char('y') => {
                            app_state.yank_selected();
                        },
                        // Paste buffer into input box, and switch editing mode
                        KeyCode::Char('p') => {
                            app_state.paste_buffer();
                            // app_state.switch_editing_mode(EditingMode::Insert);
                        },
                        // TODO: Rollback history
                        KeyCode::Char('u') => {},
                        // Deselect message
                        KeyCode::Esc => {
                            app_state.clear_msg_selection();
                        },
                        // TODO: Send message
                        KeyCode::Enter => {
                            // TODO: queue up the API call to happen on next frame draw, otherwise cannot show loading icon
                            // Generally, how do I properly use async/await? Right now, I'm just blocking the main thread anyway
                            app_state.append_history(String::from(app_state.input_text()));
                            app_state.clear_input_text();
                            let response = conversation.send_message(app_state.input_text()).await.expect("ChatGPT error");
                            app_state.append_history(response.message().content.clone());
                        }
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
                        // Remove letter before cursor
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
                            app_state.switch_editing_mode(EditingMode::Normal);
                            app_state.append_history(String::from(app_state.input_text()));
                            app_state.clear_input_text();
                            let response = conversation.send_message(app_state.input_text()).await.expect("ChatGPT error");
                            app_state.append_history(response.message().content.clone());
                        }
                        _ => {},
                    },
                    Event::Tick => {},
                }
            },
            _ => {},
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
