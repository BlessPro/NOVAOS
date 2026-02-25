pub mod actions;
pub mod adapter;
pub mod config;
pub mod dispatcher;
pub mod onboarding;
pub mod plugin_bridge;
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

    let configured_ok = configured.contains(std::path::MAIN_SEPARATOR) || command_exists(configured);
    if configured_ok {
        return configured.to_string();
    }

    #[cfg(target_os = "windows")]
    let venv_python = root.join(".venv").join("Scripts").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    let venv_python = root.join(".venv").join("bin").join("python");

    if venv_python.exists() {
        return venv_python.to_string_lossy().to_string();
    }

    configured.to_string()
}
