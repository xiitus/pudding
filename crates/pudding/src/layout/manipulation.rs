use crate::layout::{clamp_ratio, walk};
use crate::model::{Node, Orientation};

pub fn next_id(node: &Node) -> u64 {
    let mut max_id = 0;
    walk(node, &mut |n| {
        if n.id() > max_id {
            max_id = n.id();
        }
    });
    max_id + 1
}

pub fn split_bite(
    node: &mut Node,
    target_id: u64,
    orientation: Orientation,
    ratio: f32,
    new_id: u64,
    default_command: &str,
) -> bool {
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
            split_bite(
                first,
                target_id,
                orientation,
                ratio,
                new_id,
                default_command,
            ) || split_bite(
                second,
                target_id,
                orientation,
                ratio,
                new_id,
                default_command,
            )
        }
    }
}

pub fn resize_from_bite(
    node: &mut Node,
    target_id: u64,
    orientation: Orientation,
    delta: f32,
) -> bool {
    resize_from_bite_inner(node, target_id, orientation, delta)
}

pub fn swap_adjacent_bites(node: &mut Node, target_id: u64, orientation: Orientation) -> bool {
    match node {
        Node::Spoon {
            orientation: o,
            first,
            second,
            ..
        } if *o == orientation => {
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

fn resize_from_bite_inner(
    node: &mut Node,
    target_id: u64,
    orientation: Orientation,
    delta: f32,
) -> bool {
    match node {
        Node::Spoon {
            orientation: o,
            ratio,
            first,
            second,
            ..
        } if *o == orientation => {
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

fn contains_bite(node: &Node, target_id: u64) -> bool {
    match node {
        Node::Bite { id, .. } => *id == target_id,
        Node::Spoon { first, second, .. } => {
            contains_bite(first, target_id) || contains_bite(second, target_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::next_id;
    use crate::model::{Node, Orientation};

    #[test]
    fn next_id_uses_max_even_when_ids_are_duplicated() {
        let node = Node::Spoon {
            id: 10,
            orientation: Orientation::Vertical,
            ratio: 0.5,
            first: Box::new(Node::Bite {
                id: 3,
                name: "a".to_string(),
                command: "sh".to_string(),
            }),
            second: Box::new(Node::Spoon {
                id: 3,
                orientation: Orientation::Horizontal,
                ratio: 0.5,
                first: Box::new(Node::Bite {
                    id: 9,
                    name: "b".to_string(),
                    command: "sh".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 9,
                    name: "c".to_string(),
                    command: "sh".to_string(),
                }),
            }),
        };
        assert_eq!(next_id(&node), 11);
    }
}
