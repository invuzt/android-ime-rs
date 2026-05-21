use android_ime::AndroidImeEditable;
use eframe::egui::{self, Response, Ui};

/// Widget input field seperti EditText di Android
/// Keyboard akan muncul otomatis saat diklik
pub struct EditTextWidget {
    text: String,
    focus: bool,
    placeholder: String,
}

impl EditTextWidget {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            focus: false,
            placeholder: String::from("Ketik di sini..."),
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn take_text(&mut self) -> String {
        std::mem::take(&mut self.text)
    }

    pub fn show(&mut self, ui: &mut Ui, ime: &mut AndroidImeEditable) -> Response {
        let text_edit = egui::TextEdit::singleline(&mut self.text)
            .hint_text(&self.placeholder);
        
        let response = ui.add(text_edit);
        
        if response.clicked() && !self.focus {
            self.focus = true;
            let _ = ime.show_soft_keyboard();
        }
        
        if response.lost_focus() && self.focus {
            self.focus = false;
            let _ = ime.hide_soft_keyboard();
        }
        
        if response.changed() {
            ui.ctx().request_repaint();
        }
        
        response
    }
}

impl Default for EditTextWidget {
    fn default() -> Self {
        Self::new()
    }
}
