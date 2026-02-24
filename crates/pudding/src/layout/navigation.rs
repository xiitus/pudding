use ratatui::layout::Rect;

use crate::model::Node;

use super::geometry::split_rect;

pub fn walk<F: FnMut(&Node)>(node: &Node, f: &mut F) {
    f(node);
    if let Node::Spoon { first, second, .. } = node {
        walk(first, f);
        walk(second, f);
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

pub fn layout_rects(node: &Node, rect: Rect, out: &mut Vec<(u64, Rect)>) {
    match node {
        Node::Bite { id, .. } => out.push((*id, rect)),
        Node::Spoon {
            orientation,
            ratio,
            first,
            second,
            ..
        } => {
            let (r1, r2) = split_rect(rect, *orientation, *ratio);
            layout_rects(first, r1, out);
            layout_rects(second, r2, out);
        }
    }
}
