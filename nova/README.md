# Nova V0.1

Deterministic voice-driven PC automation engine prototype.

## What V0.1 does

- Push-to-talk flow: hold hotkey, record, release, transcribe.
- Deterministic intent parsing via Python plugins.
- Versioned `Action` JSON contract (`version: "0.1"`).
- Rust dispatcher executes actions through OS adapter.
- Windows adapter implements practical actions; macOS/Linux compile with stubs.
- Voice-first onboarding with confirmation and spelling fallback.

## Repo layout

```
nova/
  core/
  plugins/
  assets/
  config/
```

## Setup

1. Install Python 3.10+ and Rust stable.
2. Create Python env and install STT dependency:

```powershell
cd nova
python -m venv .venv
.venv\Scripts\activate
pip install -r plugins/requirements.txt
```

3. Install local LLM backend (Ollama) and pull a model:

```powershell
ollama pull llama3.1:8b
```

4. Run Nova core:

```powershell
cd core
cargo run
```

Or run Nova with desktop UI:

```powershell
cd ui
cargo run
```

## Notes

- Default hotkey is `F8` (configurable in `config/config.json` after first run).
- UI shows engine state (`Idle`, `Listening`, `Processing`, `Error`) and logs.
- Parser defaults to `hybrid` (`rule` first, then local LLM via Ollama).
- If STT is unavailable, Nova falls back to typed input in console.
- `plugins/stt_runner.py` uses `faster-whisper` if installed.
- For offline deterministic parser behavior, only strict pattern matches are supported.
- TTS is enabled by default; set `NOVA_DISABLE_TTS=1` to force text-only prompts.

## Deterministic command examples

- `open chrome`
- `switch to chrome`
- `close window`
- `type hello world`
- `search python async tutorial`
- `search youtube lofi beats`
- `take a screenshot`
- `volume up`

## Acceptance coverage

- First run onboarding writes `config/config.json`.
- `open calculator` maps to Windows `calc`.
- `search youtube ...` opens YouTube results URL.
- `take a screenshot` writes timestamped PNG in `screenshots/`.
- Unknown commands return clarify action: `Please repeat.`
