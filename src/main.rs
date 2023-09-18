use std::{
    fs::File,
    io::{
        self,
        BufRead,
        BufReader,
        Read,
    },
    path::Path,
    thread,
    time::{Duration, Instant},
    sync::mpsc,
};
use dirs;
use reqwest::{
    self,
    header::*,
};
use serde_json::json;
use ratatui::{
    backend::CrosstermBackend,
    widgets::*,
    layout::{Layout, Constraint, Direction},
    style::{Color, Style},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde_derive::Deserialize;

const REL_CONFIG_PATH: &str = ".config/chatgptr/chatgptr.conf";

enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let mut state = AppState::new();
    let mut message: String = String::new();
    
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // User event handler
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move | | {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(| | Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

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

            if !message.is_empty() {
                let gpt_response: GPTResponse = serde_json::from_str(message.as_str()).unwrap();

                let chat_box = Paragraph::new(gpt_response.content)
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
                            message = send_message(&state.user_text, &config.token).await?;
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
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    // let mut message: String = String::new();
    // println!("\n Message:");
    // io::stdin().read_line(&mut message).expect("failed to readline");

    

    Ok(())
}

async fn send_message(content: &String, token: &String) -> Result<String, Box<dyn std::error::Error>> {
    let payload = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{
            "role": "user",
            "content": content,
        }],
        "temperature" : 0.7,
    });

    println!("Content: {}", content);

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .body(payload.to_string())
        .send()
        .await?
        .text()
        .await?;

    Ok(String::from(response.as_str()))
}

enum GPTModel {
    V3_5,
    V4,
}

struct Config {
    token: String,
    model: GPTModel,
}

impl Config {
    fn new() -> Config {
        Config {
            token: String::new(),
            model: GPTModel::V3_5,
        }
    }

    fn load() -> Result<Config, io::Error> {
        let path = dirs::home_dir().unwrap().join(Path::new(REL_CONFIG_PATH));

        if let Ok(file) = File::open(path) {
            let mut config = Config::new();

            // let token_reader = BufReader::new(&file);

            // let test_token = token_reader
            //     .lines()
            //     .find(|line| {
            //         line
            //             .as_ref()
            //             .unwrap()
            //             .starts_with("token=")
            //     })
            //     .unwrap()?
            //     .split("=")
            //     .map(String::from)
            //     .skip(1)
            //     .next()
            //     .unwrap();

            let reader = BufReader::new(file);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .filter(|line| line.contains("="))
                .for_each(|line| {
                    let key = line.split("=").next().unwrap();
                    let value = line.split("=").skip(1).next().map(String::from).unwrap();
                    match key {
                        "token" => config.token = value,
                        "model" => config.model = match value.as_str() {
                            "gpt-3.5-turbo" => GPTModel::V3_5,
                            "gpt-4" => GPTModel::V4,
                            _ => GPTModel::V3_5,
                        },
                        _ => {}
                    }
                });

            Ok(config)
        } else {
            // file doesn't exist yet
            // TODO: prompt to create new config
            panic!("Config file not found!");
        }
    }
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

#[derive(Deserialize)]
struct GPTResponse {
    content: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_config() {
        let config = Config::load();
    }
}

