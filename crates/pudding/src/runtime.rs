use std::{
    collections::{HashMap, VecDeque},
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{CommandBuilder, PtySize};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::{
    action::{actions_from_config, Action},
    config::Config,
    keybind::KeyBinding,
    layout::{
        collect_bites, layout_rects, next_id, resize_from_bite, split_bite, swap_adjacent_bites,
    },
    model::{Node, Orientation, Template},
    template::{load_state, save_state},
};

#[path = "runtime_centered_rect.rs"]
mod runtime_centered_rect;
#[path = "runtime_key_to_bytes.rs"]
mod runtime_key_to_bytes;
#[path = "runtime_main_area.rs"]
mod runtime_main_area;
#[path = "runtime_terminal_size.rs"]
mod runtime_terminal_size;

use runtime_centered_rect::centered_rect;
use runtime_key_to_bytes::key_to_bytes;
use runtime_main_area::main_area;
use runtime_terminal_size::terminal_size;

const OUTPUT_LIMIT: usize = 2000;
const PENDING_CHAR_LIMIT: usize = 8192;
const RESIZE_STEP_RATIO: f32 = 0.20;

struct PaneProcess {
    master: Box<dyn portable_pty::MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn portable_pty::Child + Send>,
    output: Arc<Mutex<VecDeque<String>>>,
}

impl PaneProcess {
    fn spawn(command: String, size: PtySize) -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system.openpty(size)?;
        let mut cmd = CommandBuilder::new(&command);
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let output = Arc::new(Mutex::new(VecDeque::new()));
        let output_clone = output.clone();

        thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            let mut pending = String::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]);
                        let stripped = strip_ansi_escapes::strip(chunk.as_bytes());
                        let text = String::from_utf8_lossy(&stripped).replace('\r', "");
                        let mut combined = String::new();
                        std::mem::swap(&mut combined, &mut pending);
                        combined.push_str(&text);
                        let mut lines: Vec<&str> = combined.split('\n').collect();
                        let last = lines.pop().unwrap_or("");
                        pending = last.to_string();
                        if pending.chars().count() > PENDING_CHAR_LIMIT {
                            pending = pending
                                .chars()
                                .rev()
                                .take(PENDING_CHAR_LIMIT)
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect();
                        }
                        let mut guard = output_clone.lock().unwrap();
                        for line in lines {
                            guard.push_back(line.to_string());
                            if guard.len() > OUTPUT_LIMIT {
                                guard.pop_front();
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            master: pair.master,
            writer,
            _child: child,
            output,
        })
    }

    fn resize(&mut self, rows: u16, cols: u16) {
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        let _ = self.writer.write_all(bytes);
        let _ = self.writer.flush();
    }

    fn lines_for_height(&self, height: usize) -> Vec<String> {
        let guard = self.output.lock().unwrap();
        let total = guard.len();
        let start = total.saturating_sub(height);
        guard.iter().skip(start).cloned().collect()
    }
}

struct InputPrompt {
    label: String,
    buffer: String,
    mode: PromptMode,
}

#[derive(Clone, Copy)]
enum PromptMode {
    Save,
    Restore,
}

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
                area.x + 1 + label.len() as u16 + prompt.buffer.len() as u16,
                area.y + 1,
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
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

        // send to active pane
        if let Some(pane) = self.panes.get_mut(&self.active_id) {
            if let Some(bytes) = key_to_bytes(key) {
                pane.write_bytes(&bytes);
            }
        }

        Ok(false)
    }

    fn handle_action(&mut self, action: Action) -> bool {
        match action {
            Action::SplitVertical => {
                self.split_active(Orientation::Vertical);
            }
            Action::SplitHorizontal => {
                self.split_active(Orientation::Horizontal);
            }
            Action::ResizeLeft => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Vertical,
                    -RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::ResizeRight => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Vertical,
                    RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::ResizeUp => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Horizontal,
                    -RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::ResizeDown => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Horizontal,
                    RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::SwapVertical => {
                let _ = swap_adjacent_bites(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Vertical,
                );
            }
            Action::SwapHorizontal => {
                let _ = swap_adjacent_bites(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Horizontal,
                );
            }
            Action::SaveState => {
                self.prompt = Some(InputPrompt {
                    label: "保存名".to_string(),
                    buffer: String::new(),
                    mode: PromptMode::Save,
                });
            }
            Action::RestoreState => {
                self.prompt = Some(InputPrompt {
                    label: "復元名".to_string(),
                    buffer: String::new(),
                    mode: PromptMode::Restore,
                });
            }
            Action::FocusNext => {
                self.focus_next();
            }
            Action::Quit => return true,
        }
        false
    }

    fn handle_prompt_key(&mut self, prompt: &mut InputPrompt, key: KeyEvent) -> bool {
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

    fn split_active(&mut self, orientation: Orientation) {
        let new_id = next_id(&self.template.layout);
        let did = split_bite(
            &mut self.template.layout,
            self.active_id,
            orientation,
            0.5,
            new_id,
            &self.config.default_command,
        );
        if did {
            let full = terminal_size();
            let size = main_area(full);
            let mut rects = Vec::new();
            layout_rects(&self.template.layout, size, &mut rects);
            if let Some((_, rect)) = rects.iter().find(|(id, _)| *id == new_id) {
                if let Some(Node::Bite { command, .. }) =
                    crate::layout::find_bite(&self.template.layout, new_id)
                {
                    let pane = PaneProcess::spawn(
                        command.clone(),
                        PtySize {
                            rows: rect.height.saturating_sub(2),
                            cols: rect.width.saturating_sub(2),
                            pixel_width: 0,
                            pixel_height: 0,
                        },
                    );
                    if let Ok(pane) = pane {
                        self.panes.insert(new_id, pane);
                    }
                }
            }
            self.resize_all(full);
        }
    }

    fn focus_next(&mut self) {
        let mut ids = Vec::new();
        collect_bites(&self.template.layout, &mut ids);
        if ids.is_empty() {
            return;
        }
        if let Some(pos) = ids.iter().position(|id| *id == self.active_id) {
            let next = (pos + 1) % ids.len();
            self.active_id = ids[next];
        } else {
            self.active_id = ids[0];
        }
    }

    fn resize_all(&mut self, area: ratatui::layout::Rect) {
        let area = main_area(area);
        let mut rects = Vec::new();
        layout_rects(&self.template.layout, area, &mut rects);
        for (id, rect) in rects {
            if let Some(pane) = self.panes.get_mut(&id) {
                pane.resize(rect.height.saturating_sub(2), rect.width.saturating_sub(2));
            }
        }
    }
}
