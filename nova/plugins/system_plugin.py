import re
from typing import Dict, Optional

from common import build_action


def parse_system_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()

    m = re.fullmatch(r"open\s+(.+)", t)
    if m:
        return build_action("open_app", {"name": m.group(1).strip()})

    m = re.fullmatch(r"(switch to|focus)\s+(.+)", t)
    if m:
        return build_action("focus_app", {"name": m.group(2).strip()})

    if t == "close window":
        return build_action("close_window", {})

    m = re.fullmatch(r"type\s+(.+)", text.strip(), re.IGNORECASE)
    if m:
        return build_action("type_text", {"text": m.group(1)})

    if t == "volume up":
        return build_action("volume", {"action": "up"})
    if t == "volume down":
        return build_action("volume", {"action": "down"})
    if t in {"mute", "volume mute"}:
        return build_action("volume", {"action": "mute"})

    return None
