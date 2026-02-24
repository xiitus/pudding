use ratatui::layout::{Constraint, Direction, Layout, Rect};

const TAB_BAR_HEIGHT: u16 = 1;
const STATUS_BAR_HEIGHT: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EditorAreas {
    pub tab_bar: Rect,
    pub main: Rect,
    pub status: Rect,
}

pub(super) fn editor_areas(area: Rect) -> EditorAreas {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TAB_BAR_HEIGHT),
            Constraint::Min(1),
            Constraint::Length(STATUS_BAR_HEIGHT),
        ])
        .split(area);
    EditorAreas {
        tab_bar: chunks[0],
        main: chunks[1],
        status: chunks[2],
    }
}

pub(super) fn main_area(area: Rect) -> Rect {
    editor_areas(area).main
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
    use super::{centered_rect, editor_areas, main_area};
    use ratatui::layout::Rect;

    #[test]
    fn main_area_reserves_tab_and_status_bar() {
        assert_eq!(main_area(Rect::new(0, 0, 60, 20)), Rect::new(0, 1, 60, 17));
    }

    #[test]
    fn editor_areas_split_screen() {
        let areas = editor_areas(Rect::new(0, 0, 60, 20));
        assert_eq!(areas.tab_bar, Rect::new(0, 0, 60, 1));
        assert_eq!(areas.main, Rect::new(0, 1, 60, 17));
        assert_eq!(areas.status, Rect::new(0, 18, 60, 2));
    }

    #[test]
    fn centered_rect_is_centered() {
        assert_eq!(
            centered_rect(10, 4, Rect::new(0, 0, 30, 14)),
            Rect::new(10, 5, 10, 4)
        );
    }
}
