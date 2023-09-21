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
use chatgpt::{
    client::ChatGPT,
    types::Role,
};

use chatgptr::io::config::Config;

enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let mut state = AppState::new();

    let mut client = ChatGPT::new(config.token())?;
    let mut conversation = client.new_conversation();
    let mut conversation_index: usize = 0;
    
    // Set up terminal
    let (mut terminal, rx) = stdr::setup_terminal!();

    // Render loop
    loop {
        // Draw terminal
        terminal.draw(|f| {
            // Set size
            let size = f.size();
            let display_width;
            if (size.width / 2) % 2 == 0 {
                display_width = size.width / 2;
            } else {
                display_width = (size.width / 2) - 1;
            }
            let display_height = display_width / 256 * 240;

            // Divide screen into two halves, horizontally
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(20),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            if !&conversation.history.is_empty() {
                // let gpt_response: GPTResponse = serde_json::from_str(message.as_str()).unwrap();
                // TODO: get response

                let chat_box = Paragraph::new(conversation.history.iter().filter(|m| m.role == Role::Assistant).skip(conversation_index).next().unwrap().content.clone())
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("ChatGPT")
                    );
                f.render_widget(chat_box, layout[0]);
            } else {
                let chat_box = Paragraph::new("")
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("ChatGPT")
                    );
                f.render_widget(chat_box, layout[0]);
            }

            if state.mode == EditingMode::Input {
                let text_box = Paragraph::new(state.user_text.clone())
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
                let help = Paragraph::new("<esc: exit insert mode> <enter: send message>")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Help")
                    );
                f.render_widget(help, layout[2]);
            } else if state.mode == EditingMode::Browse {
                let text_box = Paragraph::new(state.user_text.clone())
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Send a message")
                    );
                f.render_widget(text_box, layout[1]);

                // Help
                let help = Paragraph::new("<q: quit> <i: insert mode>")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Help")
                    );
                f.render_widget(help, layout[2]);
            }
        })?;

        // Handle user event
        match state.mode {
            EditingMode::Browse => {
                match rx.recv().unwrap() {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('q') => {
                            break;
                        },
                        KeyCode::Char('i') => {
                            state.mode = EditingMode::Input;
                        },
                        KeyCode::Char('u') => {
                            conversation.rollback();
                        }
                        _ => {},
                    },
                    Event::Tick => {},
                }
            },
            EditingMode::Input => {
                match rx.recv().unwrap() {
                    Event::Input(event) => match event.code {
                        KeyCode::Char(c) => {
                            state.user_text.push(c);
                        },
                        KeyCode::Backspace => {
                            state.user_text.pop();
                        },
                        KeyCode::Esc => {
                            state.mode = EditingMode::Browse;
                        },
                        KeyCode::Enter => {
                            state.mode = EditingMode::Browse;
                            conversation.send_message(state.user_text.clone()).await?;
                            state.user_text.clear();
                        }
                        _ => {},
                    },
                    Event::Tick => {},
                }
            }
        }
    }

    // Restore terminal
    stdr::restore_terminal!(terminal);

    Ok(())
}

#[derive(PartialEq)]
enum EditingMode {
    Input,
    Browse,
}

struct AppState {
    mode: EditingMode,
    user_text: String,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            mode: EditingMode::Browse,
            user_text: String::new(),
        }
    }
}
