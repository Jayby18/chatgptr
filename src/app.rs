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
    cursor_position: usize,
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
    pub fn input_text(&self) -> String { self.input_text.clone() }
    pub fn set_input_text(&mut self, text: String) { self.input_text = text; }
    pub fn push_input_text(&mut self, c: char) { self.input_text.push(c); }
    pub fn pop_input_text(&mut self) { self.input_text.pop(); }
    pub fn clear_input_text(&mut self) { self.input_text = String::new(); }

    // Example
    // input_text = Hello
    //              01234
    // length is 5
    // if cursor is at 0, we want to loop around position 4 again

    // Navigate cursor
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position <= 0 {
            self.cursor_position = self.input_text.len() - 1;
        } else {
            self.cursor_position -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position >= (self.input_text.len() - 1) {
            self.cursor_position = 0;
        } else {
            self.cursor_position += 1;
        }
    }
}
