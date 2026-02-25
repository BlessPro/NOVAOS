use anyhow::{Context, Result};
use nova_core::config;
use nova_core::dispatcher;
use nova_core::onboarding;
use nova_core::plugin_bridge::PluginBridge;
use nova_core::session::{resolve_session_action, SessionContext};
use nova_core::speech;
use nova_core::stt::{PythonStt, SttEngine};
use nova_core::{build_adapter, resolve_python_cmd};

fn speak(text: &str) {
    speech::speak(text);
}

fn main() -> Result<()> {
    let root = config::project_root()?;
    let mut cfg = config::load_or_default(&root)?;
    cfg.python_cmd = resolve_python_cmd(&root, &cfg.python_cmd);

    let stt_script = root.join("plugins").join("stt_runner.py");
    let stt = PythonStt {
        python_cmd: cfg.python_cmd.clone(),
        stt_script,
    };

    onboarding::ensure_onboarded(&root, &mut cfg, &stt)?;
    config::save(&root, &cfg)?;

    let bridge = PluginBridge::new(
        cfg.python_cmd.clone(),
        root.join("plugins").join("runner.py"),
        cfg.parser_mode.clone(),
        cfg.llm_backend.clone(),
        cfg.llm_model.clone(),
        cfg.llm_timeout_ms,
    );
    let adapter = build_adapter(&root, &cfg)?;
    let mut session = SessionContext::new(std::time::Duration::from_secs(90));

    println!("Nova engine running. Press and hold {} to issue commands.", cfg.hotkey);
    loop {
        let transcript = stt
            .capture_with_hotkey(&cfg.hotkey)
            .context("Failed to capture speech")?;
        if transcript.trim().is_empty() {
            continue;
        }
        session.add_transcript(&transcript);

        let mut action = bridge
            .parse_transcript(&transcript, session.context_for_parser())
            .context("Failed to parse transcript via plugins")?;
        action = resolve_session_action(&mut session, action);

        while action.action_type == "clarify" {
            let question = action
                .params
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("Please repeat.");
            speak(question);
            let retry = stt
                .capture_with_hotkey(&cfg.hotkey)
                .context("Failed while clarify loop")?;
            session.add_transcript(&retry);
            action = bridge
                .parse_transcript(&retry, session.context_for_parser())
                .context("Clarify parse failed")?;
            action = resolve_session_action(&mut session, action);
        }

        match dispatcher::dispatch(adapter.as_ref(), &action) {
            Ok(_) => {
                println!("Action executed: {}", action.action_type);
                session.remember_action(&action);
            }
            Err(e) => eprintln!("Action failed: {e}"),
        }
    }
}
