use ratatui::layout::{Constraint, Direction, Layout, Rect};

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
    use super::centered_rect;
    use ratatui::layout::Rect;

    #[test]
    fn keeps_requested_size_when_space_is_enough() {
        let area = Rect::new(0, 0, 100, 40);
        let centered = centered_rect(20, 10, area);
        assert_eq!(centered.width, 20);
        assert_eq!(centered.height, 10);
        assert_eq!(centered.x, 40);
        assert_eq!(centered.y, 15);
    }

    #[test]
    fn saturates_to_available_size_when_request_is_too_large() {
        let area = Rect::new(0, 0, 10, 4);
        let centered = centered_rect(20, 10, area);
        assert_eq!(centered.width, 10);
        assert_eq!(centered.height, 4);
        assert_eq!(centered.x, 0);
        assert_eq!(centered.y, 0);
    }
}
