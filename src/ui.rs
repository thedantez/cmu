use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Stylize},
    symbols,
    Frame,
    layout::{Layout, Constraint, Direction}
};
use crate::vk_api::{Dialog, Message, VkClient};

pub enum Screen {
    ChatList {
        list_state: ListState,
    },
    ChatView {
        peer_id: i64,
        messages: Vec<Message>,
        input: String,
        scroll: usize,
    },
}

#[derive(Debug)]
pub enum Command {
    LoadMessages(i64),
}

pub struct App {
    pub screen: Screen,
    pub dialogs: Vec<Dialog>,
    pub min_size: (u16, u16),
    pub client: VkClient,
    pub running: bool,
}

impl App {
    pub fn new(client: VkClient, dialogs: Vec<Dialog>, min_size: (u16, u16)) -> Self {
        let mut list_state = ListState::default();
        if !dialogs.is_empty() {
            list_state.select(Some(0));
        }
        App {
            screen: Screen::ChatList { list_state },
            dialogs,
            min_size,
            client,
            running: true,
        }
    }

    pub async fn load_messages(&mut self, peer_id: i64) {
        if let Ok(messages) = self.client.get_messages(peer_id, 20).await {
            if let Screen::ChatView { messages: msg_vec, .. } = &mut self.screen {
                *msg_vec = messages;
            }
        }
    }

    // print ui
    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        // check min size
        if area.width < self.min_size.0 || area.height < self.min_size.1 {
            let message = format!(
                "minial size: {}x{}; now: {}x{}",
                self.min_size.0, self.min_size.1, area.width, area.height
            );
            let paragraph = Paragraph::new(message)
                .block(Block::default().borders(Borders::ALL))
                .centered();
            f.render_widget(paragraph, area);
            return;
        }

        // split screen for left & right sides
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        self.render_left_panel(f, chunks[0]);
        self.render_right_panel(f, chunks[1]);
    }

    fn render_left_panel(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        // block w/ ASCII-print
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::Set {
                top_left: "+",
                top_right: "+",
                bottom_left: "+",
                bottom_right: "+",
                vertical_left: "|",
                vertical_right: "|",
                horizontal_top: "-",
                horizontal_bottom: "-",
            })
            .title(" chats ");

        match &mut self.screen {
            Screen::ChatList { list_state } => {
                let items: Vec<ListItem> = self.dialogs
                    .iter()
                    .map(|d| ListItem::new(d.title.clone()))
                    .collect();
                let list = List::new(items)
                    .block(block)
                    .highlight_style(Style::default().reversed());
                f.render_stateful_widget(list, area, list_state);
            }
            // _ => unreachable!(),
            _ => {
                let items: Vec<ListItem> = self.dialogs.iter()
                    .map(|d| ListItem::new(d.title.clone()))
                    .collect();
                let list = List::new(items).block(block);
                f.render_widget(list, area);
            }
        }
    }

    fn render_right_panel(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::Set {
                top_left: "+",
                top_right: "+",
                bottom_left: "+",
                bottom_right: "+",
                vertical_left: "|",
                vertical_right: "|",
                horizontal_top: "-",
                horizontal_bottom: "-",
            })
            .title(" messages ");

        match &self.screen {
            Screen::ChatList { .. } => {
                let paragraph = Paragraph::new("choose chat from left")
                    .block(block)
                    .centered();
                f.render_widget(paragraph, area);
            }
            Screen::ChatView { messages, input, .. } => {
                let msg_text: String = messages
                    .iter()
                    .map(|m| format!("{}: {}", m.sender_name, m.text))
                    .collect::<Vec<_>>()
                    .join("\n");
                let content = format!("{}\n\ninput: {}", msg_text, input);
                let paragraph = Paragraph::new(content).block(block);
                f.render_widget(paragraph, area);
            }
        }
    }

    // handle press keys; if app has to be end then return false
    pub fn handle_input(&mut self, key_code: crossterm::event::KeyCode) -> Option<Command> {
        match &mut self.screen {
            Screen::ChatList { list_state } => {
                let dialogs = &self.dialogs;
                match key_code {
                    crossterm::event::KeyCode::Char('j') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i >= dialogs.len() - 1 { 0 } else { i + 1 }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    crossterm::event::KeyCode::Char('k') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i == 0 { dialogs.len() - 1 } else { i - 1 }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    crossterm::event::KeyCode::Char('l') => {
                        if let Some(selected) = list_state.selected() {
                            if let Some(dialog) = dialogs.get(selected) {
                                self.screen = Screen::ChatView {
                                    peer_id: dialog.peer_id,
                                    messages: Vec::new(),
                                    input: String::new(),
                                    scroll: 0,
                                };
                                return Some(Command::LoadMessages(dialog.peer_id));
                            }
                        }
                        // None
                    }
                    crossterm::event::KeyCode::Char('q') =>  {
                        self.running = false;
                        return None;
                    }
                    _ => {}
                };
                None
            }
            Screen::ChatView { peer_id, messages, input, scroll } => {
                match key_code {
                    crossterm::event::KeyCode::Backspace => { input.pop(); },
                    crossterm::event::KeyCode::Char('h') => {
                        // returning into ChatList
                        self.screen = Screen::ChatList {
                            list_state: ListState::default(),
                        };
                    }
                    crossterm::event::KeyCode::Char('q') => {
                        self.running = false;
                        return None;
                    }
                    crossterm::event::KeyCode::Char(c) => { input.push(c); }
                    _ => {}
                };
                None
            }
        }
        // None
    }
}
