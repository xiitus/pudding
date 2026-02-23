use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use anyhow::{anyhow, bail, Result};

use crate::model::{default_template, Node, Template};
use crate::paths::{states_dir, templates_dir};

pub fn load_template(name: &str) -> Result<Template> {
    validate_store_name(name)?;
    let path = template_path(name);
    if path.exists() {
        let data = fs::read_to_string(&path)?;
        let tpl = serde_json::from_str::<Template>(&data)?;
        validate_template(&tpl)?;
        Ok(tpl)
    } else {
        Ok(default_template())
    }
}

pub fn save_template(template: &Template) -> Result<()> {
    validate_store_name(&template.name)?;
    validate_template(template)?;
    let path = template_path(&template.name);
    if let Some(parent) = path.parent() {
        ensure_dir_secure(parent)?;
    }
    let data = serde_json::to_string_pretty(template)?;
    write_private_file(&path, &data)?;
    Ok(())
}

pub fn template_path(name: &str) -> PathBuf {
    templates_dir().join(format!("{}.json", name))
}

pub fn load_state(name: &str) -> Result<Template> {
    validate_store_name(name)?;
    let path = state_path(name);
    let data = fs::read_to_string(&path)?;
    let tpl = serde_json::from_str::<Template>(&data)?;
    validate_template(&tpl)?;
    Ok(tpl)
}

pub fn save_state(name: &str, template: &Template) -> Result<()> {
    validate_store_name(name)?;
    validate_template(template)?;
    let path = state_path(name);
    if let Some(parent) = path.parent() {
        ensure_dir_secure(parent)?;
    }
    let data = serde_json::to_string_pretty(template)?;
    write_private_file(&path, &data)?;
    Ok(())
}

pub fn state_path(name: &str) -> PathBuf {
    states_dir().join(format!("{}.json", name))
}

fn validate_store_name(name: &str) -> Result<()> {
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

fn validate_template(template: &Template) -> Result<()> {
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
            if !(*ratio > 0.0 && *ratio < 1.0) {
                bail!("spoon ratio must be in (0,1)");
            }
            validate_node(first, ids)?;
            validate_node(second, ids)
        }
    }
}

fn ensure_dir_secure(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn write_private_file(path: &Path, content: &str) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options.open(path)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;

    file.write_all(content.as_bytes())
}

#[cfg(test)]
mod tests {
    use crate::model::{Node, Orientation, Template};
    use crate::template::{validate_store_name, validate_template};

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
    fn reject_template_ratio_outside_open_interval() {
        let below_zero = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: -0.01,
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
                ratio: 1.01,
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
        let at_zero = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: 0.0,
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
        assert!(validate_template(&at_zero).is_err());

        let near_zero = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: 0.0001,
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
        assert!(validate_template(&near_zero).is_ok());

        let at_one = Template {
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
        assert!(validate_template(&at_one).is_err());

        let near_one = Template {
            name: "ok".to_string(),
            layout: Node::Spoon {
                id: 1,
                orientation: Orientation::Vertical,
                ratio: 0.9999,
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
        assert!(validate_template(&near_one).is_ok());
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
