use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};

use crate::actions::Action;

#[derive(Debug, Clone)]
pub struct PluginBridge {
    python_cmd: String,
    runner_path: PathBuf,
    parser_mode: String,
    llm_backend: String,
    llm_model: String,
    llm_timeout_ms: u64,
}

impl PluginBridge {
    pub fn new(
        python_cmd: String,
        runner_path: PathBuf,
        parser_mode: String,
        llm_backend: String,
        llm_model: String,
        llm_timeout_ms: u64,
    ) -> Self {
        Self {
            python_cmd,
            runner_path,
            parser_mode,
            llm_backend,
            llm_model,
            llm_timeout_ms,
        }
    }

    pub fn parse_transcript(&self, transcript: &str) -> Result<Action> {
        let payload = serde_json::json!({
            "transcript": transcript,
            "context": {
                "parser_mode": self.parser_mode,
                "llm_backend": self.llm_backend,
                "llm_model": self.llm_model,
                "llm_timeout_ms": self.llm_timeout_ms,
            }
        });

        let mut child = Command::new(&self.python_cmd)
            .arg(self.runner_path.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn plugin runner")?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(payload.to_string().as_bytes())?;
        }

        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(anyhow!("Plugin runner failed"));
        }

        let action: Action =
            serde_json::from_slice(&out.stdout).context("Plugin runner returned invalid JSON")?;
        action.validate()?;
        Ok(action)
    }
}
