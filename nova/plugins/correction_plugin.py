import re
from typing import Dict, Optional

from common import build_action


def _normalize_spaces(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip(" .,!?:;")


def _parse_spelled_value(raw: str) -> str:
    # Supports: letters A-Z, space, dash, apostrophe, backspace.
    out = []
    for token in raw.lower().split():
        if token in {"it", "is"} and not out:
            continue
        if token in {"done"}:
            break
        if token == "backspace":
            if out:
                out.pop()
            continue
        if token == "space":
            out.append(" ")
            continue
        if token == "dash":
            out.append("-")
            continue
        if token == "apostrophe":
            out.append("'")
            continue
        if len(token) == 1 and token.isalpha():
            out.append(token)
            continue
        out.append(token)
    return _normalize_spaces("".join(out))


def parse_correction_intent(text: str, context: dict) -> Optional[Dict]:
    t = text.strip().lower()
    session = context.get("session", {}) if isinstance(context, dict) else {}
    last_action = str(session.get("last_action_type", "")).strip().lower()

    if re.search(r"\b(?:yes|correct|that's right)\b", t):
        return build_action("confirm", {})

    m = re.search(r"\b(?:spell(?:ing)?(?:\s+is)?|spell\s+it)\b\s+(.+)", t)
    if m:
        spelled = _parse_spelled_value(m.group(1))
        if spelled:
            return build_action("spelling_update", {"value": spelled})

    m = re.search(r"\b(?:no[, ]+)?(?:i\s+meant|use)\b\s+(.+?)(?:\s+instead)?$", t)
    if m:
        value = _normalize_spaces(m.group(1))
        if value:
            return build_action("correction", {"value": value})

    # Reference intents rely on session memory ("this"/"that").
    if re.search(r"\bsearch\s+(?:this|that|it)\b", t):
        platform = "youtube" if "youtube" in t else "google"
        return build_action("reference", {"target": "search_web", "platform": platform})

    if re.search(r"\b(?:open|launch|start)\s+(?:this|that|it)\b", t):
        return build_action("reference", {"target": "open_app"})

    if re.search(r"\b(?:type|write|enter)\s+(?:this|that|it)\b", t):
        return build_action("reference", {"target": "type_text"})

    if last_action and re.search(r"\b(?:no|actually)\b", t):
        # Generic correction marker with no value.
        return build_action(
            "clarify",
            {"question": "What should I change it to? You can say: I meant <value>."},
        )

    return None
