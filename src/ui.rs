use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Stylize, Color},
    symbols,
    Frame,
    layout::{Layout, Constraint, Direction, Alignment}
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
    SendMessage(i64, String), // peer_id, text
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct App {
    pub screen: Screen,
    pub dialogs: Vec<Dialog>,
    pub min_size: (u16, u16),
    pub client: VkClient,
    pub running: bool,
    pub mode: Mode,
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
            mode: Mode::Normal,
        }
    }

    pub async fn load_messages(&mut self, peer_id: i64) {
        if let Ok(messages) = self.client.get_messages(peer_id, 20).await {
            if let Screen::ChatView { messages: msg_vec, .. } = &mut self.screen {
                *msg_vec = messages;
            }
        }
    }

    pub async fn send_message(&mut self, peer_id: i64, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client.send_message(peer_id, text).await
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

        // split screen for mode, left & right sides
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        self.render_left_panel(f, horizontal_chunks[0]);
        self.render_right_panel(f, horizontal_chunks[1]);

        // modes of manipulate
        let mode_text = match self.mode {
            Mode::Normal => "   NORMAL   ",
            Mode::Insert => "   INSERT   ",
        };
        let mode_paragraph = Paragraph::new(mode_text)
            .style(Style::default().fg(Color::White)) //.bg(Color::Black)
            .alignment(Alignment::Center);
        f.render_widget(mode_paragraph, main_chunks[1]);
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
        match (self.mode, key_code) {
            (Mode::Normal, crossterm::event::KeyCode::Char('i')) => {
                self.mode = Mode::Insert;
                return None;
            }
            (Mode::Insert, crossterm::event::KeyCode::Esc) => {
                self.mode = Mode::Normal;
                return None;
            }
            _ => {}
        }

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
                match self.mode {
                    Mode::Normal => {
                        match key_code {
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
                            // crossterm::event::KeyCode::Char('j') => { .. }
                            // crossterm::event::KeyCode::Char('k') => { .. }
                            // crossterm::event::KeyCode::Char('l') => { .. }
                            _ => {}
                        };
                        None
                    }
                    Mode::Insert => {
                        match key_code {
                            crossterm::event::KeyCode::Backspace => { input.pop(); }
                            crossterm::event::KeyCode::Char(c) => { input.push(c); }
                            crossterm::event::KeyCode::Enter => {
                                let text = input.clone();
                                input.clear();
                                return Some(Command::SendMessage(*peer_id, text));
                            }
                            _ => {}
                        };
                        None
                    }
                }
            }
        }
    }
}
