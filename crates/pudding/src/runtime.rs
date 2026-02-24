use std::{collections::HashMap, io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::PtySize;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    action::{actions_from_config, Action},
    config::Config,
    keybind::KeyBinding,
    layout::layout_rects,
    model::{Node, Template},
};

mod actions;
mod pane_process;
mod prompt;

#[path = "runtime_centered_rect.rs"]
mod runtime_centered_rect;
#[path = "runtime_key_to_bytes.rs"]
mod runtime_key_to_bytes;
#[path = "runtime_main_area.rs"]
mod runtime_main_area;
#[path = "runtime_terminal_size.rs"]
mod runtime_terminal_size;

use pane_process::PaneProcess;
use prompt::InputPrompt;
use runtime_centered_rect::centered_rect;
use runtime_key_to_bytes::key_to_bytes;
use runtime_main_area::main_area;
use runtime_terminal_size::terminal_size;

pub struct RuntimeApp {
    template: Template,
    config: Config,
    actions: HashMap<KeyBinding, Action>,
    panes: HashMap<u64, PaneProcess>,
    active_id: u64,
    prompt: Option<InputPrompt>,
    status: String,
}

impl RuntimeApp {
    pub fn new(template: Template, config: Config) -> Result<Self> {
        let actions = actions_from_config(&config.keybinds);
        let mut app = Self {
            active_id: template.layout.id(),
            template,
            config,
            actions,
            panes: HashMap::new(),
            prompt: None,
            status: "".to_string(),
        };
        app.spawn_all()?;
        Ok(app)
    }

    fn spawn_all(&mut self) -> Result<()> {
        let full = terminal_size();
        let main = main_area(full);
        let mut rects = Vec::new();
        layout_rects(&self.template.layout, main, &mut rects);

        for (id, rect) in rects {
            if let Some(Node::Bite { command, .. }) =
                crate::layout::find_bite(&self.template.layout, id)
            {
                let pty_size = PtySize {
                    rows: rect.height.saturating_sub(2),
                    cols: rect.width.saturating_sub(2),
                    pixel_width: 0,
                    pixel_height: 0,
                };
                let pane = PaneProcess::spawn(command.clone(), pty_size)?;
                self.panes.insert(id, pane);
            }
        }
        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
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

    fn loop_ui(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if event::poll(Duration::from_millis(30))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key)? {
                            break;
                        }
                    }
                    Event::Resize(_, _) => {
                        self.resize_all(terminal.size()?);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
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

        for (id, rect) in rects.iter() {
            let (title, highlight) = match crate::layout::find_bite(&self.template.layout, *id) {
                Some(Node::Bite { name, .. }) => (name.clone(), *id == self.active_id),
                _ => ("".to_string(), *id == self.active_id),
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
            let inner = block.inner(*rect);
            f.render_widget(block, *rect);

            if let Some(pane) = self.panes.get(id) {
                let height = rect.height.saturating_sub(2) as usize;
                let lines = pane.lines_for_height(height);
                let text = Text::from(lines.into_iter().map(Line::from).collect::<Vec<_>>());
                f.render_widget(Paragraph::new(text), inner);
            }
        }

        let mut status_line = vec![Span::raw("[pudding] ")];
        if let Some(Node::Bite { name, .. }) =
            crate::layout::find_bite(&self.template.layout, self.active_id)
        {
            status_line.push(Span::raw(format!("active: {}  ", name)));
        }
        if !self.status.is_empty() {
            status_line.push(Span::raw(self.status.clone()));
        }
        let status_widget = Paragraph::new(Line::from(status_line));
        f.render_widget(status_widget, status);

        if let Some(prompt) = &self.prompt {
            let label = format!("{}: ", prompt.label);
            let line = Line::from(vec![Span::raw(&label), Span::raw(&prompt.buffer)]);
            let area = centered_rect(80, 3, area);
            let block = Block::default().borders(Borders::ALL).title("Input");
            f.render_widget(block, area);
            f.render_widget(
                Paragraph::new(line),
                ratatui::layout::Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width - 2,
                    height: 1,
                },
            );
            f.set_cursor(
                area.x
                    + 1
                    + UnicodeWidthStr::width(label.as_str()) as u16
                    + UnicodeWidthStr::width(prompt.buffer.as_str()) as u16,
                area.y + 1,
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.is_quit_key(key) {
            return Ok(true);
        }

        if let Some(mut prompt) = self.prompt.take() {
            let close = self.handle_prompt_key(&mut prompt, key);
            if !close {
                self.prompt = Some(prompt);
            }
            return Ok(false);
        }

        for (binding, action) in self.actions.iter() {
            if binding.matches(key) {
                return Ok(self.handle_action(*action));
            }
        }

        if let Some(pane) = self.panes.get_mut(&self.active_id) {
            if let Some(bytes) = key_to_bytes(key) {
                pane.write_bytes(&bytes);
            }
        }

        Ok(false)
    }
}
