pub struct AppState {
    mode: EditingMode,
    user_text: String,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            mode: EditingMode::Normal,
            user_text: String::new(),
        }
    }

    pub fn mode(&self) -> EditingMode { self.mode }
    pub fn user_text(&self) -> String { self.user_text.clone() }

    pub fn switch_mode(&mut self, mode: EditingMode) { self.mode = mode; }

    // pub fn set_user_text(&mut self, text: String) { self.user_text = text; }
    pub fn push_user_text(&mut self, c: char) { self.user_text.push(c); }
    pub fn pop_user_text(&mut self) { self.user_text.pop(); }
    pub fn clear_user_text(&mut self) { self.user_text = String::new(); }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum EditingMode {
    Normal,
    Insert,
}