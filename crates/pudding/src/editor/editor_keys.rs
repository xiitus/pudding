use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;

use crate::{
    layout::{find_bite_at, layout_rects, next_id, split_bite},
    model::{Node, Orientation},
    template::save_template,
};

use super::{
    editor_area::main_area,
    editor_input::{InputKind, InputMode},
    EditorApp,
};

impl EditorApp {
    pub(super) fn handle_key(&mut self, key: KeyEvent, area: Rect) -> Result<bool> {
        if let Some(mut input) = self.input.take() {
            let close = self.handle_input_key(&mut input, key);
            if !close {
                self.input = Some(input);
            }
            return Ok(false);
        }

        let main = main_area(area);
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('v') => self.split_at_cursor(main, Orientation::Vertical),
            KeyCode::Char('h') => self.split_at_cursor(main, Orientation::Horizontal),
            KeyCode::Char('n') => {
                self.input = Some(InputMode::for_name());
            }
            KeyCode::Char('c') => {
                self.input = Some(InputMode::for_command());
            }
            KeyCode::Char('s') => match save_template(&self.template) {
                Ok(_) => self.message = "テンプレートを保存しました".to_string(),
                Err(_) => self.message = "保存に失敗しました".to_string(),
            },
            KeyCode::Left => {
                if self.cursor_x > main.x {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_x + 1 < main.x + main.width {
                    self.cursor_x += 1;
                }
            }
            KeyCode::Up => {
                if self.cursor_y > main.y {
                    self.cursor_y -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_y + 1 < main.y + main.height {
                    self.cursor_y += 1;
                }
            }
            _ => {}
        }

        if let Some(id) = find_bite_at(&self.template.layout, main, self.cursor_x, self.cursor_y) {
            self.selected_id = id;
        }
        Ok(false)
    }

    fn handle_input_key(&mut self, input: &mut InputMode, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.apply_input(input);
                return true;
            }
            KeyCode::Esc => {
                return true;
            }
            KeyCode::Backspace => {
                input.buffer.pop();
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return false;
                }
                input.buffer.push(c);
            }
            _ => {}
        }
        false
    }

    fn apply_input(&mut self, input: &InputMode) {
        match input.kind {
            InputKind::Name => {
                if let Some(Node::Bite { name, .. }) =
                    crate::layout::find_bite_mut(&mut self.template.layout, self.selected_id)
                {
                    if !input.buffer.is_empty() {
                        *name = input.buffer.clone();
                    }
                }
            }
            InputKind::Command => {
                if let Some(Node::Bite { command, .. }) =
                    crate::layout::find_bite_mut(&mut self.template.layout, self.selected_id)
                {
                    if !input.buffer.is_empty() {
                        *command = input.buffer.clone();
                    }
                }
            }
        }
    }

    fn split_at_cursor(&mut self, main: Rect, orientation: Orientation) {
        let target_id = self.selected_id;
        let rects = {
            let mut rects = Vec::new();
            layout_rects(&self.template.layout, main, &mut rects);
            rects
        };
        let rect = rects
            .into_iter()
            .find(|(id, _)| *id == target_id)
            .map(|(_, r)| r);
        if let Some(rect) = rect {
            let ratio = match orientation {
                Orientation::Vertical => {
                    let dx = self.cursor_x.saturating_sub(rect.x) as f32;
                    if rect.width == 0 {
                        0.5
                    } else {
                        dx / rect.width as f32
                    }
                }
                Orientation::Horizontal => {
                    let dy = self.cursor_y.saturating_sub(rect.y) as f32;
                    if rect.height == 0 {
                        0.5
                    } else {
                        dy / rect.height as f32
                    }
                }
            };
            let new_id = next_id(&self.template.layout);
            let default_command = match crate::config::Config::load() {
                Ok(cfg) => cfg.default_command,
                Err(err) => {
                    self.message = format!("設定読込に失敗: {err}");
                    return;
                }
            };
            let did = split_bite(
                &mut self.template.layout,
                target_id,
                orientation,
                ratio,
                new_id,
                &default_command,
            );
            if did {
                self.message = "分割しました".to_string();
            }
        }
    }
}
