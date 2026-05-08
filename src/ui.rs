use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Stylize, Color},
    symbols,
    Frame,
    text::{Line, Span, Text},
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
        selected: usize,
        scroll: usize,
        cursor_char_idx: usize,
        input_scroll: usize,
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

                let lines: Vec<&str> = input.split('\n').collect();
                let mut remaining_chars = *cursor_char_idx;
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
                                    selected: 0,
                                    cursor_char_idx: 0,
                                    input_scroll: 0,
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
            Screen::ChatView { peer_id, messages, input, scroll, selected, cursor_char_idx, input_scroll } => {
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
                            crossterm::event::KeyCode::Char('j') => {
                                if *selected + 1 < messages.len() {
                                    *selected += 1;
                                }
                                *scroll = *selected;
                            }
                            crossterm::event::KeyCode::Char('k') => {
                                if *selected > 0 {
                                    *selected -= 1;
                                }
                                *scroll = *selected;
                            }
                            // crossterm::event::KeyCode::Char('l') => { .. }
                            crossterm::event::KeyCode::Enter => {
                                let text = input.clone();
                                input.clear();
                                return Some(Command::SendMessage(*peer_id, text));
                            }
                            _ => {}
                        };
                        None
                    }
                    Mode::Insert => {
                        match key_code {
                            crossterm::event::KeyCode::Backspace => {
                                if *cursor_char_idx > 0 {
                                    let byte_pos = input.char_indices()
                                        .take(*cursor_char_idx)
                                        .last()
                                        .map(|(i, _)| i)
                                        .unwrap();
                                    input.remove(byte_pos);
                                    *cursor_char_idx -= 1;
                                }
                            }
                            crossterm::event::KeyCode::Delete => {
                                if *cursor_char_idx < input.chars().count() {
                                    let byte_pos = input.char_indices()
                                        .nth(*cursor_char_idx)
                                        .map(|(i, _)| i)
                                        .unwrap_or(input.len());
                                    input.remove(byte_pos);
                                }
                            }
                            crossterm::event::KeyCode::End => { *cursor_char_idx = input.chars().count(); }
                            crossterm::event::KeyCode::Home => { *cursor_char_idx = 0; }
                            crossterm::event::KeyCode::Enter => {
                                input.insert(*cursor_char_idx, '\n');
                                *cursor_char_idx += 1;
                            }
                            crossterm::event::KeyCode::Left => {
                                if *cursor_char_idx > 0 { *cursor_char_idx -= 1; }
                            }
                            crossterm::event::KeyCode::Right => {
                                if *cursor_char_idx < input.chars().count() { *cursor_char_idx += 1; }
                            }
                            crossterm::event::KeyCode::Char(c) => {
                                let byte_pos = input.char_indices()
                                    .nth(*cursor_char_idx)
                                    .map(|(i, _)| i)
                                    .unwrap_or(input.len());
                                input.insert(byte_pos, c);
                                *cursor_char_idx += 1;
                            }
                            _ => {}
                        };
                        Self::update_input_scroll(input, *cursor_char_idx, 1, input_scroll);
                        None
                    }
                }
            }
        }
    }
}
