use crossterm::event::KeyCode;
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Stylize, Color},
    symbols,
    Frame,
    text::{Line, Span, Text},
    layout::{Layout, Constraint, Direction, Alignment}
};
use crate::config::Config;
use crate::navigation::{Mode, typing};
use crate::client::{Dialog, Message, Client};

const MESSAGES_PER_PAGE: u32 = 20;
const INPUT_AREA_HEIGHT: u16 = 3;
const LEFT_PANEL_WIDTH: u16 = 30;
const MODE_BAR_HEIGHT: u16 = 1;

/// Commands emitted by the UI to the main loop
/// Used to prevent double mutable borrowing of self
#[derive(Debug)]
pub enum Command {
    LoadMessages(i64),
    SendMessage(i64, String),
}

/// Represents the current screen state
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

/// Main application state
pub struct App {
    pub screen: Screen,
    pub dialogs: Vec<Dialog>,
    pub min_size: (u16, u16),
    pub client: Box<dyn Client>,
    pub running: bool,
    pub mode: Mode,
    pub config: Config,
}

impl App {
    pub fn new(
        client: Box<dyn Client>,
        dialogs: Vec<Dialog>,
        min_size: (u16, u16),
        conf: Config,
    ) -> Self {
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
        match self.client.get_messages(peer_id, MESSAGES_PER_PAGE).await {
            Ok(messages) => {
                if let Screen::ChatView { messages: msg_vec, .. } = &mut self.screen {
                    *msg_vec = messages;
                }
            }
            Err(e) => {
                eprintln!("Failed to load messages: {}", e);
            }
        }
    }

    pub async fn send_message(&mut self, peer_id: i64, text: &str) {
        match self.client.send_message(peer_id, text).await {
            Ok(()) => {
                self.load_messages(peer_id).await;
            }
            Err(e) => {
                eprintln!("Failed to send message: {}", e);
            }
        }
    }

    /// Calculate which line the cursor is on based on character index
    fn get_cursor_line_info(input: &str, cursor_char_idx: usize) -> (usize, usize) {
        let lines: Vec<&str> = input.split('\n').collect();
        let mut remaining_chars = cursor_char_idx;
        let mut cursor_line = 0;
        let mut cursor_byte_in_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let char_count = line.chars().count();
            if remaining_chars <= char_count {
                cursor_line = i;
                cursor_byte_in_line = line
                    .char_indices()
                    .nth(remaining_chars)
                    .map(|(bi, _)| bi)
                    .unwrap_or(line.len());
                break;
            }
            remaining_chars -= char_count + 1;
            if i == lines.len() - 1 {
                cursor_line = i;
                cursor_byte_in_line = line.len();
            }
        }

