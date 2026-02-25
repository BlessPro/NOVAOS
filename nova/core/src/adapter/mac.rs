use anyhow::{anyhow, Result};

use super::{OsAdapter, ScreenshotDestination, ScreenshotMode, VolumeAction};

#[derive(Debug, Clone)]
pub struct MacAdapter;

impl MacAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl OsAdapter for MacAdapter {
    fn open_app(&self, _name: &str) -> Result<()> {
        Err(anyhow!("Not implemented on macOS in V0.1"))
    }
    fn focus_app(&self, _name: &str) -> Result<()> {
        Err(anyhow!("Not implemented on macOS in V0.1"))
    }
    fn close_window(&self) -> Result<()> {
        Err(anyhow!("Not implemented on macOS in V0.1"))
    }
    fn type_text(&self, _text: &str) -> Result<()> {
        Err(anyhow!("Not implemented on macOS in V0.1"))
    }
    fn open_url(&self, _url: &str) -> Result<()> {
        Err(anyhow!("Not implemented on macOS in V0.1"))
    }
    fn screenshot(&self, _mode: ScreenshotMode, _destination: ScreenshotDestination) -> Result<()> {
        Err(anyhow!("Not implemented on macOS in V0.1"))
    }
    fn volume(&self, _action: VolumeAction) -> Result<()> {
        Err(anyhow!("Not implemented on macOS in V0.1"))
    }
}
