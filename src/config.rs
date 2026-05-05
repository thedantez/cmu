use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub token_file: String,
    pub min_width: u16,
    pub min_height: u16,
    pub keys: KeyBindings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyBindings {
    pub move_up: char,
    pub move_down: char,
    pub enter_chat: char,
    pub go_back: char,
    pub delete_char: char,
    pub command: char,
}



pub fn load_config() -> Config {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vk-rust-tui")
        .join("config.toml");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .except("error w/ load config.toml");
        toml::from_str(&content).unwrap_or_else(|_| Config::default())
    } else {
        Config::default()
    }
}
