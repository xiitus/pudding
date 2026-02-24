use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

pub fn is_in_zellij_session() -> bool {
    std::env::var("ZELLIJ_SESSION_NAME")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

pub fn apply_layout(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("layout path must be absolute: {}", path.display());
    }

    let output = Command::new("zellij")
        .arg("action")
        .arg("apply-layout")
        .arg(path)
        .output()
        .with_context(|| {
            format!(
                "failed to run zellij action apply-layout {}",
                path.display()
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = stderr.trim();
    if message.is_empty() {
        Err(anyhow!(
            "zellij action apply-layout failed with status {}",
            output.status
        ))
    } else {
        Err(anyhow!(
            "zellij action apply-layout failed with status {}: {}",
            output.status,
            message
        ))
    }
}

pub fn apply_layout_if_in_session(path: &Path, dry_run: bool) -> Option<String> {
    if dry_run || !is_in_zellij_session() {
        return None;
    }

    apply_layout(path).err().map(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{apply_layout, apply_layout_if_in_session, is_in_zellij_session};
    use std::ffi::OsString;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn restore_env(previous: Option<OsString>) {
        match previous {
            Some(value) => std::env::set_var("ZELLIJ_SESSION_NAME", value),
            None => std::env::remove_var("ZELLIJ_SESSION_NAME"),
        }
    }

    struct EnvReset(Option<OsString>);

    impl Drop for EnvReset {
        fn drop(&mut self) {
            restore_env(self.0.take());
        }
    }

    #[test]
    fn is_in_zellij_session_false_without_env() {
        let _guard = env_lock().lock().expect("lock poisoned");
        let _reset = EnvReset(std::env::var_os("ZELLIJ_SESSION_NAME"));
        std::env::remove_var("ZELLIJ_SESSION_NAME");

        assert!(!is_in_zellij_session());
    }

    #[test]
    fn is_in_zellij_session_true_with_non_empty_env() {
        let _guard = env_lock().lock().expect("lock poisoned");
        let _reset = EnvReset(std::env::var_os("ZELLIJ_SESSION_NAME"));
        std::env::set_var("ZELLIJ_SESSION_NAME", "pudding-session");

        assert!(is_in_zellij_session());
    }

    #[test]
    fn is_in_zellij_session_false_with_empty_env() {
        let _guard = env_lock().lock().expect("lock poisoned");
        let _reset = EnvReset(std::env::var_os("ZELLIJ_SESSION_NAME"));
        std::env::set_var("ZELLIJ_SESSION_NAME", "   ");

        assert!(!is_in_zellij_session());
    }

    #[test]
    fn apply_layout_rejects_relative_path() {
        let error = apply_layout(Path::new("relative-layout.kdl")).expect_err("must fail");
        let message = error.to_string();
        assert!(message.contains("layout path must be absolute"));
    }

    #[test]
    fn apply_layout_if_in_session_returns_none_on_dry_run() {
        let _guard = env_lock().lock().expect("lock poisoned");
        let _reset = EnvReset(std::env::var_os("ZELLIJ_SESSION_NAME"));
        std::env::set_var("ZELLIJ_SESSION_NAME", "active-session");

        let result = apply_layout_if_in_session(Path::new("/tmp/layout.kdl"), true);
        assert!(result.is_none());
    }

    #[test]
    fn apply_layout_if_in_session_returns_none_outside_session() {
        let _guard = env_lock().lock().expect("lock poisoned");
        let _reset = EnvReset(std::env::var_os("ZELLIJ_SESSION_NAME"));
        std::env::remove_var("ZELLIJ_SESSION_NAME");

        let result = apply_layout_if_in_session(Path::new("/tmp/layout.kdl"), false);
        assert!(result.is_none());
    }
}
