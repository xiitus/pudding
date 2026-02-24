use ratatui::layout::Rect;

use crate::model::{Direction, Node};

pub const MIN_RATIO: f32 = 0.1;
pub const MAX_RATIO: f32 = 0.9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteNodeError {
    LastPane,
    NotFound,
}

pub fn next_id(root: &Node) -> u64 {
    let mut max_id = 0;
    walk(root, &mut |node| {
        if node.id() > max_id {
            max_id = node.id();
        }
    });
    max_id.saturating_add(1)
}

pub fn collect_panes(node: &Node, out: &mut Vec<u64>) {
    match node {
        Node::Pane { id, .. } => out.push(*id),
        Node::Split { first, second, .. } => {
            collect_panes(first, out);
            collect_panes(second, out);
        }
    }
}

pub fn find_node(node: &Node, target_id: u64) -> Option<&Node> {
    match node {
        Node::Pane { id, .. } if *id == target_id => Some(node),
        Node::Pane { .. } => None,
        Node::Split {
            id, first, second, ..
        } => {
            if *id == target_id {
                Some(node)
            } else {
                find_node(first, target_id).or_else(|| find_node(second, target_id))
            }
        }
    }
}

pub fn find_node_mut(node: &mut Node, target_id: u64) -> Option<&mut Node> {
    if node.id() == target_id {
        return Some(node);
    }

    match node {
        Node::Pane { .. } => None,
        Node::Split { first, second, .. } => {
            if let Some(found) = find_node_mut(first, target_id) {
                Some(found)
            } else {
                find_node_mut(second, target_id)
            }
        }
    }
}

pub fn split_node(
    root: &mut Node,
    target_id: u64,
    direction: Direction,
    ratio: f32,
    start_id: u64,
) -> Option<u64> {
    find_node(root, target_id)?;

    let split_id = start_id;
    let new_pane_id = split_id.checked_add(1)?;
    let target = find_node_mut(root, target_id)?;

    let original = std::mem::replace(
        target,
        Node::Pane {
            id: new_pane_id,
            command: None,
        },
    );
    *target = Node::Split {
        id: split_id,
        direction,
        ratio: clamp_ratio(ratio),
        first: Box::new(original),
        second: Box::new(Node::Pane {
            id: new_pane_id,
            command: None,
        }),
    };

    Some(new_pane_id)
}

pub fn delete_node(root: &mut Node, target_id: u64) -> Result<(), DeleteNodeError> {
    if pane_count(root) <= 1 {
        return Err(DeleteNodeError::LastPane);
    }
    if !matches!(find_node(root, target_id), Some(Node::Pane { .. })) {
        return Err(DeleteNodeError::NotFound);
    }

    let current_root = std::mem::replace(
        root,
        Node::Pane {
            id: 0,
            command: None,
        },
    );
    let (next_root, deleted) = delete_node_owned(current_root, target_id);
    *root = next_root;

    if deleted {
        Ok(())
    } else {
        Err(DeleteNodeError::NotFound)
    }
}

pub fn layout_rects(node: &Node, rect: Rect, out: &mut Vec<(u64, Rect)>) {
    match node {
        Node::Pane { id, .. } => out.push((*id, rect)),
        Node::Split {
            direction,
            ratio,
            first,
            second,
            ..
        } => {
            let (first_rect, second_rect) = split_rect(rect, *direction, *ratio);
            layout_rects(first, first_rect, out);
            layout_rects(second, second_rect, out);
        }
    }
}

pub fn find_pane_at(node: &Node, rect: Rect, x: u16, y: u16) -> Option<u64> {
    if !point_in_rect(rect, x, y) {
        return None;
    }

    match node {
        Node::Pane { id, .. } => Some(*id),
        Node::Split {
            direction,
            ratio,
            first,
            second,
            ..
        } => {
            let (first_rect, second_rect) = split_rect(rect, *direction, *ratio);
            if point_in_rect(first_rect, x, y) {
                find_pane_at(first, first_rect, x, y)
            } else if point_in_rect(second_rect, x, y) {
                find_pane_at(second, second_rect, x, y)
            } else {
                None
            }
        }
    }
}

fn walk<F: FnMut(&Node)>(node: &Node, f: &mut F) {
    f(node);
    if let Node::Split { first, second, .. } = node {
        walk(first, f);
        walk(second, f);
    }
}

fn pane_count(node: &Node) -> usize {
    match node {
        Node::Pane { .. } => 1,
        Node::Split { first, second, .. } => pane_count(first) + pane_count(second),
    }
}

fn delete_node_owned(node: Node, target_id: u64) -> (Node, bool) {
    match node {
        Node::Pane { .. } => (node, false),
        Node::Split {
            id,
            direction,
            ratio,
            first,
            second,
        } => {
            let first_node = *first;
            let second_node = *second;

            if first_node.id() == target_id {
                return (second_node, true);
            }
            if second_node.id() == target_id {
                return (first_node, true);
            }

            let (new_first, deleted_in_first) = delete_node_owned(first_node, target_id);
            if deleted_in_first {
                return (
                    Node::Split {
                        id,
                        direction,
                        ratio,
                        first: Box::new(new_first),
                        second: Box::new(second_node),
                    },
                    true,
                );
            }

            let (new_second, deleted_in_second) = delete_node_owned(second_node, target_id);
            (
                Node::Split {
                    id,
                    direction,
                    ratio,
                    first: Box::new(new_first),
                    second: Box::new(new_second),
                },
                deleted_in_second,
            )
        }
    }
}

