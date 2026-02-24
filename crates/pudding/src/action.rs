use std::collections::HashMap;

use crate::keybind::{parse_keybinding, KeyBinding};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorUp,
    MoveCursorDown,
    SplitVertical,
    SplitHorizontal,
    EditPaneCommand,
    DeletePane,
    AddTab,
    RenameTab,
    NextTab,
    PrevTab,
    Save,
    Quit,
}

pub fn actions_from_config(map: &HashMap<String, String>) -> HashMap<KeyBinding, Action> {
    let mut out = HashMap::new();

    insert_action(map, &mut out, "cursor_left", Action::MoveCursorLeft);
    insert_action(map, &mut out, "cursor_right", Action::MoveCursorRight);
    insert_action(map, &mut out, "cursor_up", Action::MoveCursorUp);
    insert_action(map, &mut out, "cursor_down", Action::MoveCursorDown);
    insert_action(map, &mut out, "split_vertical", Action::SplitVertical);
    insert_action(map, &mut out, "split_horizontal", Action::SplitHorizontal);
    insert_action(map, &mut out, "pane_command", Action::EditPaneCommand);
    insert_action(map, &mut out, "delete_pane", Action::DeletePane);
    insert_action(map, &mut out, "add_tab", Action::AddTab);
    insert_action(map, &mut out, "rename_tab", Action::RenameTab);
    insert_action(map, &mut out, "next_tab", Action::NextTab);
    insert_action(map, &mut out, "prev_tab", Action::PrevTab);
    insert_action(map, &mut out, "save", Action::Save);
    insert_action(map, &mut out, "quit", Action::Quit);

    insert_default(&mut out, "left", Action::MoveCursorLeft);
    insert_default(&mut out, "right", Action::MoveCursorRight);
    insert_default(&mut out, "up", Action::MoveCursorUp);
    insert_default(&mut out, "down", Action::MoveCursorDown);
    insert_default(&mut out, "v", Action::SplitVertical);
    insert_default(&mut out, "h", Action::SplitHorizontal);
    insert_default(&mut out, "c", Action::EditPaneCommand);
    insert_default(&mut out, "d", Action::DeletePane);
    insert_default(&mut out, "T", Action::AddTab);
    insert_default(&mut out, "n", Action::RenameTab);
    insert_default(&mut out, "tab", Action::NextTab);
    insert_default(&mut out, "shift+tab", Action::PrevTab);
    insert_default(&mut out, "s", Action::Save);
    insert_default(&mut out, "q", Action::Quit);

    out
}

fn insert_action(
    map: &HashMap<String, String>,
    out: &mut HashMap<KeyBinding, Action>,
    key: &str,
    action: Action,
) {
    if let Some(value) = map.get(key) {
        if let Some(binding) = parse_keybinding(value) {
            out.insert(binding, action);
        }
    }
}

fn insert_default(out: &mut HashMap<KeyBinding, Action>, key: &str, action: Action) {
    if let Some(binding) = parse_keybinding(key) {
        out.entry(binding).or_insert(action);
    }
}
