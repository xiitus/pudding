use ratatui::layout::Rect;

use super::geometry::{point_in_rect, split_rect};
use crate::model::Node;

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

pub fn find_bite_at(node: &Node, rect: Rect, x: u16, y: u16) -> Option<u64> {
    match node {
        Node::Bite { id, .. } => Some(*id),
        Node::Spoon {
            orientation,
            ratio,
            first,
            second,
            ..
        } => {
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
