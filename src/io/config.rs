use serde::Deserialize;

use std::fs::File;
use std::io::Read;

#[derive(Clone, Default, Deserialize)]
pub struct Config {
    api_key: Option<String>,
}

impl Config {
    pub fn new(api_key: String) -> Self {
        Config {
            api_key: Some(api_key),
        }
    }

    pub fn load() -> Self {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("chatgptr").unwrap();
        // let path = xdg_dirs.find_config_file("config.toml").unwrap_or_else(|| xdg_dirs.place_config_file("config.toml").expect("could not create config file"));
        let path = xdg_dirs.find_config_file("config.toml").expect("config.toml does not exist");
        let mut file = File::open(path).expect("could not open config.toml");   // TODO: unwrap_or_else() instead of unwrap()
        let mut content = String::new();
        file.read_to_string(&mut content).expect("could not read config.toml");

        toml::from_str(&content).unwrap()
    }

    pub fn api_key(&self) -> Option<String> { self.api_key.clone() }
}
