use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    Bite {
        id: u64,
        name: String,
        command: String,
    },
    Spoon {
        id: u64,
        orientation: Orientation,
        ratio: f32,
        first: Box<Node>,
        second: Box<Node>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub layout: Node,
}

impl Node {
    pub fn id(&self) -> u64 {
        match self {
            Node::Bite { id, .. } => *id,
            Node::Spoon { id, .. } => *id,
        }
    }

    pub fn is_bite(&self) -> bool {
        matches!(self, Node::Bite { .. })
    }
}

pub fn default_template() -> Template {
    Template {
        name: "default".to_string(),
        layout: Node::Bite {
            id: 1,
            name: "main".to_string(),
            command: "bash".to_string(),
        },
    }
}