        (cursor_line, cursor_byte_in_line)
    }

    fn update_input_scroll(
        input: &str,
        cursor_char_idx: usize,
        visible_lines: usize,
        input_scroll: &mut usize,
    ) {
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
            *input_scroll = cursor_line.saturating_sub(visible_lines - 1);
        }
    }

    /// Render the entire UI
    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        if !self.check_min_size(f, area) {
            return;
        }

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(MODE_BAR_HEIGHT)])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(LEFT_PANEL_WIDTH),
                Constraint::Percentage(100 - LEFT_PANEL_WIDTH),
            ])
            .split(vertical_chunks[0]);

        self.render_left_panel(f, horizontal_chunks[0]);
        self.render_right_panel(f, horizontal_chunks[1]);
        self.render_mode_bar(f, vertical_chunks[1]);
    }

    fn check_min_size(&self, f: &mut Frame, area: ratatui::layout::Rect) -> bool {
        if area.width < self.min_size.0 || area.height < self.min_size.1 {
            let message = format!(
                "Minimum size: {}x{}; Current: {}x{}",
                self.min_size.0, self.min_size.1, area.width, area.height
            );
            let paragraph = Paragraph::new(message)
                .block(Block::default().borders(Borders::ALL))
                .centered();
            f.render_widget(paragraph, area);
            false
        } else {
            true
        }
    }

    fn render_mode_bar(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let mode_text = match self.mode {
            Mode::Normal => "   NORMAL   ",
            Mode::Insert => "   INSERT   ",
        };
        let mode_paragraph = Paragraph::new(mode_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(mode_paragraph, area);
    }

    fn render_left_panel(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(Self::ascii_border_set())
            .title(" chats ");

        match &mut self.screen {
            Screen::ChatList { list_state } => {
                let items: Vec<ListItem> = self
                    .dialogs
                    .iter()
                    .map(|d| ListItem::new(d.title.clone()))
                    .collect();
                let list = List::new(items)
                    .block(block)
                    .highlight_style(Style::default().reversed());
                f.render_stateful_widget(list, area, list_state);
            }
            _ => {
                let items: Vec<ListItem> = self
                    .dialogs
                    .iter()
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
            .border_set(Self::ascii_border_set())
            .title(" messages ");

        match &self.screen {
            Screen::ChatList { .. } => {
                let paragraph = Paragraph::new("Select a chat from the left panel")
                    .block(block)
                    .centered();
                f.render_widget(paragraph, area);
            }
            Screen::ChatView {
                messages,
                input,
                selected,
                scroll,
                cursor_char_idx,
                input_scroll,
                ..
            } => {
                let vertical_split = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(INPUT_AREA_HEIGHT)])
                    .split(area);

                self.render_messages(f, vertical_split[0], messages, *selected, *scroll, block);
                self.render_input(
                    f,
                    vertical_split[1],
                    input,
                    *cursor_char_idx,
                    *input_scroll,
                );
            }
        }
    }

    fn render_messages(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        messages: &[Message],
        selected: usize,
        scroll: usize,
        block: Block,
    ) {
        let items: Vec<ListItem> = messages
            .iter()
            .map(|m| {
                ListItem::new(Line::from(vec![
                    if m.is_me {
                        Span::styled("You: ", Style::default().fg(Color::Blue))
                    } else {
                        Span::raw(format!("{}: ", &m.sender_name))
                    },
                    Span::raw(&m.text),
                ]))
            })
            .collect();

        let mut list_state = ListState::default()
            .with_selected(Some(selected))
            .with_offset(scroll);

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().reversed());

        f.render_stateful_widget(list, area, &mut list_state);
    }

    fn render_input(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        input: &str,
        cursor_char_idx: usize,
        input_scroll: usize,
    ) {
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("  input  ");

        let lines: Vec<&str> = input.split('\n').collect();
        let (cursor_line_idx, cursor_byte_in_line) = Self::get_cursor_line_info(input, cursor_char_idx);

        let max_visible_lines = (area.height as usize).saturating_sub(2);
        let display_lines: Vec<Line> = lines
            .iter()
            .enumerate()
            .skip(input_scroll)
            .take(max_visible_lines)
            .map(|(i, line)| {
                if i == cursor_line_idx {
                    Self::render_cursor_line(line, cursor_byte_in_line)
                } else {
                    Line::from(Span::raw(*line))
                }
            })
            .collect();

        let text = Text::from(display_lines);
        f.render_widget(Paragraph::new(text).block(input_block), area);
    }

    fn render_cursor_line(line: &str, cursor_byte_in_line: usize) -> Line {
        let before = &line[..cursor_byte_in_line.min(line.len())];
        let cursor_char = if cursor_byte_in_line < line.len() {
            let ch = line[cursor_byte_in_line..]
                .chars()
                .next()
                .unwrap_or(' ');
            &line[cursor_byte_in_line..cursor_byte_in_line + ch.len_utf8()]
        } else {
            " "
        };
        let after = if cursor_byte_in_line < line.len() {
            let ch = line[cursor_byte_in_line..]
                .chars()
                .next()
                .unwrap_or(' ');
            let next_byte = cursor_byte_in_line + ch.len_utf8();
            &line[next_byte..]
        } else {
            ""
        };

        Line::from(vec![
            Span::raw(before),
            Span::styled(
                cursor_char,
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black),
            ),
            Span::raw(after),
        ])
    }

    fn ascii_border_set() -> symbols::border::Set {
        symbols::border::Set {
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
            vertical_left: "|",
            vertical_right: "|",
            horizontal_top: "-",
            horizontal_bottom: "-",
        }
    }

    /// Process keyboard input and return commands
    pub fn handle_input(&mut self, key_code: KeyCode) -> Option<Command> {
        // Global key bindings
        if self.config.keys.quit == key_code {
            self.running = false;
            return None;
        }

        // Mode transitions
        match self.mode {
            Mode::Normal => {
                if self.config.keys.to_insert_mode == key_code {
                    self.mode = Mode::Insert;
                    return None;
                }
            }
            Mode::Insert => {
                if self.config.keys.to_normal_mode == key_code {
                    self.mode = Mode::Normal;
                    return None;
                }
            }
        }

        // Screen-specific key bindings
        match &mut self.screen {
            Screen::ChatList { list_state } => {
                self.handle_chat_list_input(key_code, list_state)
            }
            Screen::ChatView {
                peer_id,
                messages,
                input,
                scroll,
                selected,
                cursor_char_idx,
                input_scroll,
            } => self.handle_chat_view_input(
                key_code,
                *peer_id,
                messages,
                input,
                scroll,
                selected,
                cursor_char_idx,
                input_scroll,
            ),
        }
    }

    fn handle_chat_list_input(&mut self, key_code: KeyCode, list_state: &mut ListState) -> Option<Command> {
        let dialogs = &self.dialogs;

        if self.config.keys.move_down_list == key_code {
            let next_idx = match list_state.selected() {
                Some(i) if i >= dialogs.len().saturating_sub(1) => 0,
                Some(i) => i + 1,
                None => 0,
            };
            list_state.select(Some(next_idx));
        } else if self.config.keys.move_up_list == key_code {
            let prev_idx = match list_state.selected() {
                Some(0) => dialogs.len().saturating_sub(1),
                Some(i) => i - 1,
                None => 0,
            };
            list_state.select(Some(prev_idx));
        } else if [self.config.keys.enter_chat, self.config.keys.enter_chat_secondary]
            .contains(&key_code)
        {
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
        }

        None
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_chat_view_input(
        &mut self,
        key_code: KeyCode,
        peer_id: i64,
        messages: &[Message],
        input: &mut String,
        scroll: &mut usize,
        selected: &mut usize,
        cursor_char_idx: &mut usize,
        input_scroll: &mut usize,
    ) -> Option<Command> {
        match self.mode {
            Mode::Normal => {
                if self.config.keys.move_down_list == key_code {
                    if *selected + 1 < messages.len() {
                        *selected += 1;
                        *scroll = *selected;
                    }
                } else if self.config.keys.move_up_list == key_code {
                    if *selected > 0 {
                        *selected -= 1;
                        *scroll = *selected;
                    }
                } else if key_code == KeyCode::Enter {
                    if !input.is_empty() {
                        let text = input.clone();
                        input.clear();
                        return Some(Command::SendMessage(peer_id, text));
                    }
                } else if self.config.keys.view_chat_list == key_code {
                    self.screen = Screen::ChatList {
                        list_state: ListState::default(),
                    };
                }
                None
            }
            Mode::Insert => {
                typing(input, cursor_char_idx, key_code);
                Self::update_input_scroll(input, *cursor_char_idx, 1, input_scroll);
                None
            }
        }
    }
}
