mod vk_api;
mod ui;
//mod config;

use std::io;
use crossterm::event::{self, Event, KeyCode};
use vk_api::VkClient;
use ratatui::Terminal;
use ui::Command;

const MIN_SIZE: (u16, u16) = (80, 30);

#[tokio::main]
async fn main() -> io::Result<()> {
    let token = "vk1.a.1wOvxYOFAFijemO8VNUkDl1SjwPagbbzz71kSvtdNQmimc8GFEZIAwHt1Lgkwdz72gR6rLgpqyalY0UUcAUz6hGU5-8bGFjkMGhCSEIvhx9rvoR1SPuq51Br02lmGE9LDZ25Vxb5GOgRfucpMiRsd2QK2_Iy0shPZfPgwJOEDzSMJGaovLKZ_JmHwJeBvcXpK3xAwPx-iy3I_1P6MF6gjw";
    let client = VkClient::new(token.to_string());
    let dialogs = client.get_dialogs().await
        .expect("error in loading dialogs");

    // setting terminal
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = ui::App::new(client, dialogs, MIN_SIZE);

    loop {
        terminal.draw(|f| app.render(f))?;
        if let Event::Key(key) = event::read()? {
            // if !app.handle_input(key.code) {
            //     break;
            // }
            if let Some(cmd) = app.handle_input(key.code) {
                match cmd {
                    Command::LoadMessages(peer_id) => {
                        app.load_messages(peer_id).await;
                    }
                }
            }
            if !app.running { break; }
        }
    }

    // returning to default terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
