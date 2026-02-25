use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub user_name: Option<String>,
    pub hotkey: String,
    pub python_cmd: String,
    pub feedback_silent: bool,
    pub screenshots_dir: String,
    pub app_index_file: String,
    #[serde(default = "default_parser_mode")]
    pub parser_mode: String,
    #[serde(default = "default_llm_backend")]
    pub llm_backend: String,
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
    #[serde(default = "default_llm_timeout_ms")]
    pub llm_timeout_ms: u64,
}

fn default_parser_mode() -> String {
    "hybrid".to_string()
}

fn default_llm_backend() -> String {
    "ollama".to_string()
}

fn default_llm_model() -> String {
    "llama3.1:8b".to_string()
}

fn default_llm_timeout_ms() -> u64 {
    12000
}

pub fn project_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to get current dir")?;

    let mut cur = Some(cwd.as_path());
    while let Some(p) = cur {
        let has_cfg = p.join("config").join("default_config.json").exists();
        let has_plugins = p.join("plugins").join("runner.py").exists();
        if has_cfg && has_plugins {
            return Ok(p.to_path_buf());
        }
        cur = p.parent();
    }

    if cwd.ends_with("core") || cwd.ends_with("ui") {
        return Ok(cwd.parent().unwrap_or(&cwd).to_path_buf());
    }
    Ok(cwd)
}

pub fn config_dir(root: &Path) -> PathBuf {
    root.join("config")
}

pub fn config_file(root: &Path) -> PathBuf {
    config_dir(root).join("config.json")
}

pub fn default_config_file(root: &Path) -> PathBuf {
    config_dir(root).join("default_config.json")
}

pub fn load_or_default(root: &Path) -> Result<Config> {
    let cfg_path = config_file(root);
    if cfg_path.exists() {
        let raw = fs::read_to_string(&cfg_path)
            .with_context(|| format!("Failed to read {}", cfg_path.display()))?;
        let cfg: Config = serde_json::from_str(&raw).context("Invalid config.json")?;
        return Ok(cfg);
    }

    let default_path = default_config_file(root);
    let raw = fs::read_to_string(&default_path)
        .with_context(|| format!("Failed to read {}", default_path.display()))?;
    let cfg: Config = serde_json::from_str(&raw).context("Invalid default_config.json")?;
    Ok(cfg)
}

pub fn save(root: &Path, cfg: &Config) -> Result<()> {
    fs::create_dir_all(config_dir(root)).context("Failed to create config dir")?;
    let cfg_path = config_file(root);
    let raw = serde_json::to_string_pretty(cfg).context("Failed to serialize config")?;
    fs::write(&cfg_path, raw).with_context(|| format!("Failed to write {}", cfg_path.display()))?;
    Ok(())
}
