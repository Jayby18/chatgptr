#![warn(clippy::unwrap_used)]

use std::env;
use serde::Deserialize;

mod io;
use io::config::Config;

#[derive(Debug, Deserialize)]
struct CompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<CompletionChoice>,
    usage: CompletionUsage,
}

#[derive(Debug, Deserialize)]
struct CompletionChoice {
    index: u64,
    message: CompletionMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct CompletionMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct CompletionUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command>", args[0]);
        return
    }

    let command = &args[1];

    let config: Config = Config::load();

    match command.as_str() {
        "ask" => {
            if args.len() < 3 {
                println!("Usage: {} ask <question>", args[0]);
                return
            } else {
                let question = &args[2];
                ask(question, &config);
            }
        },
        "help" => println!("Usage: {} <command> <question>", args[0]),
        "version" => println!("chatgptr v0.1.0"),
        "author" => println!("chatgptr is written by Jaden Accord <dev@jadenaccord.com>"),
        // "config" => println!("{:?}\n\nLocated at $XDG_CONFIG_HOME/chatgptr/chatgptr.toml", config),
        "config" => println!("$XDG_CONFIG_HOME/chatgptr/chatgptr.toml"),
        _ => println!("Unknown command: {}", command),
    }
}

fn ask(question: &str, config: &Config) {
    // TODO: Send question to OpenAI API
    let client = reqwest::blocking::Client::new();
    // let body = format!("{{\n\"model\": \"{}\",\n\"messages\": [\n{{\"role: \":\"system\",\n\"content\": \"You are a helpful assistant.\"\n}},{{\"role\": \"user\",\n\"content\": \"{}\"\n}}\n]\n}}", config.model().unwrap_or(String::from("gpt-3.5-turbo")), question);
    let body = format!("{{\n\"model\": \"{}\",\n\"messages\": [\n{{\"role\": \"user\",\n\"content\": \"{}\"\n}}\n]\n}}", config.model().unwrap_or(String::from("gpt-3.5-turbo")), question);

    // Send request using reqwest
    let res = client.post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .bearer_auth(config.api_key().unwrap())
        .body(body)
        .send();

    // Parse response to CompletionResponse struct
    let result: CompletionResponse = serde_json::from_str(res.unwrap().text().unwrap().as_str()).unwrap();
    println!("{} ({} tokens)", result.choices[0].message.content, result.usage.total_tokens);
}
