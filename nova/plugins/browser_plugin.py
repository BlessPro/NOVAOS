import re
from typing import Dict, Optional

from common import build_action


def _clean_query(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip(" .,!?:;")


def _trim_at_connectors(text: str) -> str:
    return re.split(r"\b(?:and|then|after|also|while)\b", text, maxsplit=1)[0].strip()


def parse_browser_intent(text: str) -> Optional[Dict]:
    t = text.strip().lower()

    m = re.search(r"\b(?:search(?:\s+for)?|find|look\s+up)\b\s+(.+?)\s+\bon\s+youtube\b", t)
    if m:
        query = _clean_query(_trim_at_connectors(m.group(1)))
        if not query:
            return None
        return build_action(
            "search_web", {"query": query, "platform": "youtube"}
        )

    m = re.search(r"\b(?:search|find|look\s+up)\b\s+(?:on\s+)?youtube\s+(.+)", t)
    if m:
        query = _clean_query(_trim_at_connectors(m.group(1)))
        if not query:
            return None
        return build_action(
            "search_web", {"query": query, "platform": "youtube"}
        )

    m = re.search(r"\b(?:search(?:\s+for)?|find|look\s+up)\b\s+(.+)", t)
    if m:
        query = _clean_query(_trim_at_connectors(m.group(1)))
        if not query:
            return None
        return build_action(
            "search_web", {"query": query, "platform": "google"}
        )

    return None
