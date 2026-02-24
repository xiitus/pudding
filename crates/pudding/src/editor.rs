use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::model::Template;

mod editor_area;
mod editor_draw;
mod editor_input;
mod editor_keys;

use editor_input::InputMode;

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
}
