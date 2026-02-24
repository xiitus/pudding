use crate::{
    action::Action,
    layout::{
        collect_bites, layout_rects, next_id, resize_from_bite, split_bite, swap_adjacent_bites,
    },
    model::{Node, Orientation},
};
use portable_pty::PtySize;

use super::{main_area, terminal_size, InputPrompt, PaneProcess, RuntimeApp};

const RESIZE_STEP_RATIO: f32 = 0.20;

impl RuntimeApp {
    pub(super) fn is_quit_key(&self, key: crossterm::event::KeyEvent) -> bool {
        self.actions
            .iter()
            .any(|(binding, action)| *action == Action::Quit && binding.matches(key))
    }

    pub(super) fn handle_action(&mut self, action: Action) -> bool {
        match action {
            Action::SplitVertical => {
                self.split_active(Orientation::Vertical);
            }
            Action::SplitHorizontal => {
                self.split_active(Orientation::Horizontal);
            }
            Action::ResizeLeft => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Vertical,
                    -RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::ResizeRight => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Vertical,
                    RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::ResizeUp => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Horizontal,
                    -RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::ResizeDown => {
                let _ = resize_from_bite(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Horizontal,
                    RESIZE_STEP_RATIO,
                );
                self.resize_all(terminal_size());
            }
            Action::SwapVertical => {
                let _ = swap_adjacent_bites(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Vertical,
                );
            }
            Action::SwapHorizontal => {
                let _ = swap_adjacent_bites(
                    &mut self.template.layout,
                    self.active_id,
                    Orientation::Horizontal,
                );
            }
            Action::SaveState => {
                self.prompt = Some(InputPrompt::save_mode());
            }
            Action::RestoreState => {
                self.prompt = Some(InputPrompt::restore_mode());
            }
            Action::FocusNext => {
                self.focus_next();
            }
            Action::Quit => return true,
        }
        false
    }

    fn split_active(&mut self, orientation: Orientation) {
        let new_id = next_id(&self.template.layout);
        let did = split_bite(
            &mut self.template.layout,
            self.active_id,
            orientation,
            0.5,
            new_id,
            &self.config.default_command,
        );
        if did {
            let full = terminal_size();
            let size = main_area(full);
            let mut rects = Vec::new();
            layout_rects(&self.template.layout, size, &mut rects);
            if let Some((_, rect)) = rects.iter().find(|(id, _)| *id == new_id) {
                if let Some(Node::Bite { command, .. }) =
                    crate::layout::find_bite(&self.template.layout, new_id)
                {
                    let pane = PaneProcess::spawn(
                        command.clone(),
                        PtySize {
                            rows: rect.height.saturating_sub(2),
                            cols: rect.width.saturating_sub(2),
                            pixel_width: 0,
                            pixel_height: 0,
                        },
                    );
                    match pane {
                        Ok(pane) => {
                            self.panes.insert(new_id, pane);
                        }
                        Err(err) => {
                            let _ = rollback_split_bite(&mut self.template.layout, new_id);
                            self.status = format!("分割に失敗: {err}");
                            return;
                        }
                    }
                }
            }
            self.resize_all(full);
        }
    }

    fn focus_next(&mut self) {
        let mut ids = Vec::new();
        collect_bites(&self.template.layout, &mut ids);
        if ids.is_empty() {
            return;
        }
        if let Some(pos) = ids.iter().position(|id| *id == self.active_id) {
            let next = (pos + 1) % ids.len();
            self.active_id = ids[next];
        } else {
            self.active_id = ids[0];
        }
    }

    pub(super) fn resize_all(&mut self, area: ratatui::layout::Rect) {
        let area = main_area(area);
        let mut rects = Vec::new();
        layout_rects(&self.template.layout, area, &mut rects);
        for (id, rect) in rects {
            if let Some(pane) = self.panes.get_mut(&id) {
                pane.resize(rect.height.saturating_sub(2), rect.width.saturating_sub(2));
            }
        }
    }
}

fn rollback_split_bite(node: &mut Node, new_bite_id: u64) -> bool {
    let is_target = matches!(
        node,
        Node::Spoon { id, first, second, .. }
            if *id == new_bite_id + 1
                && matches!(first.as_ref(), Node::Bite { .. })
                && matches!(second.as_ref(), Node::Bite { id, .. } if *id == new_bite_id)
    );
    if is_target {
        let replaced = std::mem::replace(
            node,
            Node::Bite {
                id: 0,
                name: String::new(),
                command: String::new(),
            },
        );
        if let Node::Spoon { first, .. } = replaced {
            *node = *first;
            return true;
        }
    }

    match node {
        Node::Spoon { first, second, .. } => {
            rollback_split_bite(first, new_bite_id) || rollback_split_bite(second, new_bite_id)
        }
        Node::Bite { .. } => false,
    }
}
