use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(super) fn main_area(area: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(area);
    chunks[0]
}

pub(super) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
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

#[cfg(test)]
mod tests {
    use super::{centered_rect, main_area};
    use ratatui::layout::Rect;

    #[test]
    fn main_area_reserves_help_bar() {
        assert_eq!(main_area(Rect::new(0, 0, 60, 20)), Rect::new(0, 0, 60, 18));
    }

    #[test]
    fn centered_rect_is_centered() {
        assert_eq!(
            centered_rect(10, 4, Rect::new(0, 0, 30, 14)),
            Rect::new(10, 5, 10, 4)
        );
    }
}
