use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};

use crate::layout::{MAX_RATIO, MIN_RATIO};
use crate::model::{Node, Template};

pub fn validate_store_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        bail!("name must be 1..=64 chars");
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("name supports only [A-Za-z0-9_-]");
    }
    Ok(())
}

pub fn validate_template(template: &Template) -> Result<()> {
    validate_store_name(&template.name)?;
    let mut ids = HashSet::new();
    validate_node(&template.layout, &mut ids)
}

fn validate_node(node: &Node, ids: &mut HashSet<u64>) -> Result<()> {
    if !ids.insert(node.id()) {
        return Err(anyhow!("node id must be unique"));
    }
    match node {
        Node::Bite { name, command, .. } => {
            if name.trim().is_empty() {
                bail!("bite name must not be empty");
            }
            if command.trim().is_empty() {
                bail!("bite command must not be empty");
            }
            Ok(())
        }
        Node::Spoon {
            ratio,
            first,
            second,
            ..
        } => {
            if !(*ratio >= MIN_RATIO && *ratio <= MAX_RATIO) {
                bail!(
                    "spoon ratio must be in [{min:.1},{max:.1}]",
                    min = MIN_RATIO,
                    max = MAX_RATIO
                );
            }
            validate_node(first, ids)?;
            validate_node(second, ids)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layout::{MAX_RATIO, MIN_RATIO};
    use crate::model::{Node, Orientation, Template};
    use crate::template::validation::{validate_store_name, validate_template};

    #[test]
    fn reject_path_like_name() {
        assert!(validate_store_name("../evil").is_err());
        assert!(validate_store_name("ok_name-1").is_ok());
    }

    #[test]
    fn reject_invalid_template_ratio() {
        let template = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: 1.0,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&template).is_err());
    }

    #[test]
    fn reject_duplicate_node_id() {
        let template = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: 0.5,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 2,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&template).is_err());
    }

    #[test]
    fn reject_empty_bite_name_and_command() {
        let with_empty_name = Template {
            name: "ok".to_string(),
            layout: Node::Bite {
                id: 1,
                name: "   ".to_string(),
                command: "bash".to_string(),
            },
        };
        assert!(validate_template(&with_empty_name).is_err());

        let with_empty_command = Template {
            name: "ok".to_string(),
            layout: Node::Bite {
                id: 1,
                name: "valid".to_string(),
                command: "   ".to_string(),
            },
        };
        assert!(validate_template(&with_empty_command).is_err());
    }

    #[test]
    fn reject_invalid_store_name() {
        assert!(validate_store_name("bad/name").is_err());
        assert!(validate_store_name("bad.name").is_err());
        assert!(validate_store_name("bad name").is_err());
        assert!(validate_store_name("").is_err());
    }

    #[test]
    fn validate_store_name_length_boundaries() {
        let valid_64 = "a".repeat(64);
        let invalid_65 = "a".repeat(65);
        assert!(validate_store_name(&valid_64).is_ok());
        assert!(validate_store_name(&invalid_65).is_err());
    }

    #[test]
    fn reject_template_ratio_outside_clamp_interval() {
        let below_zero = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: MIN_RATIO - 0.01,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&below_zero).is_err());

        let above_one = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: MAX_RATIO + 0.01,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&above_one).is_err());
    }

    #[test]
    fn reject_duplicate_node_id_across_levels() {
        let template = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: 0.5,
                first: Box::new(Node::Bite {
                    id: 1,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&template).is_err());
    }

    #[test]
    fn validate_template_ratio_boundaries() {
        let below_min = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: MIN_RATIO - 0.001,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&below_min).is_err());

        let at_min = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: MIN_RATIO,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&at_min).is_ok());

        let above_max = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: MAX_RATIO + 0.001,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&above_max).is_err());

        let at_max = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: MAX_RATIO,
                first: Box::new(Node::Bite {
                    id: 2,
                    name: "a".to_string(),
                    command: "bash".to_string(),
                }),
                second: Box::new(Node::Bite {
                    id: 3,
                    name: "b".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&at_max).is_ok());
    }

    #[test]
    fn accept_nested_valid_template() {
        let template = Template {
            name: "valid_name_01".to_string(),
            layout: Node::Spoon {
                id: 10,
                orientation: Orientation::Horizontal,
                ratio: 0.4,
                first: Box::new(Node::Spoon {
                    id: 11,
                    orientation: Orientation::Vertical,
                    ratio: 0.6,
                    first: Box::new(Node::Bite {
                        id: 12,
                        name: "left-top".to_string(),
                        command: "bash".to_string(),
                    }),
                    second: Box::new(Node::Bite {
                        id: 13,
                        name: "left-bottom".to_string(),
                        command: "bash".to_string(),
                    }),
                }),
                second: Box::new(Node::Bite {
                    id: 14,
                    name: "right".to_string(),
                    command: "bash".to_string(),
                }),
            },
        };
        assert!(validate_template(&template).is_ok());
    }
}
