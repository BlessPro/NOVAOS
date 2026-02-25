use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const ACTION_VERSION: &str = "0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub version: String,
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(default)]
    pub params: Value,
}

impl Action {
    pub fn validate(&self) -> Result<()> {
        if self.version != ACTION_VERSION {
            return Err(anyhow!(
                "Unsupported action version: {}, expected {}",
                self.version,
                ACTION_VERSION
            ));
        }

        let allowed = [
            "open_app",
            "focus_app",
            "close_window",
            "type_text",
            "search_web",
            "screenshot",
            "volume",
            "clarify",
            "reference",
            "correction",
            "spelling_update",
            "confirm",
        ];

        if !allowed.contains(&self.action_type.as_str()) {
            return Err(anyhow!("Unsupported action type: {}", self.action_type));
        }

        Ok(())
    }
}
