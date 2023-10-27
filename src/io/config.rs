use toml::Table;

#[derive(Clone, Default)]
pub struct Config {
    api_key: Option<String>,
}

impl Config {
    pub fn new(api_key: String) -> Self {
        Config {
            api_key: Some(api_key),
        }
    }

    // pub fn load() -> Self {
    // }

    // pub fn from_str(text: &str) -> Self {
    // }
}
