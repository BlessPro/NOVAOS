import re
from typing import Dict, Optional

from common import build_action


def parse_browser_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()

    m = re.fullmatch(r"search youtube\s+(.+)", t)
    if m:
        return build_action(
            "search_web", {"query": m.group(1).strip(), "platform": "youtube"}
        )

    m = re.fullmatch(r"search\s+(.+)", t)
    if m:
        return build_action(
            "search_web", {"query": m.group(1).strip(), "platform": "google"}
        )

    return None
