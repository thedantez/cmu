use crossterm::event::KeyCode;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
}


pub fn typing(input: &mut String, cursor_char_idx: &mut usize, key_code: KeyCode) {
    match key_code {
        KeyCode::Backspace => {
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
        KeyCode::Delete => {
            if *cursor_char_idx < input.chars().count() {
                let byte_pos = input.char_indices()
                    .nth(*cursor_char_idx)
                    .map(|(i, _)| i)
                    .unwrap_or(input.len());
                input.remove(byte_pos);
            }
        }
        KeyCode::End => { *cursor_char_idx = input.chars().count(); }
        KeyCode::Home => { *cursor_char_idx = 0; }
        KeyCode::Enter => {
            input.insert(*cursor_char_idx, '\n');
            *cursor_char_idx += 1;
        }
        KeyCode::Left => {
            if *cursor_char_idx > 0 { *cursor_char_idx -= 1; }
        }
        KeyCode::Right => {
            if *cursor_char_idx < input.chars().count() { *cursor_char_idx += 1; }
        }
        KeyCode::Char(c) => {
            let byte_pos = input.char_indices()
                .nth(*cursor_char_idx)
                .map(|(i, _)| i)
                .unwrap_or(input.len());
            input.insert(byte_pos, c);
            *cursor_char_idx += 1;
        }
        _ => {}
    };
}
