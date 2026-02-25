use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use anyhow::{anyhow, bail, Context, Result};
use kdl::{KdlDocument, KdlNode, KdlValue};

use crate::layout::{MAX_RATIO, MIN_RATIO};
use crate::model::{default_layout, Direction, Layout, Node, Tab};
use crate::paths::templates_dir;

pub fn to_kdl_document(layout: &Layout) -> KdlDocument {
    let mut doc = KdlDocument::new();
    let mut layout_node = KdlNode::new("layout");
    layout_node.insert("name", layout.name.as_str());

    let mut layout_children = KdlDocument::new();
    for (index, tab) in layout.tabs.iter().enumerate() {
        layout_children
            .nodes_mut()
            .push(tab_to_kdl(tab, index == layout.active_tab));
    }

    layout_node.set_children(layout_children);
    doc.nodes_mut().push(layout_node);
    doc
}

pub fn from_kdl_document(doc: &KdlDocument) -> Result<Layout> {
    let layout_node = doc
        .nodes()
        .iter()
        .find(|node| node.name().value() == "layout")
        .ok_or_else(|| anyhow!("missing `layout` root node"))?;

    let Some(children) = layout_node.children() else {
        bail!("`layout` node must have children");
    };

    let tab_nodes: Vec<&KdlNode> = children
        .nodes()
        .iter()
        .filter(|node| node.name().value() == "tab")
        .collect();

    if tab_nodes.is_empty() {
        bail!("`layout` must contain at least one `tab`");
    }

    let mut next_id = 1_u64;
    let mut tabs = Vec::with_capacity(tab_nodes.len());
    let mut active_tab = 0_usize;

    for (index, tab_node) in tab_nodes.iter().enumerate() {
        let tab = tab_from_kdl(tab_node, &mut next_id, index)?;
        if node_bool(tab_node, "active").unwrap_or(false) {
            active_tab = index;
        }
        tabs.push(tab);
    }

    let mut layout = Layout {
        name: node_string(layout_node, "name").unwrap_or_else(|| "default".to_string()),
        tabs,
        active_tab,
    };

    if layout.active_tab >= layout.tabs.len() {
        layout.active_tab = 0;
    }

    validate_layout(&layout)?;
    Ok(layout)
}

