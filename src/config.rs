use crossterm::event::KeyCode;
use dirs;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub token: String,
    pub min_width: u16,
    pub min_height: u16,
    pub keys: KeyBindings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyBindings{
    // Global 
    pub command: KeyCode,
    pub quit: KeyCode,

    // Normal Mode
    pub to_normal_mode: KeyCode,

    // Insert Mode
    pub go_back_word: KeyCode,
    pub go_forward_word: KeyCode,
    pub to_insert_mode: KeyCode,

    // Chat List
    pub enter_chat: KeyCode,
    pub enter_chat_secondary: KeyCode,
    pub move_up_list: KeyCode,
    pub move_down_list: KeyCode,

    // Chat View
    pub scroll_up_chat: KeyCode,
    pub scroll_down_chat: KeyCode,
    pub view_chat_list: KeyCode,
}


// Default config
impl Default for Config {
    fn default() -> Self {
        Self {
            token: generate_token(),
            min_width: 300,
            min_height: 300,
            keys: KeyBindings::default()
        }
    }
}


// Default keybinds
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            // Global 
            command: KeyCode::Char(':'),
            quit: KeyCode::Char('q'),

            // Normal Mode
            to_normal_mode: KeyCode::Esc,

            // Insert Mode
            go_back_word: KeyCode::Char('b'),
            go_forward_word: KeyCode::Char('w'),
            to_insert_mode: KeyCode::Char('i'),

            // Chat List
            enter_chat: KeyCode::Char('l'),
            enter_chat_secondary: KeyCode::Enter,
            move_up_list: KeyCode::Char('k'),
            move_down_list: KeyCode::Char('j'),

            // Chat View
            scroll_up_chat: KeyCode::Char('k'),
            scroll_down_chat: KeyCode::Char('j'),
            view_chat_list: KeyCode::Char('h'),
        }
    }
}


pub fn load_config() -> Config {
    // Extracting config struct from config file
    let mut config: Config;
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vk-rust-tui")
        .join("config.toml");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .expect("error w/ loading config.toml");
        config = toml::from_str(&content).unwrap_or_else(|_| Config::default());
    } else {
        config = Config::default();
    }

    // Token validation
    if "".to_string() == config.token {
        config.token = generate_token();
    }
    // TODO: Сделать валидацию для непустого токена через авторизатор
    config
}


pub fn generate_token() -> String {
    // TODO: Сделать авторизацию пользователя
    "".to_string()
}


