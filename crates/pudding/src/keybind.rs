use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn matches(&self, ev: KeyEvent) -> bool {
        let (lhs_code, lhs_modifiers) = normalize(self.code, self.modifiers);
        let (rhs_code, rhs_modifiers) = normalize(ev.code, ev.modifiers);
        lhs_code == rhs_code && lhs_modifiers == rhs_modifiers
    }
}

pub fn parse_keybinding(input: &str) -> Option<KeyBinding> {
    let parts: Vec<&str> = input.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = KeyModifiers::empty();
    let key_part = parts[parts.len() - 1].trim();
    for part in &parts[..parts.len().saturating_sub(1)] {
        match part.trim().to_lowercase().as_str() {
            "ctrl" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            _ => return None,
        }
    }

    if key_part.len() == 1 {
        let ch = key_part.chars().next()?;
        let mut mods = modifiers;
        if ch.is_uppercase() && !mods.contains(KeyModifiers::SHIFT) {
            mods |= KeyModifiers::SHIFT;
        }
        return Some(KeyBinding {
            code: KeyCode::Char(ch),
            modifiers: mods,
        });
    }

    let lower = key_part.to_lowercase();
    let mut key = match lower.as_str() {
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "enter" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "insert" => KeyCode::Insert,
        "space" => KeyCode::Char(' '),
        _ if lower.starts_with('f') => {
            let num = lower.trim_start_matches('f').parse::<u8>().ok()?;
            KeyCode::F(num)
        }
        _ => return None,
    };

    if key == KeyCode::BackTab {
        key = KeyCode::Tab;
        modifiers |= KeyModifiers::SHIFT;
    }

    Some(KeyBinding {
        code: key,
        modifiers,
    })
}

fn normalize(code: KeyCode, modifiers: KeyModifiers) -> (KeyCode, KeyModifiers) {
    if code == KeyCode::BackTab {
        return (KeyCode::Tab, modifiers | KeyModifiers::SHIFT);
    }
    (code, modifiers)
}
