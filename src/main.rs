mod vk_api;
mod ui;
mod config;
mod navigation;
mod client;
mod test_client;

use std::io;
use crossterm::event::{self, Event};
use vk_api::VkClient;
use ratatui::Terminal;
use config::{Config, load_config};
use client::{Client};
use test_client::TestClient;

const MIN_SIZE: (u16, u16) = (80, 23);


#[tokio::main]
async fn main() -> io::Result<()> {
    let conf: Config = load_config();
    let client = Box::new(TestClient::new(conf.token.to_string()));
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
            app.handle_input(key.code).await;
        }
    }

    // Returning to default terminal and closing the app
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
