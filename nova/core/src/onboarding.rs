use anyhow::Result;

use crate::config::{self, Config};
use crate::speech;
use crate::stt::SttEngine;

fn speak(text: &str) {
    speech::speak(text);
}

fn normalize_yes_no(input: &str) -> Option<bool> {
    let cleaned: String = input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphabetic() || c.is_ascii_whitespace() { c } else { ' ' })
        .collect();
    let words: Vec<&str> = cleaned.split_whitespace().collect();
    let has_yes = words
        .iter()
        .any(|w| matches!(*w, "yes" | "yeah" | "yep" | "correct"));
    let has_no = words
        .iter()
        .any(|w| matches!(*w, "no" | "nope" | "incorrect"));

    if has_yes && !has_no {
        return Some(true);
    }
    if has_no && !has_yes {
        return Some(false);
    }
    None
}

fn parse_spelling_token(token: &str) -> Option<String> {
    let t = token.trim().to_lowercase();
    if t == "space" {
        return Some(" ".to_string());
    }
    if t == "dash" {
        return Some("-".to_string());
    }
    if t == "apostrophe" {
        return Some("'".to_string());
    }
    if t.len() == 1 && t.chars().all(|c| c.is_ascii_alphabetic()) {
        return Some(t.to_uppercase());
    }
    None
}

fn run_spell_mode(stt: &dyn SttEngine, hotkey: &str) -> Result<String> {
    speak("Please spell your name. Say letters one by one. Say done when finished.");
    let mut out = String::new();
    loop {
        let heard = stt.capture_with_hotkey(hotkey)?;
        let token = heard.trim().to_lowercase();
        if token == "done" {
            break;
        }
        if token == "backspace" {
            out.pop();
            continue;
        }
        if let Some(c) = parse_spelling_token(&token) {
            out.push_str(&c);
        }
    }
    Ok(out.trim().to_string())
}

pub fn ensure_onboarded(root: &std::path::Path, cfg: &mut Config, stt: &dyn SttEngine) -> Result<()> {
    if cfg.user_name.is_some() {
        return Ok(());
    }

    speak("Hello. What is your name?");
    let mut final_name = stt.capture_with_hotkey(&cfg.hotkey)?;

    loop {
        speak(&format!("I heard: {}. Is that correct? Say yes or no.", final_name));
        let confirm = stt.capture_with_hotkey(&cfg.hotkey)?;
        match normalize_yes_no(&confirm) {
            Some(true) => break,
            Some(false) => {
                final_name = run_spell_mode(stt, &cfg.hotkey)?;
                break;
            }
            None => {
                speak("Please say yes or no.");
            }
        }
    }

    cfg.user_name = Some(final_name);
    config::save(root, cfg)?;

    speak("Nova runs in the background.");
    speak("Nova is accessible by hotkey and system tray.");
    speak(&format!(
        "To give a command, press and hold {}, speak, then release.",
        cfg.hotkey
    ));
    speak("Good. Nova is ready.");
    Ok(())
}
