use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::actions::{Action, ACTION_VERSION};

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub last_action: Option<Action>,
    pub last_entities: Vec<(String, String)>,
    pub pending_slot: Option<String>,
    pub recent_transcripts: VecDeque<String>,
    pub recent_actions: VecDeque<String>,
    pub last_updated: Option<Instant>,
    ttl: Duration,
}

impl SessionContext {
    pub fn new(ttl: Duration) -> Self {
        Self {
            last_action: None,
            last_entities: Vec::new(),
            pending_slot: None,
            recent_transcripts: VecDeque::new(),
            recent_actions: VecDeque::new(),
            last_updated: None,
            ttl,
        }
    }

    pub fn is_stale(&self) -> bool {
        match self.last_updated {
            Some(t) => t.elapsed() > self.ttl,
            None => false,
        }
    }

    pub fn add_transcript(&mut self, transcript: &str) {
        self.recent_transcripts.push_back(transcript.to_string());
        while self.recent_transcripts.len() > 10 {
            let _ = self.recent_transcripts.pop_front();
        }
        self.last_updated = Some(Instant::now());
    }

    pub fn remember_action(&mut self, action: &Action) {
        self.last_action = Some(action.clone());
        self.recent_actions.push_back(action.action_type.clone());
        while self.recent_actions.len() > 10 {
            let _ = self.recent_actions.pop_front();
        }
        self.last_entities = extract_entities(action);
        self.last_updated = Some(Instant::now());
    }

    pub fn summary(&self) -> String {
        let action = self
            .last_action
            .as_ref()
            .map(|a| a.action_type.clone())
            .unwrap_or_else(|| "none".to_string());
        let entities = if self.last_entities.is_empty() {
            "none".to_string()
        } else {
            self.last_entities
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "last_action={action}; pending_slot={}; entities=[{entities}]",
            self.pending_slot.clone().unwrap_or_else(|| "none".to_string())
        )
    }

    pub fn context_for_parser(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        if let Some(a) = &self.last_action {
            obj.insert(
                "last_action_type".to_string(),
                serde_json::Value::String(a.action_type.clone()),
            );
        }
        if !self.last_entities.is_empty() {
            let mut ents = serde_json::Map::new();
            for (k, v) in &self.last_entities {
                ents.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            obj.insert("last_entities".to_string(), serde_json::Value::Object(ents));
        }
        serde_json::Value::Object(obj)
    }
}

fn extract_entities(action: &Action) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(obj) = action.params.as_object() {
        for key in ["name", "query", "text", "platform", "action", "mode", "destination"] {
            if let Some(val) = obj.get(key).and_then(|v| v.as_str()) {
                out.push((key.to_string(), val.to_string()));
            }
        }
    }
    out
}

fn clarify(question: &str) -> Action {
    Action {
        version: ACTION_VERSION.to_string(),
        action_type: "clarify".to_string(),
        params: serde_json::json!({ "question": question }),
    }
}

fn replace_field(action: &Action, field: &str, value: &str) -> Option<Action> {
    let mut cloned = action.clone();
    let params = cloned.params.as_object_mut()?;
    params.insert(
        field.to_string(),
        serde_json::Value::String(value.trim().to_string()),
    );
    Some(cloned)
}

pub fn resolve_session_action(session: &mut SessionContext, action: Action) -> Action {
    if session.is_stale() {
        session.last_action = None;
        session.last_entities.clear();
    }

    match action.action_type.as_str() {
        "reference" => {
            let target = action.params.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let last = match &session.last_action {
                Some(a) => a.clone(),
                None => return clarify("I need previous context. Please repeat."),
            };

            if target == "search_web" && last.action_type == "search_web" {
                let platform = action.params.get("platform").and_then(|v| v.as_str());
                let mut out = last.clone();
                if let Some(p) = platform {
                    if let Some(params) = out.params.as_object_mut() {
                        params.insert("platform".to_string(), serde_json::Value::String(p.to_string()));
                    }
                }
                return out;
            }
            if target == "open_app" && matches!(last.action_type.as_str(), "open_app" | "focus_app") {
                return last;
            }
            if target == "type_text" && last.action_type == "type_text" {
                return last;
            }
            clarify("I could not resolve what \"this\" refers to.")
        }
        "correction" => {
            let value = action.params.get("value").and_then(|v| v.as_str()).unwrap_or("");
            if value.is_empty() {
                return clarify("Please say the corrected value.");
            }
            let last = match &session.last_action {
                Some(a) => a.clone(),
                None => return clarify("There is nothing to correct yet."),
            };
            match last.action_type.as_str() {
                "open_app" | "focus_app" => replace_field(&last, "name", value)
                    .unwrap_or_else(|| clarify("Correction failed. Please repeat.")),
                "search_web" => replace_field(&last, "query", value)
                    .unwrap_or_else(|| clarify("Correction failed. Please repeat.")),
                "type_text" => replace_field(&last, "text", value)
                    .unwrap_or_else(|| clarify("Correction failed. Please repeat.")),
                _ => clarify("I cannot apply correction to the last command."),
            }
        }
        "spelling_update" => {
            let value = action.params.get("value").and_then(|v| v.as_str()).unwrap_or("");
            if value.is_empty() {
                return clarify("Please spell it again.");
            }
            let last = match &session.last_action {
                Some(a) => a.clone(),
                None => return clarify("There is nothing to update yet."),
            };
            match last.action_type.as_str() {
                "open_app" | "focus_app" => replace_field(&last, "name", value)
                    .unwrap_or_else(|| clarify("Spelling update failed. Please repeat.")),
                "search_web" => replace_field(&last, "query", value)
                    .unwrap_or_else(|| clarify("Spelling update failed. Please repeat.")),
                "type_text" => replace_field(&last, "text", value)
                    .unwrap_or_else(|| clarify("Spelling update failed. Please repeat.")),
                _ => clarify("I cannot apply spelling update to the last command."),
            }
        }
        "confirm" => {
            if let Some(last) = &session.last_action {
                last.clone()
            } else {
                clarify("There is nothing to confirm.")
            }
        }
        _ => action,
    }
}
