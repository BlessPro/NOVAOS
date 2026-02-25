use anyhow::{anyhow, Result};

use crate::actions::Action;
use crate::adapter::{OsAdapter, ScreenshotDestination, ScreenshotMode, VolumeAction};

pub fn dispatch(adapter: &dyn OsAdapter, action: &Action) -> Result<()> {
    match action.action_type.as_str() {
        "open_app" => {
            let name = action
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing param name"))?;
            adapter.open_app(name)
        }
        "focus_app" => {
            let name = action
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing param name"))?;
            adapter.focus_app(name)
        }
        "close_window" => adapter.close_window(),
        "type_text" => {
            let text = action
                .params
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing param text"))?;
            adapter.type_text(text)
        }
        "search_web" => {
            let query = action
                .params
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing param query"))?;
            let platform = action
                .params
                .get("platform")
                .and_then(|v| v.as_str())
                .unwrap_or("google");
            let encoded = urlencoding::encode(query);
            let url = match platform {
                "youtube" => format!("https://www.youtube.com/results?search_query={encoded}"),
                _ => format!("https://www.google.com/search?q={encoded}"),
            };
            adapter.open_url(&url)
        }
        "screenshot" => {
            let mode = match action
                .params
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("full")
            {
                "full" => ScreenshotMode::Full,
                "window" => ScreenshotMode::Window,
                "area" => ScreenshotMode::Area,
                x => return Err(anyhow!("Invalid screenshot mode: {x}")),
            };
            let destination = match action
                .params
                .get("destination")
                .and_then(|v| v.as_str())
                .unwrap_or("file")
            {
                "file" => ScreenshotDestination::File,
                "clipboard" => ScreenshotDestination::Clipboard,
                x => return Err(anyhow!("Invalid screenshot destination: {x}")),
            };
            adapter.screenshot(mode, destination)
        }
        "volume" => {
            let va = match action
                .params
                .get("action")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing param action"))?
            {
                "up" => VolumeAction::Up,
                "down" => VolumeAction::Down,
                "mute" => VolumeAction::Mute,
                x => return Err(anyhow!("Invalid volume action: {x}")),
            };
            adapter.volume(va)
        }
        "clarify" => Ok(()),
        "reference" | "correction" | "spelling_update" | "confirm" => {
            Err(anyhow!("Session intent reached dispatcher without resolution"))
        }
        x => Err(anyhow!("Unhandled action type: {x}")),
    }
}
