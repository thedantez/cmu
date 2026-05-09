mod vk_api;
mod ui;
mod config;

use std::io;
use crossterm::event::{self, Event};
use vk_api::VkClient;
use ratatui::Terminal;
use ui::Command;

const MIN_SIZE: (u16, u16) = (80, 23);

#[tokio::main]
async fn main() -> io::Result<()> {
    let token = "";
    let client = VkClient::new(token.to_string());
    let dialogs = client.get_dialogs().await
        .expect("Error loading dialogs: ");

    // Loading the app
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = ui::App::new(client, dialogs, MIN_SIZE);

    loop {
        terminal.draw(|f| app.render(f))?;
        if let Event::Key(key) = event::read()? {
            if let Some(cmd) = app.handle_input(key.code) {
                match cmd {
                    Command::LoadMessages(peer_id) => {
                        app.load_messages(peer_id).await;
                    }
                    Command::SendMessage(peer_id, text) => {
                        if let Err(e) = app.send_message(peer_id, &text).await {
                            eprintln!("Error sending a message: {}", e);
                        } else {
                            app.load_messages(peer_id).await;
                        }
                    }
                }
            }
            if !app.running { break; }
        }
    }

    // Returning to default terminal and exiting the app
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
