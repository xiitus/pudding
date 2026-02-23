use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::{
    layout::{find_bite_at, layout_rects, next_id, split_bite},
    model::{Node, Orientation, Template},
    template::save_template,
};

#[derive(Debug, Clone, Copy)]
enum InputKind {
    Name,
    Command,
}

struct InputMode {
    kind: InputKind,
    buffer: String,
}

pub struct EditorApp {
    template: Template,
    cursor_x: u16,
    cursor_y: u16,
    selected_id: u64,
    input: Option<InputMode>,
    message: String,
}

impl EditorApp {
    pub fn new(template: Template) -> Self {
        let selected_id = template.layout.id();
        Self {
            template,
            cursor_x: 1,
            cursor_y: 1,
            selected_id,
            input: None,
            message: "v/hで分割、nで名前、cでコマンド、sで保存、qで終了".to_string(),
        }
    }

    pub fn run(mut self) -> Result<Template> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.loop_ui(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        res
    }

    fn loop_ui(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<Template> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key(key, terminal.size()?)? {
                        break;
                    }
                }
            }
        }
        Ok(self.template.clone())
    }

    fn draw(&mut self, f: &mut ratatui::Frame) {
        let area = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(area);
        let main = chunks[0];
        let status = chunks[1];

        let mut rects = Vec::new();
        layout_rects(&self.template.layout, main, &mut rects);

        // Draw panes
        for (id, rect) in rects.iter() {
            let (title, highlight) = match crate::layout::find_bite(&self.template.layout, *id) {
                Some(Node::Bite { name, .. }) => (name.clone(), *id == self.selected_id),
                _ => ("".to_string(), *id == self.selected_id),
            };
            let style = if highlight {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(style);
            f.render_widget(block, *rect);
        }

        // Cursor marker
        if main.width > 0 && main.height > 0 {
            let x = self.cursor_x.min(main.x + main.width - 1);
            let y = self.cursor_y.min(main.y + main.height - 1);
            if x >= main.x && y >= main.y {
                let cell = f.buffer_mut().get_mut(x, y);
                cell.set_symbol("x");
                cell.set_style(Style::default().fg(Color::Cyan));
            }
        }

        let status_text = Line::from(vec![
            Span::raw("[Editor] "),
            Span::raw(self.message.clone()),
        ]);
        let status_widget = Paragraph::new(status_text);
        f.render_widget(status_widget, status);

        if let Some(input) = &self.input {
            let prompt = match input.kind {
                InputKind::Name => "名前を入力: ",
                InputKind::Command => "コマンドを入力: ",
            };
            let line = Line::from(vec![Span::raw(prompt), Span::raw(&input.buffer)]);
            let block = Block::default().borders(Borders::ALL).title("Input");
            let area = centered_rect(80, 3, area);
            f.render_widget(block, area);
            f.render_widget(
                Paragraph::new(line),
                Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width - 2,
                    height: 1,
                },
            );
            f.set_cursor(
                area.x + 1 + prompt.len() as u16 + input.buffer.len() as u16,
                area.y + 1,
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent, area: Rect) -> Result<bool> {
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
                self.input = Some(InputMode {
                    kind: InputKind::Name,
                    buffer: String::new(),
                });
            }
            KeyCode::Char('c') => {
                self.input = Some(InputMode {
                    kind: InputKind::Command,
                    buffer: String::new(),
                });
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

fn main_area(area: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(area);
    chunks[0]
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let vertical = popup_layout[1];
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((vertical.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical);
    horizontal[1]
}
