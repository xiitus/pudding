use ratatui::layout::Rect;

use crate::model::{Node, Orientation};

const MIN_RATIO: f32 = 0.1;
const MAX_RATIO: f32 = 0.9;

pub fn next_id(node: &Node) -> u64 {
    let mut max_id = 0;
    walk(node, &mut |n| {
        if n.id() > max_id {
            max_id = n.id();
        }
    });
    max_id + 1
}

pub fn walk<F: FnMut(&Node)>(node: &Node, f: &mut F) {
    f(node);
    if let Node::Spoon { first, second, .. } = node {
        walk(first, f);
        walk(second, f);
    }
}

pub fn walk_mut<F: FnMut(&mut Node)>(node: &mut Node, f: &mut F) {
    f(node);
    if let Node::Spoon { first, second, .. } = node {
        walk_mut(first, f);
        walk_mut(second, f);
    }
}

pub fn collect_bites(node: &Node, out: &mut Vec<u64>) {
    match node {
        Node::Bite { id, .. } => out.push(*id),
        Node::Spoon { first, second, .. } => {
            collect_bites(first, out);
            collect_bites(second, out);
        }
    }
}

pub fn find_bite(node: &Node, target_id: u64) -> Option<&Node> {
    match node {
        Node::Bite { id, .. } if *id == target_id => Some(node),
        Node::Bite { .. } => None,
        Node::Spoon { first, second, .. } => {
            find_bite(first, target_id).or_else(|| find_bite(second, target_id))
        }
    }
}

pub fn find_bite_mut(node: &mut Node, target_id: u64) -> Option<&mut Node> {
    match node {
        Node::Bite { id, .. } if *id == target_id => Some(node),
        Node::Bite { .. } => None,
        Node::Spoon { first, second, .. } => {
            if let Some(found) = find_bite_mut(first, target_id) {
                Some(found)
            } else {
                find_bite_mut(second, target_id)
            }
        }
    }
}

pub fn split_bite(node: &mut Node, target_id: u64, orientation: Orientation, ratio: f32, new_id: u64, default_command: &str) -> bool {
    match node {
        Node::Bite { id, name, command } if *id == target_id => {
            let original = Node::Bite {
                id: *id,
                name: name.clone(),
                command: command.clone(),
            };
            let new_bite = Node::Bite {
                id: new_id,
                name: format!("bite-{}", new_id),
                command: default_command.to_string(),
            };
            *node = Node::Spoon {
                id: new_id + 1,
                orientation,
                ratio: clamp_ratio(ratio),
                first: Box::new(original),
                second: Box::new(new_bite),
            };
            true
        }
        Node::Bite { .. } => false,
        Node::Spoon { first, second, .. } => {
            split_bite(first, target_id, orientation, ratio, new_id, default_command)
                || split_bite(second, target_id, orientation, ratio, new_id, default_command)
        }
    }
}

pub fn resize_from_bite(node: &mut Node, target_id: u64, orientation: Orientation, delta: f32) -> bool {
    resize_from_bite_inner(node, target_id, orientation, delta)
}

fn resize_from_bite_inner(node: &mut Node, target_id: u64, orientation: Orientation, delta: f32) -> bool {
    match node {
        Node::Spoon { orientation: o, ratio, first, second, .. } if *o == orientation => {
            let in_first = contains_bite(first, target_id);
            let in_second = contains_bite(second, target_id);
            if in_first || in_second {
                *ratio = clamp_ratio(*ratio + delta);
                return true;
            }
            resize_from_bite_inner(first, target_id, orientation, delta)
                || resize_from_bite_inner(second, target_id, orientation, delta)
        }
        Node::Spoon { first, second, .. } => {
            resize_from_bite_inner(first, target_id, orientation, delta)
                || resize_from_bite_inner(second, target_id, orientation, delta)
        }
        Node::Bite { .. } => false,
    }
}

pub fn swap_adjacent_bites(node: &mut Node, target_id: u64, orientation: Orientation) -> bool {
    match node {
        Node::Spoon { orientation: o, first, second, .. } if *o == orientation => {
            let can_swap = matches!(first.as_ref(), Node::Bite { .. })
                && matches!(second.as_ref(), Node::Bite { .. });
            if can_swap {
                let first_id = first.id();
                let second_id = second.id();
                if first_id == target_id || second_id == target_id {
                    std::mem::swap(first, second);
                    return true;
                }
            }
            swap_adjacent_bites(first, target_id, orientation)
                || swap_adjacent_bites(second, target_id, orientation)
        }
        Node::Spoon { first, second, .. } => {
            swap_adjacent_bites(first, target_id, orientation)
                || swap_adjacent_bites(second, target_id, orientation)
        }
        Node::Bite { .. } => false,
    }
}

pub fn layout_rects(node: &Node, rect: Rect, out: &mut Vec<(u64, Rect)>) {
    match node {
        Node::Bite { id, .. } => out.push((*id, rect)),
        Node::Spoon { orientation, ratio, first, second, .. } => {
            let (r1, r2) = split_rect(rect, *orientation, *ratio);
            layout_rects(first, r1, out);
            layout_rects(second, r2, out);
        }
    }
}

pub fn find_bite_at(node: &Node, rect: Rect, x: u16, y: u16) -> Option<u64> {
    match node {
        Node::Bite { id, .. } => Some(*id),
        Node::Spoon { orientation, ratio, first, second, .. } => {
            let (r1, r2) = split_rect(rect, *orientation, *ratio);
            if point_in_rect(r1, x, y) {
                find_bite_at(first, r1, x, y)
            } else if point_in_rect(r2, x, y) {
                find_bite_at(second, r2, x, y)
            } else {
                None
            }
        }
    }
}

pub fn clamp_ratio(ratio: f32) -> f32 {
    if ratio < MIN_RATIO {
        MIN_RATIO
    } else if ratio > MAX_RATIO {
        MAX_RATIO
    } else {
        ratio
    }
}

pub fn split_rect(rect: Rect, orientation: Orientation, ratio: f32) -> (Rect, Rect) {
    let ratio = clamp_ratio(ratio);
    match orientation {
        Orientation::Vertical => {
            let mut w1 = (rect.width as f32 * ratio).round() as u16;
            if w1 < 1 {
                w1 = 1;
            }
            if w1 >= rect.width {
                w1 = rect.width - 1;
            }
            let r1 = Rect { x: rect.x, y: rect.y, width: w1, height: rect.height };
            let r2 = Rect { x: rect.x + w1, y: rect.y, width: rect.width - w1, height: rect.height };
            (r1, r2)
        }
        Orientation::Horizontal => {
            let mut h1 = (rect.height as f32 * ratio).round() as u16;
            if h1 < 1 {
                h1 = 1;
            }
            if h1 >= rect.height {
                h1 = rect.height - 1;
            }
            let r1 = Rect { x: rect.x, y: rect.y, width: rect.width, height: h1 };
            let r2 = Rect { x: rect.x, y: rect.y + h1, width: rect.width, height: rect.height - h1 };
            (r1, r2)
        }
    }
}

fn contains_bite(node: &Node, target_id: u64) -> bool {
    match node {
        Node::Bite { id, .. } => *id == target_id,
        Node::Spoon { first, second, .. } => contains_bite(first, target_id) || contains_bite(second, target_id),
    }
}

fn point_in_rect(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
