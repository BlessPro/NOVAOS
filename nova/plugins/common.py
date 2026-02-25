import json
import sys
from typing import Any, Dict

ACTION_VERSION = "0.1"


def build_action(action_type: str, params: Dict[str, Any]) -> Dict[str, Any]:
    return {"version": ACTION_VERSION, "type": action_type, "params": params}


def clarify(question: str = "Please repeat.") -> Dict[str, Any]:
    return build_action("clarify", {"question": question})


def validate_action(action: Dict[str, Any]) -> Dict[str, Any]:
    allowed = {
        "open_app",
        "focus_app",
        "close_window",
        "type_text",
        "search_web",
        "screenshot",
        "volume",
        "clarify",
    }
    if action.get("version") != ACTION_VERSION:
        raise ValueError("invalid action version")
    if action.get("type") not in allowed:
        raise ValueError("invalid action type")
    if not isinstance(action.get("params"), dict):
        raise ValueError("invalid params")
    return action


def read_stdin_json() -> Dict[str, Any]:
    raw = sys.stdin.read().strip()
    return json.loads(raw) if raw else {}


def write_stdout_json(payload: Dict[str, Any]) -> None:
    print(json.dumps(payload, ensure_ascii=True))
