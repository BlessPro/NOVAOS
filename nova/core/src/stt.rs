use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use device_query::{DeviceQuery, DeviceState, Keycode};
use serde::Deserialize;

pub trait SttEngine {
    fn capture_with_hotkey(&self, hotkey: &str) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct PythonStt {
    pub python_cmd: String,
    pub stt_script: PathBuf,
}

#[derive(Debug, Deserialize)]
struct SttResponse {
    #[serde(default)]
    text: String,
    #[serde(default)]
    error: Option<String>,
}

fn key_from_hotkey(hotkey: &str) -> Keycode {
    match hotkey.to_uppercase().as_str() {
        "F1" => Keycode::F1,
        "F2" => Keycode::F2,
        "F3" => Keycode::F3,
        "F4" => Keycode::F4,
        "F5" => Keycode::F5,
        "F6" => Keycode::F6,
        "F7" => Keycode::F7,
        "F8" => Keycode::F8,
        "F9" => Keycode::F9,
        "F10" => Keycode::F10,
        "F11" => Keycode::F11,
        "F12" => Keycode::F12,
        _ => Keycode::F8,
    }
}

impl PythonStt {
    fn transcribe_file(&self, wav_path: &PathBuf) -> Result<SttResponse> {
        let payload = serde_json::json!({
            "audio_path": wav_path.to_string_lossy()
        });

        let mut child = Command::new(&self.python_cmd)
            .arg(self.stt_script.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to launch STT python process")?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(payload.to_string().as_bytes())?;
        }

        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(anyhow!("STT script failed"));
        }

        let parsed: SttResponse = serde_json::from_slice(&out.stdout).context("Invalid STT response JSON")?;
        Ok(parsed)
    }

    fn record_push_to_talk(&self, hotkey: &str) -> Result<PathBuf> {
        let target_key = key_from_hotkey(hotkey);
        let device_state = DeviceState::new();

        println!("Hold {hotkey} to speak...");
        loop {
            let keys = device_state.get_keys();
            if keys.contains(&target_key) {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        println!("Listening...");

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow!("No default input audio device found"))?;
        let config = device.default_input_config().context("No default input config")?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let collected: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = Arc::clone(&collected);

        let err_fn = |err| eprintln!("Audio input stream error: {err}");

        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[i16], _| {
                    if let Ok(mut guard) = collected_clone.lock() {
                        guard.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[u16], _| {
                    if let Ok(mut guard) = collected_clone.lock() {
                        guard.extend(data.iter().map(|v| (*v as i32 - 32768) as i16));
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _| {
                    if let Ok(mut guard) = collected_clone.lock() {
                        guard.extend(data.iter().map(|v| (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16));
                    }
                },
                err_fn,
                None,
            )?,
            _ => return Err(anyhow!("Unsupported sample format")),
        };

        stream.play().context("Failed to start audio stream")?;

        loop {
            let keys = device_state.get_keys();
            if !keys.contains(&target_key) {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }

        drop(stream);
        println!("Processing...");

        let samples = {
            let guard = collected.lock().map_err(|_| anyhow!("Audio lock poisoned"))?;
            guard.clone()
        };
        if samples.is_empty() {
            return Err(anyhow!("No audio captured"));
        }

        let tmp_dir = std::env::temp_dir().join("nova");
        fs::create_dir_all(&tmp_dir)?;
        let path = tmp_dir.join(format!(
            "ptt_{}.wav",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        ));
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec)?;
        for s in samples {
            writer.write_sample(s)?;
        }
        writer.finalize()?;

        Ok(path)
    }
}

impl SttEngine for PythonStt {
    fn capture_with_hotkey(&self, hotkey: &str) -> Result<String> {
        let wav = self.record_push_to_talk(hotkey)?;
        let stt_response = match self.transcribe_file(&wav) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("STT bridge failed: {e}");
                SttResponse {
                    text: String::new(),
                    error: Some(e.to_string()),
                }
            }
        };
        let _ = fs::remove_file(&wav);
        if let Some(err) = stt_response.error.as_deref() {
            if !err.trim().is_empty() {
                eprintln!("STT info: {err}");
            }
        }
        let transcript = stt_response.text.trim().to_string();

        if transcript.is_empty() {
            print!("Transcript empty. Type fallback input: ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            return Ok(line.trim().to_string());
        }

        println!("Heard: {}", transcript);
        Ok(transcript)
    }
}
