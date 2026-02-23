use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(super) fn main_area(area: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(area);
    chunks[0]
}

#[cfg(test)]
mod tests {
    use super::main_area;
    use ratatui::layout::Rect;

    #[test]
    fn reserves_two_rows_for_help_bar() {
        let area = Rect::new(0, 0, 80, 24);
        let main = main_area(area);
        assert_eq!(main, Rect::new(0, 0, 80, 22));
    }
}
