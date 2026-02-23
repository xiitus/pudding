use std::collections::HashMap;

use crate::keybind::{parse_keybinding, KeyBinding};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    SplitVertical,
    SplitHorizontal,
    ResizeLeft,
    ResizeRight,
    ResizeUp,
    ResizeDown,
    SwapVertical,
    SwapHorizontal,
    SaveState,
    RestoreState,
    FocusNext,
    Quit,
}

pub fn actions_from_config(map: &HashMap<String, String>) -> HashMap<KeyBinding, Action> {
    let mut out = HashMap::new();
    insert_action(map, &mut out, "split_vertical", Action::SplitVertical);
    insert_action(map, &mut out, "split_horizontal", Action::SplitHorizontal);
    insert_action(map, &mut out, "resize_left", Action::ResizeLeft);
    insert_action(map, &mut out, "resize_right", Action::ResizeRight);
    insert_action(map, &mut out, "resize_up", Action::ResizeUp);
    insert_action(map, &mut out, "resize_down", Action::ResizeDown);
    insert_action(map, &mut out, "swap_vertical", Action::SwapVertical);
    insert_action(map, &mut out, "swap_horizontal", Action::SwapHorizontal);
    insert_action(map, &mut out, "save_state", Action::SaveState);
    insert_action(map, &mut out, "restore_state", Action::RestoreState);
    insert_action(map, &mut out, "focus_next", Action::FocusNext);
    insert_action(map, &mut out, "quit", Action::Quit);
    out
}

fn insert_action(map: &HashMap<String, String>, out: &mut HashMap<KeyBinding, Action>, key: &str, action: Action) {
    if let Some(value) = map.get(key) {
        if let Some(binding) = parse_keybinding(value) {
            out.insert(binding, action);
        }
    }
}
