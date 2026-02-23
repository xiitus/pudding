pub(super) fn terminal_size() -> ratatui::layout::Rect {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: cols,
        height: rows,
    }
}
