use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

const REL_CONFIG_PATH: &str = ".config/chatgptr/chatgptr.conf";

pub struct Config {
    token: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            token: String::new(),
        }
    }

    pub fn load() -> Result<Config, std::io::Error> {
        let path = dirs::home_dir().expect("no home dir").join(Path::new(REL_CONFIG_PATH));

        if let Ok(file) = File::open(path) {
            let mut config = Config::new();

            let reader = BufReader::new(file);
            reader
                .lines()
                .map_while(Result::ok)
                .filter(|line| line.contains('='))
                .for_each(|line| {
                    let key = line.split('=')
                        .next()
                        .unwrap_or("");
                    let value = line.split('=')
                        .nth(1)
                        .map(String::from)
                        .unwrap_or(String::new());
                    match key {
                        "token" => config.token = value,
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

    pub fn token(&self) -> String { self.token.clone() }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}