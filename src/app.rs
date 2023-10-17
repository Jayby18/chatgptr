/// Editing editing_modes
#[derive(Clone, Copy, Default, PartialEq)]
pub enum EditingMode {
    #[default] Normal,
    Insert,
    Visual,
}

/// App state
#[derive(Default)]
pub struct AppState {
    editing_mode: EditingMode,
    input_text: String,
    cursor_position: u16,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            editing_mode: EditingMode::Normal,
            input_text: String::new(),
            cursor_position: 0,
        }
    }

    // Get and switch editing editing_modes
    pub fn editing_mode(&self) -> EditingMode { self.editing_mode }
    pub fn switch_editing_mode(&mut self, editing_mode: EditingMode) { self.editing_mode = editing_mode; }

    // Get and change user text
    pub fn input_text(&self) -> &str { &self.input_text }
    pub fn insert_char(&mut self, c: char) {
        self.input_text.insert(self.cursor_position as usize, c);
        self.move_cursor_right();
    }
    pub fn remove_char(&mut self) {
        if self.cursor_position >= self.input_text.len() as u16 {
            return
        }
        self.input_text.remove(self.cursor_position as usize);
    }
    pub fn backspace(&mut self) {
        // TODO: backspace
    }
    pub fn set_input_text(&mut self, text: &str) { self.input_text = String::from(text); }
    pub fn clear_input_text(&mut self) { self.input_text = String::new(); }

    // Example
    // input_text = Hello
    //              01234
    // length is 5
    // if cursor is at 0, we want to loop around position 4 again

    pub fn cursor_position(&self) -> u16 { self.cursor_position }
    // Navigate cursor
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position == 0 {
            self.cursor_position = self.input_text.len() as u16 - 1;
        } else {
            self.cursor_position -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position + 1 > self.input_text.len() as u16 {
            self.cursor_position = 0;
        } else {
            self.cursor_position += 1;
        }
    }
}
