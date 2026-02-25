import re
from typing import Dict, Optional

from common import build_action


def _clean_query(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip(" .,!?:;")


def parse_browser_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()

    m = re.fullmatch(r"(?:search(?:\s+for)?|find|look\s+up)\s+(.+?)\s+on\s+youtube", t)
    if m:
        return build_action(
            "search_web", {"query": _clean_query(m.group(1)), "platform": "youtube"}
        )

    m = re.fullmatch(r"(?:search|find|look\s+up)\s+(?:on\s+)?youtube\s+(.+)", t)
    if m:
        return build_action(
            "search_web", {"query": _clean_query(m.group(1)), "platform": "youtube"}
        )

    m = re.fullmatch(r"(?:search(?:\s+for)?|find|look\s+up)\s+(.+)", t)
    if m:
        return build_action(
            "search_web", {"query": _clean_query(m.group(1)), "platform": "google"}
        )

    return None
