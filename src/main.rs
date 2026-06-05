mod vk_api;
mod ui;
mod config;
mod navigation;
mod client;
// mod test_client;
mod auth;

use std::io;
use crossterm::event::{self, KeyEvent};
use vk_api::VkClient;
use ratatui::Terminal;
use config::{Config, load_config, save_config};
use client::{Client, Message};
use std::sync::mpsc;
use std::thread;
//use test_client::TestClient; //debug w/ test client
use ui::Command;

const MIN_SIZE: (u16, u16) = (80, 23);

// unification of keyboard events & network events
pub enum AppEvent {
    Input(KeyEvent),
    NewMessage(Message),
    Tick,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut conf: Config = load_config(); // load conf

    // Initializing a client
    let valid_token = match &conf.token {
        Some(token) => {
            auth::validate_token(token).await
        }
        None => false,
    };
    
    if !valid_token {
        println!("Authorization required");
        match auth::get_access_token().await {
            Ok(token) => {
                conf.token = Some(token);
                save_config(&conf);
            }
            Err(e) => {
                eprintln!("Authorization failed: {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, "Authorization failed"));
            }
        }
    }

    let token = conf.token.clone().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Token not found after authorization")
    })?;

    let client = match VkClient::new(token).await {
        Ok(c) => Box::new(c) as Box<dyn Client>,
        Err(e) => {
            eprintln!("Failed to initialize VK client: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, "Client initialization failed"));
        }
    };

    let dialogs = match client.get_dialogs().await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to load dialogs: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to load dialogs"));
        }
    };

    // Loading the app
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = ui::App::new(client, dialogs, MIN_SIZE, conf);

    let (tx, rx) = mpsc::channel(); // "pipe" as channel connection
    let tx_keys = tx.clone();
    thread::spawn(move || {
        loop {
            if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                let _ = tx_keys.send(AppEvent::Input(key));
            }
        }
    });

    let tx_tick = tx.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(std::time::Duration::from_secs(1));
            let _ = tx_tick.send(AppEvent::Tick);
        }
    });

    while app.running {
        terminal.draw(|f| app.render(f))?;
        if let Ok(event) = rx.recv() {
            match event {
                AppEvent::Input(key_event) => {
                    if let Some(cmd) = app.handle_input(key_event.code) {
                        match cmd {
                            Command::LoadMessages(peer_id) => {
                                app.load_messages(peer_id).await;
                            }
                            Command::SendMessage(peer_id, text) => {
                                app.send_message(peer_id, &text).await;
                            }
                        }
                    }
                }
                AppEvent::NewMessage(msg) => {
                    if let ui::Screen::ChatView { messages, .. } = &mut app.screen {
                        messages.push(msg);
                    }
                }
                AppEvent::Tick => {
                    let active_peer_id = match &app.screen {
                        ui::Screen::ChatView { peer_id, .. } => Some(*peer_id),
                        _ => None,
                    };
                    if let Some(id) = active_peer_id {
                        app.load_messages(id).await;
                    }
                }
            }
        }
    }

    // Returning to default terminal and closing the app
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
