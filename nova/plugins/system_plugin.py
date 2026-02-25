import re
from typing import Dict, Optional

from common import build_action


def _clean_payload(text: str) -> str:
    cleaned = re.sub(r"\s+", " ", text).strip(" .,!?:;")
    cleaned = re.sub(r"\b(?:for me|please|now)$", "", cleaned).strip(" .,!?:;")
    return cleaned


def _trim_at_connectors(text: str) -> str:
    return re.split(r"\b(?:and|then|after|also|while)\b", text, maxsplit=1)[0].strip()


def parse_system_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()

    m = re.search(
        r"\b(?:open|launch|start|run)\b(?:\s+the)?(?:\s+app)?\s+(.+)",
        t,
    )
    if m:
        name = _clean_payload(_trim_at_connectors(m.group(1)))
        if name:
            return build_action("open_app", {"name": name})

    m = re.search(
        r"\b(?:switch\s+to|focus(?:\s+on)?|go\s+to|bring\s+up)\b\s+(.+)",
        t,
    )
    if m:
        name = _clean_payload(_trim_at_connectors(m.group(1)))
        if name:
            return build_action("focus_app", {"name": name})

    if re.search(r"\b(?:close|exit|dismiss)\b.*\bwindow\b", t):
        return build_action("close_window", {})

    if re.search(r"\b(?:close|exit)\b", t):
        # V0.1 supports only closing current window, not app-by-name close.
        return build_action("close_window", {})

    m = re.search(r"\b(?:type|write|enter)\b\s+(.+)", text.strip(), re.IGNORECASE)
    if m:
        typed = _clean_payload(_trim_at_connectors(m.group(1)))
        if typed:
            return build_action("type_text", {"text": typed})

    if re.search(r"\b(?:volume\s+up|increase\s+volume|turn\s+volume\s+up)\b", t):
        return build_action("volume", {"action": "up"})
    if re.search(r"\b(?:volume\s+down|decrease\s+volume|turn\s+volume\s+down)\b", t):
        return build_action("volume", {"action": "down"})
    if re.search(r"\b(?:mute|volume\s+mute|turn\s+volume\s+off)\b", t):
        return build_action("volume", {"action": "mute"})

    return None
