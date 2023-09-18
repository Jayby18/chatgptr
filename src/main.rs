use std::{
    fs::File,
    io::{
        self,
        BufRead,
        BufReader,
        Read,
    },
    path::Path,
};
use dirs;
use reqwest::{
    self,
    header::*,
};
use serde_json::json;

const REL_CONFIG_PATH: &str = ".config/chatgptr/chatgptr.conf";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    println!("Token: {}", config.token);

    // let mut message: String = String::new();
    // println!("\n Message:");
    // io::stdin().read_line(&mut message).expect("failed to readline");

    let payload = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{
            "role": "user",
            "content": "What is Rustlang, in 1 sentence.",
        }],
        "temperature" : 0.7,
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", config.token))
        .body(payload.to_string())
        .send()
        .await?
        .text()
        .await?;
    println!("{:?}", response);

    Ok(())
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_config() {
        let config = Config::load();
    }
}

