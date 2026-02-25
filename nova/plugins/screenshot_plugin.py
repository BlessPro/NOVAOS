from typing import Dict, Optional

from common import build_action


def parse_screenshot_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()
    variants = {
        "take a screenshot",
        "take screenshot",
        "capture screenshot",
        "capture screen",
        "take a screen shot",
        "screenshot",
        "screen shot",
    }
    if t in variants or "screenshot" in t or "screen shot" in t or "capture screen" in t:
        return build_action("screenshot", {"mode": "full", "destination": "file"})
    return None
