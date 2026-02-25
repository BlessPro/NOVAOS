# Changelog

All notable product changes to Nova are documented in this file.

## [0.1.0] - 2026-02-25

### Added
- End-to-end voice automation pipeline: push-to-talk, STT bridge, deterministic action schema, dispatcher, and OS adapters.
- Windows-first adapter actions (`open_app`, `focus_app`, `close_window`, `type_text`, `open_url`, `screenshot`, `volume`) with macOS/Linux stubs.
- Voice-first onboarding with name capture, yes/no confirmation, spelling mode, and local config persistence.
- UI app (`nova/ui`) with live status and logs.
- In-app example phrases panel for quick onboarding.
- Plugin-based intent parser with system, browser, and screenshot plugins.
- Local LLM parser plugin (`ollama`) and parser modes: `rule`, `llm`, `hybrid`.

### Changed
- Parser behavior evolved from strict full-phrase matching to permissive in-sentence matching for natural speech.
- Hybrid parsing now supports filler-word tolerance and natural phrasing before executing deterministic actions.
- Python command resolution now prefers project `.venv` when config uses generic `python`.

### Fixed
- Windows URL/app launch reliability improved by moving away from fragile `cmd start` behavior.
- Fixed local LLM subprocess Unicode decode failures on Windows (`cp1252` issues).
- Improved STT diagnostics and fallback behavior for empty transcript/error scenarios.

