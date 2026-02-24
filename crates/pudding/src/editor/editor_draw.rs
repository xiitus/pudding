use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{layout::layout_rects, model::Node};

use super::{editor_area::centered_rect, editor_input::InputKind, EditorApp};

impl EditorApp {
    pub(super) fn draw(&mut self, f: &mut ratatui::Frame) {
        let area = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(2)])
            .split(area);
        let main = chunks[0];
        let status = chunks[1];

        let mut rects = Vec::new();
        layout_rects(&self.template.layout, main, &mut rects);

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
}