fn clamp_ratio(ratio: f32) -> f32 {
    ratio.clamp(MIN_RATIO, MAX_RATIO)
}

fn split_rect(rect: Rect, direction: Direction, ratio: f32) -> (Rect, Rect) {
    let ratio = clamp_ratio(ratio);
    match direction {
        Direction::Vertical => {
            if rect.width <= 1 {
                return (
                    rect,
                    Rect {
                        x: rect.x.saturating_add(rect.width),
                        y: rect.y,
                        width: 0,
                        height: rect.height,
                    },
                );
            }

            let mut first_width = (rect.width as f32 * ratio).round() as u16;
            first_width = first_width.clamp(1, rect.width - 1);
            let second_width = rect.width - first_width;

            (
                Rect {
                    x: rect.x,
                    y: rect.y,
                    width: first_width,
                    height: rect.height,
                },
                Rect {
                    x: rect.x + first_width,
                    y: rect.y,
                    width: second_width,
                    height: rect.height,
                },
            )
        }
        Direction::Horizontal => {
            if rect.height <= 1 {
                return (
                    rect,
                    Rect {
                        x: rect.x,
                        y: rect.y.saturating_add(rect.height),
                        width: rect.width,
                        height: 0,
                    },
                );
            }

            let mut first_height = (rect.height as f32 * ratio).round() as u16;
            first_height = first_height.clamp(1, rect.height - 1);
            let second_height = rect.height - first_height;

            (
                Rect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: first_height,
                },
                Rect {
                    x: rect.x,
                    y: rect.y + first_height,
                    width: rect.width,
                    height: second_height,
                },
            )
        }
    }
}

fn point_in_rect(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use ratatui::layout::Rect;

    use crate::layout::{
        collect_panes, delete_node, find_node, find_pane_at, layout_rects, next_id, split_node,
        DeleteNodeError,
    };
    use crate::model::{Direction, Node};

    fn single_pane(id: u64) -> Node {
        Node::Pane { id, command: None }
    }

    fn sample_tree() -> Node {
        Node::Split {
            id: 10,
            direction: Direction::Vertical,
            ratio: 0.5,
            first: Box::new(single_pane(1)),
            second: Box::new(Node::Split {
                id: 20,
                direction: Direction::Horizontal,
                ratio: 0.5,
                first: Box::new(single_pane(2)),
                second: Box::new(single_pane(3)),
            }),
        }
    }

    fn collect_all_ids(node: &Node, out: &mut Vec<u64>) {
        out.push(node.id());
        if let Node::Split { first, second, .. } = node {
            collect_all_ids(first, out);
            collect_all_ids(second, out);
        }
    }

    #[test]
    fn next_id_returns_max_id_plus_one() {
        let tree = sample_tree();
        assert_eq!(next_id(&tree), 21);
    }

    #[test]
    fn split_node_creates_split_and_keeps_ids_unique() {
        let mut tree = single_pane(1);
        let start = next_id(&tree);
        let new_pane_id = split_node(&mut tree, 1, Direction::Vertical, 0.0, start).unwrap();
        assert_eq!(new_pane_id, 3);

        match &tree {
            Node::Split {
                id,
                direction,
                ratio,
                first,
                second,
            } => {
                assert_eq!(*id, 2);
                assert_eq!(*direction, Direction::Vertical);
                assert!((*ratio - 0.1).abs() < f32::EPSILON);
                assert_eq!(first.id(), 1);
                assert_eq!(second.id(), 3);
            }
            _ => panic!("split_node should convert pane into split"),
        }

        let mut ids = Vec::new();
        collect_all_ids(&tree, &mut ids);
        let set: HashSet<u64> = ids.iter().copied().collect();
        assert_eq!(set.len(), ids.len());
    }

    #[test]
    fn delete_node_replaces_parent_with_sibling() {
        let mut tree = sample_tree();
        delete_node(&mut tree, 2).unwrap();

        assert!(find_node(&tree, 2).is_none());
        let mut pane_ids = Vec::new();
        collect_panes(&tree, &mut pane_ids);
        assert_eq!(pane_ids, vec![1, 3]);
    }

    #[test]
    fn delete_node_rejects_last_pane() {
        let mut tree = single_pane(1);
        let err = delete_node(&mut tree, 1).unwrap_err();
        assert_eq!(err, DeleteNodeError::LastPane);
    }

    #[test]
    fn delete_node_requires_existing_pane_id() {
        let mut tree = sample_tree();
        let err = delete_node(&mut tree, 999).unwrap_err();
        assert_eq!(err, DeleteNodeError::NotFound);
    }

    #[test]
    fn find_pane_at_maps_cursor_to_expected_pane() {
        let tree = sample_tree();
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        };

        assert_eq!(find_pane_at(&tree, area, 10, 10), Some(1));
        assert_eq!(find_pane_at(&tree, area, 80, 5), Some(2));
        assert_eq!(find_pane_at(&tree, area, 80, 30), Some(3));
        assert_eq!(find_pane_at(&tree, area, 120, 5), None);
    }

    #[test]
    fn layout_rects_returns_one_rect_per_pane() {
        let tree = sample_tree();
        let area = Rect {
            x: 0,
            y: 0,
            width: 90,
            height: 30,
        };
        let mut rects = Vec::new();
        layout_rects(&tree, area, &mut rects);

        let ids: Vec<u64> = rects.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }
}
