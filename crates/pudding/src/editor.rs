use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};

mod editor_area;

use self::editor_area::{centered_rect, editor_areas, main_area};

use crate::{
    layout::{
        collect_panes, delete_node, find_node_mut, find_pane_at, layout_rects, split_node,
        DeleteNodeError,
    },
    model::{Direction, Layout, Node, Tab},
    template, zellij,
};

enum InputMode {
    PaneCommand { pane_id: u64, buffer: String },
    TabName { buffer: String },
    ConfirmDelete { pane_id: u64 },
    ConfirmQuit,
}

pub struct EditorApp {
    layout: Layout,
    name: String,
    cursor: (u16, u16),
    input_mode: Option<InputMode>,
    dirty: bool,
    status_msg: String,
}

impl EditorApp {
    pub fn new(layout: Layout, name: String) -> Self {
        Self {
            layout,
            name,
            cursor: (0, 0),
            input_mode: None,
            dirty: false,
            status_msg: "←↑↓→:移動 v/h:分割 c:コマンド d:削除 T:タブ追加 n:改名 s:保存 q:終了"
                .to_string(),
        }
    }

    pub fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.loop_ui(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn loop_ui(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key, terminal.size()?)? {
                            break;
                        }
                    }
                    Event::Resize(_, _) => {
                        self.reset_cursor(main_area(terminal.size()?));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut ratatui::Frame) {
        self.ensure_layout_invariants();

        let area = f.size();
        let areas = editor_areas(area);
        self.clamp_cursor(areas.main);
        let active_pane = self.active_pane_id(areas.main);

        self.draw_tab_bar(f, areas.tab_bar);
        self.draw_main(f, areas.main, active_pane);
        self.draw_status(f, areas.status, active_pane);
        self.draw_modal(f, area);
    }

