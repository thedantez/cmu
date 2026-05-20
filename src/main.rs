mod vk_api;
mod ui;
mod config;
mod navigation;
mod client;
mod test_client;
mod auth;

use std::io;
use crossterm::event::{self, Event};
use vk_api::VkClient;
use ratatui::Terminal;
use config::{Config, load_config, save_config};
use client::{Client};
//use test_client::TestClient;
use ui::Command;

const MIN_SIZE: (u16, u16) = (80, 23);


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
        println!("Needs authorization");
        let token = auth::get_access_token().await.expect("Error: Authorization failed!");
        conf.token = Some(token);
        save_config(&conf);
    }
    //let client = Box::new(TestClient::new(conf.token.to_string()));
    let token = conf.token.clone().unwrap();
    let client = Box::new(VkClient::new(token));
    let dialogs = client.get_dialogs().await
        .expect("Error loading dialogs: ");

    // Loading the app
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = ui::App::new(client, dialogs, MIN_SIZE, conf);

    while app.running {
        terminal.draw(|f| app.render(f))?;
        // Processing key input
        if let Event::Key(key) = event::read()? {
            if let Some(cmd) = app.handle_input(key.code) {
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
    }

    // Returning to default terminal and closing the app
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
