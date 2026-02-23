use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn matches(&self, ev: KeyEvent) -> bool {
        self.code == ev.code && self.modifiers == ev.modifiers
    }
}

pub fn parse_keybinding(input: &str) -> Option<KeyBinding> {
    let parts: Vec<&str> = input.split('+').collect();
    if parts.is_empty() {
        return None;
    }
    let mut modifiers = KeyModifiers::empty();
    let mut key_part = parts[parts.len() - 1].trim();
    for part in &parts[..parts.len().saturating_sub(1)] {
        match part.trim().to_lowercase().as_str() {
            "ctrl" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            _ => {}
        }
    }

    if key_part.len() == 1 {
        let ch = key_part.chars().next().unwrap();
        let mut mods = modifiers;
        if ch.is_uppercase() && !mods.contains(KeyModifiers::SHIFT) {
            mods |= KeyModifiers::SHIFT;
        }
        let code = KeyCode::Char(ch);
        return Some(KeyBinding { code, modifiers: mods });
    }

    let lower = key_part.to_lowercase();
    let code = match lower.as_str() {
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "enter" => KeyCode::Enter,
        "esc" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        _ => {
            if lower.starts_with('f') {
                let num = lower.trim_start_matches('f').parse::<u8>().ok()?;
                KeyCode::F(num)
            } else {
                return None;
            }
        }
    };

    Some(KeyBinding { code, modifiers })
}
