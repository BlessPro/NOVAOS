use anyhow::{Context, Result};
use nova_core::config;
use nova_core::dispatcher;
use nova_core::onboarding;
use nova_core::plugin_bridge::PluginBridge;
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

    println!("Nova engine running. Press and hold {} to issue commands.", cfg.hotkey);
    loop {
        let transcript = stt
            .capture_with_hotkey(&cfg.hotkey)
            .context("Failed to capture speech")?;
        if transcript.trim().is_empty() {
            continue;
        }

        let mut action = bridge
            .parse_transcript(&transcript)
            .context("Failed to parse transcript via plugins")?;

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
            action = bridge
                .parse_transcript(&retry)
                .context("Clarify parse failed")?;
        }

        match dispatcher::dispatch(adapter.as_ref(), &action) {
            Ok(_) => println!("Action executed: {}", action.action_type),
            Err(e) => eprintln!("Action failed: {e}"),
        }
    }
}
