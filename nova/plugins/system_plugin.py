import re
from typing import Dict, Optional

from common import build_action


def _clean_payload(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip(" .,!?:;")


def parse_system_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()

    m = re.fullmatch(
        r"(?:open|launch|start|run)\s+(?:the\s+)?(?:app\s+)?(.+?)(?:\s+for\s+me)?",
        t,
    )
    if m:
        return build_action("open_app", {"name": _clean_payload(m.group(1))})

    m = re.fullmatch(
        r"(?:switch\s+to|focus(?:\s+on)?|go\s+to|bring\s+up)\s+(.+?)(?:\s+for\s+me)?",
        t,
    )
    if m:
        return build_action("focus_app", {"name": _clean_payload(m.group(1))})

    if re.fullmatch(r"(?:close|exit|dismiss)\s+(?:this\s+|the\s+)?window", t):
        return build_action("close_window", {})

    if re.fullmatch(r"(?:close|exit)\s+.+", t):
        # V0.1 supports only closing current window, not app-by-name close.
        return build_action("close_window", {})

    m = re.fullmatch(r"(?:type|write|enter)\s+(.+)", text.strip(), re.IGNORECASE)
    if m:
        return build_action("type_text", {"text": _clean_payload(m.group(1))})

    if re.fullmatch(r"(?:volume\s+up|increase\s+volume|turn\s+volume\s+up)", t):
        return build_action("volume", {"action": "up"})
    if re.fullmatch(r"(?:volume\s+down|decrease\s+volume|turn\s+volume\s+down)", t):
        return build_action("volume", {"action": "down"})
    if re.fullmatch(r"(?:mute|volume\s+mute|turn\s+volume\s+off)", t):
        return build_action("volume", {"action": "mute"})

    return None
