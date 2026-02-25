pub mod actions;
pub mod adapter;
pub mod config;
pub mod dispatcher;
pub mod onboarding;
pub mod plugin_bridge;
pub mod session;
pub mod speech;
pub mod stt;

use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use adapter::OsAdapter;

pub fn build_adapter(root: &std::path::Path, cfg: &config::Config) -> Result<Box<dyn OsAdapter>> {
    let screenshots_dir = if PathBuf::from(&cfg.screenshots_dir).is_absolute() {
        PathBuf::from(&cfg.screenshots_dir)
    } else {
        root.join(&cfg.screenshots_dir)
    };
    let app_index = if PathBuf::from(&cfg.app_index_file).is_absolute() {
        PathBuf::from(&cfg.app_index_file)
    } else {
        root.join(&cfg.app_index_file)
    };

    if cfg!(target_os = "windows") {
        return Ok(Box::new(adapter::windows::WindowsAdapter::new(
            app_index,
            screenshots_dir,
        )?));
    }
    if cfg!(target_os = "macos") {
        return Ok(Box::new(adapter::mac::MacAdapter::new()));
    }
    Ok(Box::new(adapter::linux::LinuxAdapter::new()))
}

pub fn resolve_python_cmd(root: &std::path::Path, configured: &str) -> String {
    let command_exists = |cmd: &str| -> bool {
        Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };

    #[cfg(target_os = "windows")]
    let venv_python = root.join(".venv").join("Scripts").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    let venv_python = root.join(".venv").join("bin").join("python");

    let configured_trimmed = configured.trim();
    let configured_lower = configured_trimmed.to_lowercase();
    let is_generic_python = matches!(configured_lower.as_str(), "python" | "python3");
    if is_generic_python && venv_python.exists() {
        return venv_python.to_string_lossy().to_string();
    }

    let configured_ok = if configured_trimmed.contains(std::path::MAIN_SEPARATOR) {
        std::path::Path::new(configured_trimmed).exists()
    } else {
        command_exists(configured_trimmed)
    };
    if configured_ok {
        return configured_trimmed.to_string();
    }

    if venv_python.exists() {
        return venv_python.to_string_lossy().to_string();
    }

    configured_trimmed.to_string()
}