pub fn validate_name(name: &str) -> Result<()> {
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

pub fn load(name: &str) -> Result<Layout> {
    validate_name(name)?;

    let path = template_path(name);
    if !path.exists() {
        let mut layout = default_layout();
        layout.name = name.to_string();
        return Ok(layout);
    }

    let data = fs::read_to_string(&path)
        .with_context(|| format!("failed to read template: {}", path.display()))?;
    let doc = data
        .parse::<KdlDocument>()
        .with_context(|| format!("failed to parse KDL template: {}", path.display()))?;

    let mut layout = from_kdl_document(&doc)?;
    layout.name = name.to_string();
    validate_layout(&layout)?;
    Ok(layout)
}

pub fn save(name: &str, layout: &Layout) -> Result<PathBuf> {
    validate_name(name)?;

    let mut to_write = layout.clone();
    to_write.name = name.to_string();
    validate_layout(&to_write)?;

    let path = template_path(name);
    if let Some(parent) = path.parent() {
        ensure_dir_secure(parent)?;
    }

    let text = to_kdl_document(&to_write).to_string();
    write_private_file(&path, &text)?;

    Ok(path)
}

pub fn save_dry_run(layout: &Layout) -> String {
    to_kdl_document(layout).to_string()
}

pub fn template_path(name: &str) -> PathBuf {
    templates_dir().join(format!("{name}.kdl"))
}

fn tab_to_kdl(tab: &Tab, active: bool) -> KdlNode {
    let mut tab_node = KdlNode::new("tab");
    tab_node.insert("name", tab.name.as_str());
    if active {
        tab_node.insert("active", true);
    }

    let mut children = KdlDocument::new();
    children.nodes_mut().push(node_to_kdl(&tab.root));
    tab_node.set_children(children);

    tab_node
}

fn node_to_kdl(node: &Node) -> KdlNode {
    match node {
        Node::Pane { command, .. } => {
            let mut pane = KdlNode::new("pane");
            if let Some(cmd) = command.as_deref().filter(|value| !value.trim().is_empty()) {
                pane.insert("command", cmd);
            }
            pane
        }
        Node::Split {
            direction,
            first,
            second,
            ..
        } => {
            let mut pane = KdlNode::new("pane");
            pane.insert(
                "split_direction",
                match direction {
                    Direction::Vertical => "vertical",
                    Direction::Horizontal => "horizontal",
                },
            );

            let mut children = KdlDocument::new();
            children.nodes_mut().push(node_to_kdl(first));
            children.nodes_mut().push(node_to_kdl(second));
            pane.set_children(children);

            pane
        }
    }
}

fn tab_from_kdl(node: &KdlNode, next_id: &mut u64, index: usize) -> Result<Tab> {
    if node.name().value() != "tab" {
        bail!("expected `tab` node, found `{}`", node.name().value());
    }

    let tab_name = node_string(node, "name").unwrap_or_else(|| format!("tab-{}", index + 1));

    let root_node = node
        .children()
        .and_then(|children| {
            children
                .nodes()
                .iter()
                .find(|child| child.name().value() == "pane")
        })
        .ok_or_else(|| anyhow!("`tab` node must contain a root `pane`"))?;

    let root = node_from_kdl(root_node, next_id)?;

    Ok(Tab {
        name: tab_name,
        root,
    })
}

fn node_from_kdl(node: &KdlNode, next_id: &mut u64) -> Result<Node> {
    if node.name().value() != "pane" {
        bail!("expected `pane` node, found `{}`", node.name().value());
    }

    let id = if let Some(explicit_id) = node_u64(node, "id") {
        if explicit_id >= *next_id {
            *next_id = explicit_id + 1;
        }
        explicit_id
    } else {
        alloc_id(next_id)
    };

    let pane_children: Vec<&KdlNode> = node
        .children()
        .map(|children| {
            children
                .nodes()
                .iter()
                .filter(|child| child.name().value() == "pane")
                .collect()
        })
        .unwrap_or_default();

    if !pane_children.is_empty() {
        let direction_text = node_string(node, "split_direction")
            .ok_or_else(|| anyhow!("split pane must set `split_direction`"))?;

        let direction = match direction_text.as_str() {
            "vertical" => Direction::Vertical,
            "horizontal" => Direction::Horizontal,
            _ => bail!("invalid split_direction: {}", direction_text),
        };

        if pane_children.len() != 2 {
            bail!("split pane must have exactly two child panes");
        }

        let ratio = node_f32(node, "ratio").unwrap_or(0.5);

        return Ok(Node::Split {
            id,
            direction,
            ratio,
            first: Box::new(node_from_kdl(pane_children[0], next_id)?),
            second: Box::new(node_from_kdl(pane_children[1], next_id)?),
        });
    }

    let command = node_string(node, "command").filter(|value| !value.trim().is_empty());

    Ok(Node::Pane { id, command })
}

fn node_string(node: &KdlNode, key: &str) -> Option<String> {
    node.get(key)
        .and_then(KdlValue::as_string)
        .map(ToOwned::to_owned)
}

fn node_u64(node: &KdlNode, key: &str) -> Option<u64> {
    node.get(key)
        .and_then(KdlValue::as_integer)
        .and_then(|value| u64::try_from(value).ok())
}

fn node_f32(node: &KdlNode, key: &str) -> Option<f32> {
    node.get(key).and_then(|value| match value {
        KdlValue::Integer(raw) => Some(*raw as f32),
        KdlValue::Float(raw) => Some(*raw as f32),
        _ => None,
    })
}

fn node_bool(node: &KdlNode, key: &str) -> Option<bool> {
    node.get(key).and_then(KdlValue::as_bool)
}

fn alloc_id(next_id: &mut u64) -> u64 {
    let id = *next_id;
    *next_id += 1;
    id
}

fn validate_layout(layout: &Layout) -> Result<()> {
    validate_name(&layout.name)?;

    if layout.tabs.is_empty() {
        bail!("layout must contain at least one tab");
    }

    if layout.active_tab >= layout.tabs.len() {
        bail!("active_tab out of range");
    }

    let mut ids = HashSet::new();
    for tab in &layout.tabs {
        if tab.name.trim().is_empty() {
            bail!("tab name must not be empty");
        }
        validate_node(&tab.root, &mut ids)?;
    }

    Ok(())
}

fn validate_node(node: &Node, ids: &mut HashSet<u64>) -> Result<()> {
    if !ids.insert(node.id()) {
        bail!("node id must be unique");
    }

    match node {
        Node::Pane { command, .. } => {
            if command
                .as_deref()
                .map(|value| value.trim().is_empty())
                .unwrap_or(false)
            {
                bail!("pane command must not be empty string");
            }
            Ok(())
        }
        Node::Split {
            ratio,
            first,
            second,
            ..
        } => {
            if !(*ratio >= MIN_RATIO && *ratio <= MAX_RATIO) {
                bail!(
                    "split ratio must be in [{min:.1},{max:.1}]",
                    min = MIN_RATIO,
                    max = MAX_RATIO
                );
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
    use std::{
        fs,
        path::Path,
        sync::Mutex,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use anyhow::Result;
    use kdl::KdlDocument;

    use crate::model::{Direction, Layout, Node, Tab};
    use crate::template::{
        from_kdl_document, load, save, save_dry_run, template_path, to_kdl_document, validate_name,
    };

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn round_trip_kdl_document() -> Result<()> {
        let layout = sample_layout("workspace");

        let doc = to_kdl_document(&layout);
        let text = doc.to_string();

        assert!(text.contains("layout"));
        assert!(text.contains("tab"));
        assert!(text.contains("split_direction=vertical"));
        assert!(text.contains("split_direction=horizontal"));
        assert!(text.contains("command=htop"));

        let parsed = from_kdl_document(&doc)?;
        assert_eq!(parsed.tabs, layout.tabs);
        assert_eq!(parsed.active_tab, layout.active_tab);

        Ok(())
    }

    #[test]
    fn from_kdl_rejects_missing_layout_root() {
        let doc = "tab name=\"main\" { pane }"
            .parse::<KdlDocument>()
            .expect("invalid test doc");
        assert!(from_kdl_document(&doc).is_err());
    }

    #[test]
    fn from_kdl_rejects_duplicate_ids() {
        let doc = r#"
            layout {
              tab name="main" {
                pane split_direction="vertical" id=1 {
                  pane id=2
                  pane id=2
                }
              }
            }
        "#
        .parse::<KdlDocument>()
        .expect("invalid test doc");

        assert!(from_kdl_document(&doc).is_err());
    }

    #[test]
    fn validate_name_rules() {
        assert!(validate_name("ok_name-1").is_ok());
        assert!(validate_name("A_b-C_9").is_ok());

        assert!(validate_name("").is_err());
        assert!(validate_name("../evil").is_err());
        assert!(validate_name("bad/name").is_err());
        assert!(validate_name("bad.name").is_err());
        assert!(validate_name("bad name").is_err());

        let valid_64 = "a".repeat(64);
        let invalid_65 = "a".repeat(65);
        assert!(validate_name(&valid_64).is_ok());
        assert!(validate_name(&invalid_65).is_err());
    }

    #[test]
    fn save_load_fs_round_trip() -> Result<()> {
        with_temp_config_home("fs-roundtrip", |root| {
            let layout = sample_layout("dev");

            let path = save("dev", &layout)?;
            assert_eq!(path, root.join("pudding/templates/dev.kdl"));
            assert_eq!(template_path("dev"), path);

            let loaded = load("dev")?;
            assert_eq!(loaded, layout);

            let dry_run = save_dry_run(&layout);
            assert!(dry_run.contains("layout"));
            assert!(dry_run.contains("tab"));
            assert!(dry_run.contains("pane"));

            Ok(())
        })
    }

    #[test]
    #[cfg(unix)]
    fn save_sets_unix_permissions() -> Result<()> {
        with_temp_config_home("permissions", |_| {
            let path = save("perm", &sample_layout("perm"))?;

            let file_mode = fs::metadata(&path)?.permissions().mode() & 0o777;
            assert_eq!(file_mode, 0o600);

            let dir_mode = fs::metadata(path.parent().expect("parent"))?
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(dir_mode, 0o700);

            Ok(())
        })
    }

    fn sample_layout(name: &str) -> Layout {
        Layout {
            name: name.to_string(),
            tabs: vec![
                Tab {
                    name: "main".to_string(),
                    root: Node::Split {
                        id: 1,
                        direction: Direction::Vertical,
                        ratio: 0.5,
                        first: Box::new(Node::Pane {
                            id: 2,
                            command: Some("bash".to_string()),
                        }),
                        second: Box::new(Node::Pane {
                            id: 3,
                            command: Some("htop".to_string()),
                        }),
                    },
                },
                Tab {
                    name: "logs".to_string(),
                    root: Node::Split {
                        id: 4,
                        direction: Direction::Horizontal,
                        ratio: 0.5,
                        first: Box::new(Node::Pane {
                            id: 5,
                            command: Some("tail -f /var/log/system.log".to_string()),
                        }),
                        second: Box::new(Node::Pane {
                            id: 6,
                            command: None,
                        }),
                    },
                },
            ],
            active_tab: 0,
        }
    }

    fn with_temp_config_home<F>(name: &str, f: F) -> Result<()>
    where
        F: FnOnce(&Path) -> Result<()>,
    {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();

        let root = std::env::temp_dir().join(format!(
            "pudding-template-test-{name}-{}-{nonce}",
            std::process::id()
        ));

        fs::create_dir_all(&root)?;

        let old = std::env::var_os("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", &root);

        let result = f(&root);

        if let Some(value) = old {
            std::env::set_var("XDG_CONFIG_HOME", value);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }

        let _ = fs::remove_dir_all(&root);
        result
    }
}
