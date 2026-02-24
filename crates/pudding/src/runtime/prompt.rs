use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::template::{load_state, save_state};

use super::RuntimeApp;

pub(super) struct InputPrompt {
    pub(super) label: String,
    pub(super) buffer: String,
    mode: PromptMode,
}

#[derive(Clone, Copy)]
enum PromptMode {
    Save,
    Restore,
}

impl InputPrompt {
    pub(super) fn save_mode() -> Self {
        Self {
            label: "保存名".to_string(),
            buffer: String::new(),
            mode: PromptMode::Save,
        }
    }

    pub(super) fn restore_mode() -> Self {
        Self {
            label: "復元名".to_string(),
            buffer: String::new(),
            mode: PromptMode::Restore,
        }
    }
}

impl RuntimeApp {
    pub(super) fn handle_prompt_key(&mut self, prompt: &mut InputPrompt, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                let name = prompt.buffer.trim().to_string();
                match prompt.mode {
                    PromptMode::Save => {
                        if !name.is_empty() {
                            match save_state(&name, &self.template) {
                                Ok(_) => {
                                    self.status = format!("保存しました: {}", name);
                                }
                                Err(err) => {
                                    self.status = format!("保存に失敗: {err}");
                                }
                            }
                        }
                    }
                    PromptMode::Restore => {
                        if !name.is_empty() {
                            match load_state(&name) {
                                Ok(tpl) => {
                                    self.template = tpl;
                                    self.panes.clear();
                                    match self.spawn_all() {
                                        Ok(_) => {
                                            self.status = format!("復元しました: {}", name);
                                        }
                                        Err(err) => {
                                            self.status = format!("復元に失敗: {err}");
                                        }
                                    }
                                }
                                Err(err) => {
                                    self.status = format!("復元に失敗: {err}");
                                }
                            }
                        }
                    }
                }
                return true;
            }
            KeyCode::Esc => {
                return true;
            }
            KeyCode::Backspace => {
                prompt.buffer.pop();
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return false;
                }
                prompt.buffer.push(c);
            }
            _ => {}
        }
        false
    }
}
