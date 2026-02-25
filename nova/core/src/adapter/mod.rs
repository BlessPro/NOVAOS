use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenshotMode {
    Full,
    Window,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenshotDestination {
    File,
    Clipboard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeAction {
    Up,
    Down,
    Mute,
}

pub trait OsAdapter: Send + Sync {
    fn open_app(&self, name: &str) -> Result<()>;
    fn focus_app(&self, name: &str) -> Result<()>;
    fn close_window(&self) -> Result<()>;
    fn type_text(&self, text: &str) -> Result<()>;
    fn open_url(&self, url: &str) -> Result<()>;
    fn screenshot(&self, mode: ScreenshotMode, destination: ScreenshotDestination) -> Result<()>;
    fn volume(&self, action: VolumeAction) -> Result<()>;
}

pub mod linux;
pub mod mac;
pub mod windows;
