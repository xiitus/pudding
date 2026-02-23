use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(super) fn main_area(area: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(area);
    chunks[0]
}
