use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    Pane {
        id: u64,
        command: Option<String>,
    },
    Split {
        id: u64,
        direction: Direction,
        ratio: f32,
        first: Box<Node>,
        second: Box<Node>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tab {
    pub name: String,
    pub root: Node,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layout {
    pub name: String,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
}

impl Node {
    pub fn id(&self) -> u64 {
        match self {
            Node::Pane { id, .. } => *id,
            Node::Split { id, .. } => *id,
        }
    }
}

impl Layout {
    pub fn default_with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tabs: vec![Tab {
                name: "main".to_string(),
                root: Node::Pane {
                    id: 1,
                    command: None,
                },
            }],
            active_tab: 0,
        }
    }
}

pub fn default_layout() -> Layout {
    Layout::default_with_name("default")
}

// Invariant:
// - Node IDs must be unique across all tabs in a single `Layout`.
// - `Node::Split::ratio` must stay in [0.1, 0.9].