    fn draw_tab_bar(&self, f: &mut ratatui::Frame, area: Rect) {
        let mut spans = Vec::new();
        for (idx, tab) in self.layout.tabs.iter().enumerate() {
            if idx > 0 {
                spans.push(Span::raw(" "));
            }
            let style = if idx == self.layout.active_tab {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            spans.push(Span::styled(format!("[{}]", tab.name), style));
        }
        if spans.is_empty() {
            spans.push(Span::raw("[tab-1]"));
        }
        f.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn draw_main(&self, f: &mut ratatui::Frame, area: Rect, active_pane: Option<u64>) {
        let mut rects = Vec::new();
        if let Some(tab) = self.active_tab() {
            layout_rects(&tab.root, area, &mut rects);
        }

        for (pane_id, rect) in &rects {
            let style = if active_pane == Some(*pane_id) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            let block = Block::default()
                .title(format!("pane-{}", pane_id))
                .borders(Borders::ALL)
                .border_style(style);
            f.render_widget(block, *rect);
        }

        if area.width > 0 && area.height > 0 {
            let x = self.cursor.0.clamp(area.x, area.x + area.width - 1);
            let y = self.cursor.1.clamp(area.y, area.y + area.height - 1);
            let cell = f.buffer_mut().get_mut(x, y);
            cell.set_symbol("x");
            cell.set_style(Style::default().fg(Color::Cyan));
        }
    }

    fn draw_status(&self, f: &mut ratatui::Frame, area: Rect, active_pane: Option<u64>) {
        let tab_name = self
            .active_tab()
            .map(|tab| tab.name.as_str())
            .unwrap_or("tab-1");
        let pane_name = active_pane
            .map(|id| format!("pane-{}", id))
            .unwrap_or_else(|| "-".to_string());
        let dirty_label = if self.dirty {
            "未保存"
        } else {
            "保存済み"
        };

        let line1 = Line::from(vec![
            Span::raw("[pudding] "),
            Span::raw(format!("tab:{} ", tab_name)),
            Span::raw(format!("active:{} ", pane_name)),
            Span::raw(format!("state:{} ", dirty_label)),
            Span::raw(format!("{} ", self.status_msg)),
        ]);
        let line2 = Line::from(vec![Span::raw(
            "Tab/Shift+Tab:切替  T:追加  n:改名  v/h:分割  c:コマンド  d:削除  s:保存  q:終了",
        )]);
        f.render_widget(Paragraph::new(vec![line1, line2]), area);
    }

    fn draw_modal(&self, f: &mut ratatui::Frame, area: Rect) {
        let Some(mode) = &self.input_mode else {
            return;
        };

        let width = area.width.saturating_sub(6).min(80);
        let dialog = centered_rect(width.max(20), 5, area);
        f.render_widget(Clear, dialog);

        let title = match mode {
            InputMode::PaneCommand { .. } => "ペインコマンド",
            InputMode::TabName { .. } => "タブ名変更",
            InputMode::ConfirmDelete { .. } => "削除確認",
            InputMode::ConfirmQuit => "終了確認",
        };
        f.render_widget(Block::default().title(title).borders(Borders::ALL), dialog);

        match mode {
            InputMode::PaneCommand { buffer, .. } => {
                let prompt = "コマンド: ";
                let line = Line::from(vec![Span::raw(prompt), Span::raw(buffer)]);
                let inner = Rect::new(
                    dialog.x + 1,
                    dialog.y + 2,
                    dialog.width.saturating_sub(2),
                    1,
                );
                f.render_widget(Paragraph::new(line), inner);
                f.set_cursor(inner.x + prompt.len() as u16 + buffer.len() as u16, inner.y);
            }
            InputMode::TabName { buffer } => {
                let prompt = "新しいタブ名: ";
                let line = Line::from(vec![Span::raw(prompt), Span::raw(buffer)]);
                let inner = Rect::new(
                    dialog.x + 1,
                    dialog.y + 2,
                    dialog.width.saturating_sub(2),
                    1,
                );
                f.render_widget(Paragraph::new(line), inner);
                f.set_cursor(inner.x + prompt.len() as u16 + buffer.len() as u16, inner.y);
            }
            InputMode::ConfirmDelete { .. } => {
                let line = Line::from("このペインを削除しますか？ (y/n)");
                f.render_widget(
                    Paragraph::new(line),
                    Rect::new(
                        dialog.x + 1,
                        dialog.y + 2,
                        dialog.width.saturating_sub(2),
                        1,
                    ),
                );
            }
            InputMode::ConfirmQuit => {
                let line = Line::from("保存せず終了しますか？ (y/n)");
                f.render_widget(
                    Paragraph::new(line),
                    Rect::new(
                        dialog.x + 1,
                        dialog.y + 2,
                        dialog.width.saturating_sub(2),
                        1,
                    ),
                );
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent, area: Rect) -> Result<bool> {
        self.ensure_layout_invariants();
        let main = main_area(area);
        self.clamp_cursor(main);

        if self.input_mode.is_some() {
            return self.handle_input_mode_key(key, main);
        }

        match key.code {
            KeyCode::Left => self.move_cursor_left(main),
            KeyCode::Right => self.move_cursor_right(main),
            KeyCode::Up => self.move_cursor_up(main),
            KeyCode::Down => self.move_cursor_down(main),
            KeyCode::Char('v') => self.split_active(main, Direction::Vertical),
            KeyCode::Char('h') => self.split_active(main, Direction::Horizontal),
            KeyCode::Char('c') => self.open_command_input(main),
            KeyCode::Char('d') => self.open_delete_confirm(main),
            KeyCode::Char('n') => self.open_tab_name_input(),
            KeyCode::Char('T') => self.add_tab(main),
            KeyCode::BackTab => self.switch_tab(-1, main),
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.switch_tab(-1, main)
            }
            KeyCode::Tab => self.switch_tab(1, main),
            KeyCode::Char('s') => self.save_layout(),
            KeyCode::Char('q') => {
                if self.dirty {
                    self.input_mode = Some(InputMode::ConfirmQuit);
                } else {
                    return Ok(true);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_input_mode_key(&mut self, key: KeyEvent, main: Rect) -> Result<bool> {
        let Some(mode) = self.input_mode.take() else {
            return Ok(false);
        };

        match mode {
            InputMode::PaneCommand {
                pane_id,
                mut buffer,
            } => match key.code {
                KeyCode::Enter => {
                    self.apply_pane_command(pane_id, &buffer);
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    buffer.pop();
                    self.input_mode = Some(InputMode::PaneCommand { pane_id, buffer });
                }
                KeyCode::Char(c) => {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        buffer.push(c);
                    }
                    self.input_mode = Some(InputMode::PaneCommand { pane_id, buffer });
                }
                _ => {
                    self.input_mode = Some(InputMode::PaneCommand { pane_id, buffer });
                }
            },
            InputMode::TabName { mut buffer } => match key.code {
                KeyCode::Enter => self.apply_tab_name(&buffer),
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    buffer.pop();
                    self.input_mode = Some(InputMode::TabName { buffer });
                }
                KeyCode::Char(c) => {
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        buffer.push(c);
                    }
                    self.input_mode = Some(InputMode::TabName { buffer });
                }
                _ => {
                    self.input_mode = Some(InputMode::TabName { buffer });
                }
            },
            InputMode::ConfirmDelete { pane_id } => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.confirm_delete(pane_id, main);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {}
                _ => {
                    self.input_mode = Some(InputMode::ConfirmDelete { pane_id });
                }
            },
            InputMode::ConfirmQuit => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {}
                _ => self.input_mode = Some(InputMode::ConfirmQuit),
            },
        }

        Ok(false)
    }

    fn split_active(&mut self, main: Rect, direction: Direction) {
        let Some(target_id) = self.active_pane_id(main) else {
            self.status_msg = "分割対象のペインがありません".to_string();
            return;
        };
        if let Some(tab) = self.active_tab_mut() {
            if split_node(&mut tab.root, target_id, direction, 0.5).is_some() {
                self.dirty = true;
                self.status_msg = "分割しました".to_string();
            } else {
                self.status_msg = "分割に失敗しました".to_string();
            }
        }
    }

    fn open_command_input(&mut self, main: Rect) {
        let Some(pane_id) = self.active_pane_id(main) else {
            self.status_msg = "コマンド編集対象のペインがありません".to_string();
            return;
        };
        self.input_mode = Some(InputMode::PaneCommand {
            pane_id,
            buffer: String::new(),
        });
    }

    fn apply_pane_command(&mut self, pane_id: u64, buffer: &str) {
        if let Some(tab) = self.active_tab_mut() {
            if let Some(Node::Pane { command, .. }) = find_node_mut(&mut tab.root, pane_id) {
                let input = buffer.trim();
                *command = if input.is_empty() {
                    None
                } else {
                    Some(input.to_string())
                };
                self.dirty = true;
                self.status_msg = "ペインコマンドを更新しました".to_string();
            } else {
                self.status_msg = "対象ペインが見つかりません".to_string();
            }
        }
    }

    fn open_delete_confirm(&mut self, main: Rect) {
        let Some(tab) = self.active_tab() else {
            self.status_msg = "削除対象のペインがありません".to_string();
            return;
        };

        let mut panes = Vec::new();
        collect_panes(&tab.root, &mut panes);
        if panes.len() <= 1 {
            self.status_msg = "削除できません（最後のペイン）".to_string();
            return;
        }

        if let Some(pane_id) = self.active_pane_id(main) {
            self.input_mode = Some(InputMode::ConfirmDelete { pane_id });
        } else {
            self.status_msg = "削除対象のペインがありません".to_string();
        }
    }

    fn confirm_delete(&mut self, pane_id: u64, main: Rect) {
        if let Some(tab) = self.active_tab_mut() {
            match delete_node(&mut tab.root, pane_id) {
                Ok(()) => {
                    self.dirty = true;
                    self.status_msg = "ペインを削除しました".to_string();
                    self.reset_cursor(main);
                }
                Err(DeleteNodeError::LastPane) => {
                    self.status_msg = "削除できません（最後のペイン）".to_string();
                }
                Err(DeleteNodeError::NotFound) => {
                    self.status_msg = "ペイン削除に失敗しました".to_string();
                }
            }
        }
    }

    fn open_tab_name_input(&mut self) {
        let current = self
            .active_tab()
            .map(|tab| tab.name.clone())
            .unwrap_or_default();
        self.input_mode = Some(InputMode::TabName { buffer: current });
    }

    fn apply_tab_name(&mut self, buffer: &str) {
        let name = buffer.trim();
        if name.is_empty() {
            self.status_msg = "タブ名は空にできません".to_string();
            return;
        }
        if let Some(tab) = self.active_tab_mut() {
            tab.name = name.to_string();
            self.dirty = true;
            self.status_msg = "タブ名を変更しました".to_string();
        }
    }

    fn switch_tab(&mut self, step: i32, main: Rect) {
        if self.layout.tabs.is_empty() {
            return;
        }
        let len = self.layout.tabs.len() as i32;
        let current = self.layout.active_tab as i32;
        self.layout.active_tab = (current + step).rem_euclid(len) as usize;
        self.reset_cursor(main);
        if let Some(tab) = self.active_tab() {
            self.status_msg = format!("タブ切替: {}", tab.name);
        }
    }

    fn add_tab(&mut self, main: Rect) {
        let id = self.next_layout_id();
        let tab_no = self.layout.tabs.len() + 1;
        self.layout.tabs.push(Tab {
            name: format!("tab-{}", tab_no),
            root: Node::Pane { id, command: None },
        });
        self.layout.active_tab = self.layout.tabs.len() - 1;
        self.dirty = true;
        self.status_msg = format!("タブを追加しました: tab-{}", tab_no);
        self.reset_cursor(main);
    }

    fn save_layout(&mut self) {
        match template::save(&self.name, &self.layout) {
            Ok(path) => {
                self.dirty = false;
                let in_session = zellij::is_in_zellij_session();
                if let Some(err_msg) = zellij::apply_layout_if_in_session(path.as_path(), false) {
                    self.status_msg = format!("保存しました（zellij反映失敗: {}）", err_msg);
                } else if in_session {
                    self.status_msg = "保存しました（zellij に反映）".to_string();
                } else {
                    self.status_msg = "保存しました".to_string();
                }
            }
            Err(err) => {
                self.status_msg = format!("保存に失敗しました: {}", err);
            }
        }
    }

    fn active_pane_id(&self, main: Rect) -> Option<u64> {
        let tab = self.active_tab()?;
        let mut panes = Vec::new();
        collect_panes(&tab.root, &mut panes);
        if panes.is_empty() {
            return None;
        }

        if main.width == 0 || main.height == 0 {
            return panes.into_iter().next();
        }

        find_pane_at(&tab.root, main, self.cursor.0, self.cursor.1)
            .or_else(|| panes.into_iter().next())
    }

    fn ensure_layout_invariants(&mut self) {
        if self.layout.tabs.is_empty() {
            self.layout.tabs.push(Tab {
                name: "tab-1".to_string(),
                root: Node::Pane {
                    id: self.next_layout_id(),
                    command: None,
                },
            });
            self.layout.active_tab = 0;
        }
        if self.layout.active_tab >= self.layout.tabs.len() {
            self.layout.active_tab = 0;
        }
    }

    fn active_tab(&self) -> Option<&Tab> {
        self.layout.tabs.get(self.layout.active_tab)
    }

    fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.layout.tabs.get_mut(self.layout.active_tab)
    }

    fn next_layout_id(&self) -> u64 {
        let max_id = self
            .layout
            .tabs
            .iter()
            .map(|tab| max_node_id(&tab.root))
            .max()
            .unwrap_or(0);
        max_id.saturating_add(1)
    }

    fn reset_cursor(&mut self, main: Rect) {
        if main.width == 0 || main.height == 0 {
            self.cursor = (main.x, main.y);
            return;
        }
        self.cursor = (main.x, main.y);
    }

    fn clamp_cursor(&mut self, main: Rect) {
        if main.width == 0 || main.height == 0 {
            self.cursor = (main.x, main.y);
            return;
        }
        self.cursor.0 = self.cursor.0.clamp(main.x, main.x + main.width - 1);
        self.cursor.1 = self.cursor.1.clamp(main.y, main.y + main.height - 1);
    }

    fn move_cursor_left(&mut self, main: Rect) {
        if self.cursor.0 > main.x {
            self.cursor.0 -= 1;
        }
    }

    fn move_cursor_right(&mut self, main: Rect) {
        if main.width == 0 {
            return;
        }
        let max_x = main.x + main.width - 1;
        if self.cursor.0 < max_x {
            self.cursor.0 += 1;
        }
    }

    fn move_cursor_up(&mut self, main: Rect) {
        if self.cursor.1 > main.y {
            self.cursor.1 -= 1;
        }
    }

    fn move_cursor_down(&mut self, main: Rect) {
        if main.height == 0 {
            return;
        }
        let max_y = main.y + main.height - 1;
        if self.cursor.1 < max_y {
            self.cursor.1 += 1;
        }
    }
}

fn max_node_id(node: &Node) -> u64 {
    match node {
        Node::Pane { id, .. } => *id,
        Node::Split {
            id, first, second, ..
        } => (*id).max(max_node_id(first)).max(max_node_id(second)),
    }
}
