use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};
use eframe::egui;
use nova_core::config;
use nova_core::dispatcher;
use nova_core::onboarding;
use nova_core::plugin_bridge::PluginBridge;
use nova_core::session::{resolve_session_action, SessionContext};
use nova_core::speech;
use nova_core::stt::{PythonStt, SttEngine};
use nova_core::{build_adapter, resolve_python_cmd};

const EXAMPLE_PHRASES: &[(&str, &[&str])] = &[
    (
        "Open / Focus",
        &[
            "open chrome",
            "launch vscode",
            "switch to chrome",
            "bring up notepad",
        ],
    ),
    (
        "Typing / Window",
        &[
            "type hello world",
            "write meeting starts at 5",
            "close window",
            "close explorer",
        ],
    ),
    (
        "Search",
        &[
            "search rust async tutorial",
            "search youtube lofi beats",
            "look up python regex on youtube",
            "find best rust crates",
        ],
    ),
    (
        "Screenshot / Volume",
        &[
            "take a screenshot",
            "capture screen",
            "volume up",
            "mute",
        ],
    ),
];

#[derive(Debug, Clone)]
enum EngineEvent {
    Status(String),
    Log(String),
    Session(String),
    Error(String),
}

struct NovaUiApp {
    status: String,
    session_summary: String,
    logs: Vec<String>,
    running: bool,
    stop_flag: Arc<AtomicBool>,
    engine_thread: Option<JoinHandle<()>>,
    event_rx: Receiver<EngineEvent>,
    event_tx: Sender<EngineEvent>,
}

impl NovaUiApp {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            status: "Idle".to_string(),
            session_summary: "last_action=none; pending_slot=none; entities=[none]".to_string(),
            logs: vec!["Nova UI ready.".to_string()],
            running: false,
            stop_flag: Arc::new(AtomicBool::new(false)),
            engine_thread: None,
            event_rx: rx,
            event_tx: tx,
        }
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
        if self.logs.len() > 300 {
            self.logs.drain(0..100);
        }
    }

    fn start_engine(&mut self) {
        if self.running {
            return;
        }
        self.stop_flag.store(false, Ordering::SeqCst);
        let tx = self.event_tx.clone();
        let stop = Arc::clone(&self.stop_flag);
        self.engine_thread = Some(thread::spawn(move || {
            if let Err(e) = run_engine_loop(stop, tx.clone()) {
                let _ = tx.send(EngineEvent::Error(e.to_string()));
            }
            let _ = tx.send(EngineEvent::Status("Stopped".to_string()));
        }));
        self.running = true;
    }

    fn stop_engine(&mut self) {
        if !self.running {
            return;
        }
        self.stop_flag.store(true, Ordering::SeqCst);
        self.push_log("Stop requested. Engine will stop after current capture cycle.");
    }
}

fn send_log(tx: &Sender<EngineEvent>, msg: impl Into<String>) {
    let _ = tx.send(EngineEvent::Log(msg.into()));
}

fn send_status(tx: &Sender<EngineEvent>, msg: impl Into<String>) {
    let _ = tx.send(EngineEvent::Status(msg.into()));
}

