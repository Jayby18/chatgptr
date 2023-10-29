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
    history: Vec<String>,
    selected_message: Option<usize>,
    yank_buffer: String,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            editing_mode: EditingMode::Normal,
            input_text: String::new(),
            cursor_position: 0,
            // TODO: set history back to empty vec
            history: vec![String::from("What is Rust?"), String::from("A blazingly fast programming language!"), String::from("Qui consequat deserunt sint. Velit excepteur Lorem ut excepteur ad labore voluptate.")],
            selected_message: None,
            yank_buffer: String::new(),
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

    // Get cursor position
    pub fn cursor_position(&self) -> u16 { self.cursor_position }
    // Navigate cursor
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position == 0 {
            if self.input_text.len() > 1 {
                self.cursor_position = self.input_text.len() as u16 - 1;
            }
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

    // Yank and paste
    // TODO: implement yanking from input box
    pub fn yank_selected(&mut self) {
        self.yank_buffer = self.history[self.selected_message.unwrap_or_default()].clone();
    }
    pub fn paste_buffer(&mut self) {
        self.yank_buffer.chars().for_each(|c| {
            self.input_text.insert(self.cursor_position as usize, c);
            self.cursor_position += 1;
            // TODO: check whether this cursor move breaks anything
        });
    }

    pub fn history(&self) -> &Vec<String> { &self.history }
    pub fn append_history(&mut self, message: String) {
        self.history.push(message);
    }

    // Selected message
    pub fn selected_message(&self) -> Option<usize> { self.selected_message }
    pub fn clear_msg_selection(&mut self) { self.selected_message = None }
    pub fn select_prev_msg(&mut self) {
        if let Some(index) = self.selected_message {
            if index == 0 {
                self.selected_message = Some(self.history.len() - 1);
            } else {
                self.selected_message = Some(index - 1);
            }
        } else if !self.history.is_empty() {
            self.selected_message = Some(self.history.len() - 1);
        }
    }
    pub fn select_next_msg(&mut self) {
        if let Some(index) = self.selected_message {
            if (index + 1) >= self.history.len() {
                self.selected_message = Some(0);
            } else {
                self.selected_message = Some(index + 1);
            }
        } else if !self.history.is_empty() {
            self.selected_message = Some(0);
        }
    }
}
