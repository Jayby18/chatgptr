use serde::Deserialize;

use std::fs::File;
use std::io::Read;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    api_key: Option<String>,
    model: Option<String>,
}

impl Config {
    pub fn new(api_key: String, model: String) -> Self {
        Config {
            api_key: Some(api_key),
            model: Some(model),
        }
    }

    pub fn load() -> Self {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("chatgptr").unwrap();
        let path = xdg_dirs.find_config_file("config.toml").expect("config.toml not found! please read the docs");
        let mut file = File::open(path).expect("could not open config.toml");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("could not read config.toml");

        toml::from_str(&content).expect("could not parse config.toml")
    }

    pub fn api_key(&self) -> Option<String> { self.api_key.clone() }
    pub fn model(&self) -> Option<String> { self.model.clone() }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: None,
            model: None,
        }
    }
}