fn run_engine_loop(stop: Arc<AtomicBool>, tx: Sender<EngineEvent>) -> Result<()> {
    send_status(&tx, "Initializing");
    let root = config::project_root()?;
    let mut cfg = config::load_or_default(&root)?;
    cfg.python_cmd = resolve_python_cmd(&root, &cfg.python_cmd);
    config::save(&root, &cfg)?;

    send_log(&tx, format!("Project root: {}", root.display()));
    send_log(&tx, format!("Hotkey: {}", cfg.hotkey));
    send_log(&tx, format!("Python: {}", cfg.python_cmd));

    let stt = PythonStt {
        python_cmd: cfg.python_cmd.clone(),
        stt_script: root.join("plugins").join("stt_runner.py"),
    };
    onboarding::ensure_onboarded(&root, &mut cfg, &stt)?;
    config::save(&root, &cfg)?;
    send_log(&tx, "Onboarding completed.");

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
    send_status(&tx, "Idle");
    let _ = tx.send(EngineEvent::Session(session.summary()));
    send_log(
        &tx,
        format!("Engine running. Hold {} to issue voice commands.", cfg.hotkey),
    );

    while !stop.load(Ordering::SeqCst) {
        send_status(&tx, "Listening");
        let transcript = stt
            .capture_with_hotkey(&cfg.hotkey)
            .context("Failed to capture speech")?;
        if stop.load(Ordering::SeqCst) {
            break;
        }
        if transcript.trim().is_empty() {
            send_status(&tx, "Idle");
            continue;
        }
        session.add_transcript(&transcript);
        let _ = tx.send(EngineEvent::Session(session.summary()));
        send_log(&tx, format!("Heard: {}", transcript));

        send_status(&tx, "Processing");
        let mut action = bridge
            .parse_transcript(&transcript, session.context_for_parser())
            .context("Failed to parse transcript via plugins")?;
        action = resolve_session_action(&mut session, action);
        send_log(&tx, format!("Action: {}", action.action_type));

        while action.action_type == "clarify" && !stop.load(Ordering::SeqCst) {
            let question = action
                .params
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("Please repeat.");
            send_log(&tx, format!("Clarify: {}", question));
            speech::speak(question);
            send_status(&tx, "Listening");
            let retry = stt
                .capture_with_hotkey(&cfg.hotkey)
                .context("Failed while clarify loop")?;
            if stop.load(Ordering::SeqCst) {
                break;
            }
            session.add_transcript(&retry);
            action = bridge
                .parse_transcript(&retry, session.context_for_parser())
                .context("Clarify parse failed")?;
            action = resolve_session_action(&mut session, action);
        }

        if stop.load(Ordering::SeqCst) {
            break;
        }

        match dispatcher::dispatch(adapter.as_ref(), &action) {
            Ok(_) => {
                send_log(&tx, format!("Executed: {}", action.action_type));
                session.remember_action(&action);
                let _ = tx.send(EngineEvent::Session(session.summary()));
            }
            Err(e) => send_log(&tx, format!("Action failed: {e}")),
        }
        send_status(&tx, "Idle");
    }

    Ok(())
}

impl eframe::App for NovaUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(evt) = self.event_rx.try_recv() {
            match evt {
                EngineEvent::Status(s) => {
                    self.status = s.clone();
                    if s == "Stopped" {
                        self.running = false;
                    }
                }
                EngineEvent::Log(l) => self.push_log(l),
                EngineEvent::Session(s) => self.session_summary = s,
                EngineEvent::Error(e) => {
                    self.status = "Error".to_string();
                    self.running = false;
                    self.push_log(format!("Error: {e}"));
                }
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("Nova Control");
            ui.label(format!("Status: {}", self.status));
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.running, egui::Button::new("Start Engine"))
                    .clicked()
                {
                    self.start_engine();
                }
                if ui
                    .add_enabled(self.running, egui::Button::new("Stop Engine"))
                    .clicked()
                {
                    self.stop_engine();
                }
            });
            ui.label("Voice flow: hold hotkey, speak, release.");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::CollapsingHeader::new("Session Context")
                .default_open(true)
                .show(ui, |ui| {
                    ui.monospace(&self.session_summary);
                });
            ui.separator();
            egui::CollapsingHeader::new("Example Phrases")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("Speak naturally. Nova maps intent to supported actions.");
                    egui::ScrollArea::vertical()
                        .max_height(170.0)
                        .show(ui, |ui| {
                            for (section, phrases) in EXAMPLE_PHRASES {
                                ui.strong(*section);
                                for phrase in *phrases {
                                    ui.monospace(format!("- {}", phrase));
                                }
                                ui.add_space(6.0);
                            }
                        });
                });
            ui.separator();
            ui.heading("Logs");
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.logs {
                        ui.monospace(line);
                    }
                });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }
}

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Nova UI",
        native_options,
        Box::new(|_cc| Ok(Box::new(NovaUiApp::new()))),
    )
    .map_err(|e| anyhow::anyhow!("UI runtime failed: {e}"))?;
    Ok(())
}
