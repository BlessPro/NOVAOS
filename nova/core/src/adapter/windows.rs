use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::{OsAdapter, ScreenshotDestination, ScreenshotMode, VolumeAction};

#[derive(Debug, Clone)]
pub struct WindowsAdapter {
    app_index: HashMap<String, String>,
    screenshots_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct AppIndex {
    apps: HashMap<String, String>,
}

impl WindowsAdapter {
    pub fn new(app_index_path: PathBuf, screenshots_dir: PathBuf) -> Result<Self> {
        let raw = fs::read_to_string(&app_index_path)
            .with_context(|| format!("Failed to read app index: {}", app_index_path.display()))?;
        let index: AppIndex = serde_json::from_str(&raw).context("Invalid app index JSON")?;

        Ok(Self {
            app_index: index
                .apps
                .into_iter()
                .map(|(k, v)| (k.to_lowercase(), v))
                .collect(),
            screenshots_dir,
        })
    }

    fn run_ps(&self, script: &str) -> Result<()> {
        let status = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(script)
            .status()
            .context("Failed to run powershell")?;

        if !status.success() {
            return Err(anyhow!("Powershell command failed"));
        }
        Ok(())
    }

    fn normalize_app_name(name: &str) -> String {
        let lowered = name.trim().to_lowercase();
        let compact: String = lowered
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        match compact.as_str() {
            "vscode" | "visualstudiocode" | "visualstudio" => "vscode".to_string(),
            "msedge" | "microsoftedge" | "edge" => "edge".to_string(),
            "googlechrome" => "chrome".to_string(),
            "firefoxbrowser" => "firefox".to_string(),
            _ => compact,
        }
    }

    fn try_start_process(&self, target: &str) -> Result<()> {
        let script = format!("Start-Process -FilePath '{}'", target.replace('\'', "''"));
        self.run_ps(&script)
    }
}

impl OsAdapter for WindowsAdapter {
    fn open_app(&self, name: &str) -> Result<()> {
        let normalized = Self::normalize_app_name(name);

        if let Some(target) = self.app_index.get(&normalized) {
            self.try_start_process(target)
                .context("Failed to open app from index")?;
            return Ok(());
        }

        // Retry fallback with raw name first, then compact alias.
        if self.try_start_process(name).is_ok() {
            return Ok(());
        }
        self.try_start_process(&normalized)
            .context("Failed to open app by fallback")
    }

    fn focus_app(&self, name: &str) -> Result<()> {
        let script = format!(
            "(New-Object -ComObject WScript.Shell).AppActivate('{}') | Out-Null",
            name.replace('\'', "''")
        );
        self.run_ps(&script)
    }

    fn close_window(&self) -> Result<()> {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.SendKeys]::SendWait('%{F4}')
"#;
        self.run_ps(script)
    }

    fn type_text(&self, text: &str) -> Result<()> {
        let escaped = text
            .replace('{', "{{}")
            .replace('}', "{}}")
            .replace('+', "{+}")
            .replace('^', "{^}")
            .replace('%', "{%}")
            .replace('~', "{~}")
            .replace('(', "{(}")
            .replace(')', "{)}");
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{}')",
            escaped.replace('\'', "''")
        );
        self.run_ps(&script)
    }

    fn open_url(&self, url: &str) -> Result<()> {
        let script = format!(
            "Start-Process '{}'",
            url.replace('\'', "''")
        );
        self.run_ps(&script).context("Failed to open URL")
    }

    fn screenshot(&self, mode: ScreenshotMode, destination: ScreenshotDestination) -> Result<()> {
        if !matches!(mode, ScreenshotMode::Full) || !matches!(destination, ScreenshotDestination::File) {
            return Err(anyhow!(
                "V0.1 Windows screenshot supports only mode=full and destination=file"
            ));
        }

        fs::create_dir_all(&self.screenshots_dir).context("Failed to create screenshots dir")?;
        let filename = format!("screenshot_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let path = self.screenshots_dir.join(filename);
        let escaped_path = path.to_string_lossy().replace('\'', "''");

        let script = format!(
            r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bitmap = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$bitmap.Save('{0}', [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bitmap.Dispose()
"#,
            escaped_path
        );
        self.run_ps(&script)
    }

    fn volume(&self, action: VolumeAction) -> Result<()> {
        let key = match action {
            VolumeAction::Up => "{VOLUME_UP}",
            VolumeAction::Down => "{VOLUME_DOWN}",
            VolumeAction::Mute => "{VOLUME_MUTE}",
        };
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{}')",
            key
        );
        self.run_ps(&script)
    }
}
