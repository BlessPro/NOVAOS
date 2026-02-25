from typing import Dict, Optional

from common import build_action


def parse_screenshot_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()
    if t in {"take a screenshot", "screenshot", "take screenshot"}:
        return build_action("screenshot", {"mode": "full", "destination": "file"})
    return None
