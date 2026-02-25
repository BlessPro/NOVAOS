import json
import re
import subprocess
from typing import Any, Dict, Optional

from common import validate_action


def _extract_json_object(raw: str) -> Optional[Dict[str, Any]]:
    match = re.search(r"\{.*\}", raw, flags=re.DOTALL)
    if not match:
        return None
    try:
        obj = json.loads(match.group(0))
    except Exception:
        return None
    if not isinstance(obj, dict):
        return None
    return obj


def _build_prompt(transcript: str) -> str:
    return (
        "You convert user commands into strict JSON actions for desktop automation.\n"
        "Allowed types: open_app, focus_app, close_window, type_text, search_web, screenshot, volume, clarify.\n"
        "Required format only:\n"
        '{"version":"0.1","type":"<type>","params":{...}}\n'
        "Rules:\n"
        "- Ignore filler words like please/can you/could you.\n"
        "- Use only allowed types and params.\n"
        "- search_web params: query, platform (google|youtube).\n"
        "- screenshot params: mode (full|window|area), destination (file|clipboard).\n"
        "- volume params: action (up|down|mute).\n"
        "- If unclear, return clarify with question 'Please repeat.'.\n"
        "Output JSON only. No markdown.\n"
        f'Input: "{transcript}"\n'
    )


def parse_with_llm(
    transcript: str, backend: str, model: str, timeout_ms: int
) -> Optional[Dict[str, Any]]:
    if backend != "ollama":
        return None

    prompt = _build_prompt(transcript)
    try:
        proc = subprocess.run(
            ["ollama", "run", model, prompt],
            capture_output=True,
            text=True,
            timeout=max(2, int(timeout_ms / 1000)),
            check=False,
        )
    except Exception:
        return None

    if proc.returncode != 0:
        return None
    parsed = _extract_json_object(proc.stdout.strip())
    if not parsed:
        return None

    try:
        return validate_action(parsed)
    except Exception:
        return None
