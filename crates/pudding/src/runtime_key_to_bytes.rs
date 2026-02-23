use crossterm::event::{KeyCode, KeyEvent};

pub(super) fn key_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    match key.code {
        KeyCode::Char(c) if c.is_ascii() => Some(vec![c as u8]),
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::key_to_bytes;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn converts_ascii_char() {
        assert_eq!(key_to_bytes(key(KeyCode::Char('a'))), Some(vec![b'a']));
    }

    #[test]
    fn rejects_non_ascii_char() {
        assert_eq!(key_to_bytes(key(KeyCode::Char('あ'))), None);
    }

    #[test]
    fn converts_arrow_key() {
        assert_eq!(key_to_bytes(key(KeyCode::Left)), Some(b"\x1b[D".to_vec()));
    }
}
