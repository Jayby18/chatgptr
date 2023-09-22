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

mod io;
use io::config::Config;

mod app;
use app::{AppState, EditingMode};

enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let mut state = AppState::new();

    let client = ChatGPT::new(config.token())?;
    let mut conversation = client.new_conversation();
    let mut msg_index: usize = 0;
    
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

            // if conversation.history.len() > 0 {
            //     let chat_box = Paragraph::new(
            //         conversation
            //             .history
            //             .iter()
            //             // .filter(|m| m.role == Role::Assistant)
            //             .nth(msg_index)
            //             .expect("no chat message")
            //             .content
            //             .clone())
            //         .wrap(Wrap { trim: true })
            //         .block(
            //             Block::default()
            //                 .borders(Borders::ALL)
            //                 .title("ChatGPT")
            //         );
            //     f.render_widget(chat_box, layout[0]);
            // } else {
            //     let chat_box = Paragraph::new("")
            //         .wrap(Wrap { trim: true })
            //         .block(
            //             Block::default()
            //                 .borders(Borders::ALL)
            //                 .title("ChatGPT")
            //         );
            //     f.render_widget(chat_box, layout[0]);
            // }

            // TODO: wrap text
            // TODO: add divider between messages
            let msg_list = List::new(
                    conversation.history
                    .iter()
                    .map(|msg| ListItem::new(msg.content.clone()))
                    .collect::<Vec<ListItem>>())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("ChatGPT")
                );
            f.render_widget(msg_list, layout[0]);

            // Update user input box
            if state.mode() == EditingMode::Insert {
                let text_box = Paragraph::new(state.user_text().clone())
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
            } else if state.mode() == EditingMode::Normal {
                let text_box = Paragraph::new(state.user_text().clone())
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

        // Handle user event, depending on current editing mode
        match state.mode() {
            EditingMode::Normal => {
                match rx.recv().expect("recv error") {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('q') => {
                            break;
                        },
                        KeyCode::Char('i') => {
                            state.switch_mode(EditingMode::Insert);
                        },
                        KeyCode::Char('u') => {
                            conversation.rollback();
                        },
                        KeyCode::Char('j') => {
                            if conversation.history.len() > msg_index + 1 { msg_index += 1; }
                        },
                        KeyCode::Char('k') => {
                            if msg_index != 0 { msg_index -= 1; }
                        },
                        _ => {},
                    },
                    Event::Tick => {},
                }
            },
            EditingMode::Insert => {
                match rx.recv().expect("recv error") {
                    Event::Input(event) => match event.code {
                        KeyCode::Char(c) => {
                            state.push_user_text(c);
                        },
                        KeyCode::Backspace => {
                            state.pop_user_text();
                        },
                        KeyCode::Esc => {
                            state.switch_mode(EditingMode::Normal);
                        },
                        KeyCode::Enter => {
                            state.switch_mode(EditingMode::Normal);
                            conversation.send_message(state.user_text()).await?;
                            msg_index += 2;
                            state.clear_user_text();
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
