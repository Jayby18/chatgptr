use std::{
    fs::File,
    io::{
        self,
        BufRead,
        BufReader,
    },
    path::Path,
};

use dirs;

const REL_CONFIG_PATH: &str = ".config/chatgptr/chatgptr.conf";

fn main() -> Result<(), io::Error> {
    let config = Config::load()?;
    println!("Token: {}", config.token);

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
    fn load() -> Result<Config, io::Error> {
        let path = dirs::home_dir().unwrap().join(Path::new(REL_CONFIG_PATH));

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);

            let token = reader
                .lines()
                .find(|line| {
                    line
                        .as_ref()
                        .unwrap()
                        .starts_with("token=")
                })
                .unwrap()?
                .split("=")
                .map(String::from)
                .skip(1)
                .next()
                .unwrap();

            let config = Config {
                token,
                model: GPTModel::V4,
            };

            Ok(config)
        } else {
            // file doesn't exist yet
            // TODO: prompt to create new config
            panic!("Config file not found!");
        }
    }
}

