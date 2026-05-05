use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Style, Stylize},
    symbols,
    Frame,
    layout::{Layout, Constraint, Direction}
};
use crate::vk_api::{Dialog, Message};

pub enum Screen {
    ChatList {
        list_state: ListState,
        dialogs: Vec<Dialog>,
    },
    ChatView {
        peer_id: i64,
        messages: Vec<Message>,
        input: String,
        scroll: usize,
    },
}

pub struct App {
    pub screen: Screen,
    pub min_size: (u16, u16),
}

impl App {
    pub fn new(dialogs: Vec<Dialog>, min_size: (u16, u16)) -> Self {
        let mut list_state = ListState::default();
        if !dialogs.is_empty() {
            list_state.select(Some(0));
        }
        App {
            screen: Screen::ChatList {
                list_state,
                dialogs,
            },
            min_size,
        }
    }

    /// Отрисовывает весь интерфейс в зависимости от текущего экрана
    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();   // используем area() вместо устаревшего size()

        // Проверяем минимальный размер
        if area.width < self.min_size.0 || area.height < self.min_size.1 {
            let message = format!(
                "Минимальный размер: {}x{}; сейчас терминал {}x{}",
                self.min_size.0, self.min_size.1, area.width, area.height
            );
            let paragraph = Paragraph::new(message)
                .block(Block::default().borders(Borders::ALL))
                .centered();
            f.render_widget(paragraph, area);
            return;
        }

        // Делим экран на левую и правую части
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        self.render_left_panel(f, chunks[0]);
        self.render_right_panel(f, chunks[1]);
    }

    fn render_left_panel(&mut self, f: &mut Frame, area: ratatui::layout::Rect) {
        // Блок с ASCII-границами
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
            .title(" Чаты ");

        match &mut self.screen {
            Screen::ChatList { list_state, dialogs } => {
                let items: Vec<ListItem> = dialogs
                    .iter()
                    .map(|d| ListItem::new(d.title.clone()))
                    .collect();
                let list = List::new(items)
                    .block(block)
                    .highlight_style(Style::default().reversed());
                f.render_stateful_widget(list, area, list_state);
            }
            _ => unreachable!(),
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
            .title(" Сообщения ");

        match &self.screen {
            Screen::ChatList { .. } => {
                let paragraph = Paragraph::new("Выберите чат слева")
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
                let content = format!("{}\n\nВвод: {}", msg_text, input);
                let paragraph = Paragraph::new(content).block(block);
                f.render_widget(paragraph, area);
            }
        }
    }

    /// Обработка нажатий клавиш; возвращает false, если приложение должно завершиться
    pub fn handle_input(&mut self, key_code: crossterm::event::KeyCode) -> bool {
        match &mut self.screen {
            Screen::ChatList { list_state, dialogs } => {
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
                            }
                        }
                    }
                    crossterm::event::KeyCode::Char('q') => return false,
                    _ => {}
                }
            }
            Screen::ChatView { .. } => {
                match key_code {
                    crossterm::event::KeyCode::Char('h') => {
                        // Возврат в список чатов (диалоги нужно сохранить, сейчас временно пустой список)
                        self.screen = Screen::ChatList {
                            list_state: ListState::default(),
                            dialogs: Vec::new(),
                        };
                    }
                    crossterm::event::KeyCode::Char('q') => return false,
                    _ => {}
                }
            }
        }
        true
    }
}
