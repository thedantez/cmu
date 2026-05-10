use crossterm::event::KeyCode;
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Stylize, Color},
    symbols,
    Frame,
    text::{Line, Span, Text},
    layout::{Layout, Constraint, Direction, Alignment}
};
use crate::config::{Config};
use crate::vk_api::{VkClient};
use crate::navigation::{Mode, typing};
use crate::client::{Dialog, Message, Client};

#[derive(Clone)]
pub enum Screen {
    ChatList {
        list_state: ListState,
    },
    ChatView {
        peer_id: i64,
        messages: Vec<Message>,
        input: String,
        selected: usize,
        scroll: usize,
        cursor_char_idx: usize,
        input_scroll: usize,
    },
}

pub struct App {
    pub screen: Screen,
    pub dialogs: Vec<Dialog>,
    pub min_size: (u16, u16),
    pub client: VkClient, pub running: bool,
    pub mode: Mode,
    pub config: Config
}

impl App {
    pub fn new(client: VkClient, dialogs: Vec<Dialog>, min_size: (u16, u16), conf: Config) -> Self {
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
            config: conf,
        }
    }

    pub async fn load_messages(&mut self, peer_id: i64) {
        if let Ok(messages) = self.client.get_messages(peer_id, 20).await {
            if let Screen::ChatView { messages: msg_vec, .. } = &mut self.screen {
                *msg_vec = messages;
            }
        }
    }

    pub async fn send_message(&mut self, peer_id: i64, text: &str) {
        if let Err(e) = self.client.send_message(peer_id, &text).await {
            eprintln!("Error sending a message: {}", e);
        } else {
            self.load_messages(peer_id).await;
        }
    }

    fn update_input_scroll(input: &str, cursor_char_idx: usize, visible_lines: usize, input_scroll: &mut usize) {
        let lines: Vec<&str> = input.split('\n').collect();
        let mut remaining_chars = cursor_char_idx;
        let mut cursor_line = 0;
        for (i, line) in lines.iter().enumerate() {
            let char_count = line.chars().count();
            if remaining_chars <= char_count {
                cursor_line = i;
                break;
            }
            remaining_chars -= char_count + 1;
            if i == lines.len() - 1 {
                cursor_line = i;
            }
        }
        if cursor_line < *input_scroll {
            *input_scroll = cursor_line;
        } else if cursor_line >= *input_scroll + visible_lines {
            *input_scroll = cursor_line - visible_lines + 1;
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

        // split screen for mode, left & right sides
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1), // for mode
            ])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(vertical_chunks[0]); // was area

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
        f.render_widget(mode_paragraph, vertical_chunks[1]);
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
            Screen::ChatView { messages, input, selected, scroll, cursor_char_idx, input_scroll, .. } => {
                let vertical_split = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ])
                    .split(area);
                let msg_area = vertical_split[0];
                let input_area = vertical_split[1];

                // list messages
                let items: Vec<ListItem> = messages.iter()
                    .map(|m| ListItem::new(format!("{} : {}", m.sender_name, m.text)))
                    .collect();
                let mut list_state = ListState::default()
                    .with_selected(Some(*selected))
                    .with_offset(*scroll);
                // list_state.select_and_offset(Some(*selected), *scroll);
                let list = List::new(items)
                    .block(block) // block w/ header "  messages  "
                    .highlight_style(Style::default().reversed());
                f.render_stateful_widget(list, msg_area, &mut list_state);

                // input area
                let input_block = Block::default()
                    .borders(Borders::ALL)
                    .title("  input  ");

                let lines: Vec<&str> = input.split('\n').collect(); let mut remaining_chars = *cursor_char_idx;
                let mut cursor_line_idx = 0;
                let mut cursor_byte_in_line = 0;
                for (i, line) in lines.iter().enumerate() {
                    let char_count = line.chars().count();
                    if remaining_chars <= char_count {
                        cursor_line_idx = i;
                        cursor_byte_in_line = line.char_indices()
                            .nth(remaining_chars)
                            .map(|(bi, _)| bi)
                            .unwrap_or(line.len());
                        break;
                    }
                    remaining_chars -= char_count + 1; // +1 cause '\n'
                    if i == lines.len() - 1 {
                        cursor_line_idx = i;
                        cursor_byte_in_line = line.len();
                    }
                }
                let max_visible_lines = (input_area.height as usize).saturating_sub(2); // height -
                                                                                        // frame
                let display_lines: Vec<Line> = lines
                    .iter()
                    .enumerate()
                    .skip(*input_scroll)
                    .take(max_visible_lines)
                    .map(|(i, line)| {
                        if i == cursor_line_idx {
                            let before = &line[..cursor_byte_in_line.min(line.len())];
                            let cursor_char = if cursor_byte_in_line < line.len() {
                                let ch = line[cursor_byte_in_line..].chars().next().unwrap_or(' ');
                                &line[cursor_byte_in_line..cursor_byte_in_line + ch.len_utf8()]
                            } else {
                                " "
                            };
                            let after = if cursor_byte_in_line < line.len() {
                                let ch = line[cursor_byte_in_line..].chars().next().unwrap_or(' ');
                                let next_byte = cursor_byte_in_line + ch.len_utf8();
                                &line[next_byte..]
                            } else {
                                ""
                            };
                            Line::from(vec![
                                Span::raw(before),
                                Span::styled(cursor_char, Style::default().bg(Color::White).fg(Color::Black)),
                                Span::raw(after),
                            ])
                        } else {
                            Line::from(Span::raw(*line))
                        }
                    })
                    .collect();
                let text = Text::from(display_lines);
                f.render_widget(Paragraph::new(text).block(input_block), input_area);
            }
        }
    }

    // Process key input
    pub async fn handle_input(&mut self, key_code: KeyCode) {
        // Global key bindings
        if self.config.keys.quit == key_code {
            self.running = false;
        }

        // Mode keybinds
        match self.mode {
            Mode::Normal => {
                if self.config.keys.to_insert_mode == key_code {
                    self.mode = Mode::Insert;
                }
            }
            Mode::Insert => {
                if self.config.keys.to_normal_mode == key_code {
                    self.mode = Mode::Normal;
                }
            }
        }

        // Key bindings for specific types of screens
        match &mut self.screen.clone() {
            Screen::ChatList { list_state } => {
                let dialogs = &self.dialogs;
                if self.config.keys.move_up_list == key_code {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i >= dialogs.len() - 1 { 0 } else { i + 1 }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                if self.config.keys.move_down_list == key_code {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i == 0 { dialogs.len() - 1 } else { i - 1 }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                if [self.config.keys.enter_chat, self.config.keys.enter_chat_secondary].contains(&key_code) {
                    if let Some(selected) = list_state.selected() {
                        if let Some(dialog) = dialogs.get(selected) {
                            self.screen = Screen::ChatView {
                                peer_id: dialog.peer_id,
                                messages: Vec::new(),
                                input: String::new(),
                                scroll: 0,
                                selected: 0,
                                cursor_char_idx: 0,
                                input_scroll: 0,
                            };
                        }
                    }
                }
            }

            Screen::ChatView { peer_id, messages, input, scroll, selected, cursor_char_idx, input_scroll } => {
                match self.mode {
                    Mode::Normal => {
                        if self.config.keys.view_chat_list == key_code {
                            self.screen = Screen::ChatList {
                                list_state: ListState::default(),
                            };
                        }
                        if self.config.keys.move_down_list == key_code {
                            if *selected + 1 < messages.len() {
                                *selected += 1;
                            }
                            *scroll = *selected;
                        }
                        if self.config.keys.move_up_list == key_code {
                            if *selected > 0 {
                                *selected -= 1;
                            }
                            *scroll = *selected;
                        }
                        if let KeyCode::Enter = key_code {
                            let text = input.clone();
                            input.clear();
                            self.send_message(*peer_id, &text).await;
                        }
                    }

                    Mode::Insert => {
                        typing(input, cursor_char_idx, key_code);
                        Self::update_input_scroll(input, *cursor_char_idx, 1, input_scroll);
                    }
                }
            }
        }
    }
}
