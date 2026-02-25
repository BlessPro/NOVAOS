from common import clarify, read_stdin_json, validate_action, write_stdout_json
from browser_plugin import parse_browser_intent
from llm_plugin import parse_with_llm
from screenshot_plugin import parse_screenshot_intent
from system_plugin import parse_system_intent


def _normalize_tokens(text: str) -> str:
    return " ".join(text.split())


def _normalize_transcript(transcript: str) -> str:
    # Drop filler words while preserving command payload words.
    filler = {
        "please",
        "could",
        "would",
        "can",
        "you",
        "kindly",
        "just",
        "hey",
        "nova",
        "now",
    }
    t = transcript.strip().lower()
    t = t.replace("screen shot", "screenshot")
    t = t.replace("vs code", "vscode")
    tokens = [tok.strip(".,!?;:") for tok in t.split()]
    cleaned = [tok for tok in tokens if tok and tok not in filler]
    return _normalize_tokens(" ".join(cleaned)).strip() or transcript.strip()


def _parse_rules(transcript: str):
    parsers = [parse_system_intent, parse_browser_intent, parse_screenshot_intent]
    for parser in parsers:
        action = parser(transcript)
        if action is not None:
            return action
    return None


def parse(transcript: str, context: dict):
    parser_mode = str(context.get("parser_mode", "hybrid")).strip().lower()
    llm_backend = str(context.get("llm_backend", "ollama")).strip().lower()
    llm_model = str(context.get("llm_model", "llama3.1:8b")).strip()
    llm_timeout_ms = int(context.get("llm_timeout_ms", 12000))

    normalized = _normalize_transcript(transcript)
    if parser_mode in {"rule", "hybrid"}:
        action = _parse_rules(normalized) or _parse_rules(transcript)
        if action is not None:
            return action

    if parser_mode in {"llm", "hybrid"}:
        action = parse_with_llm(
            normalized, backend=llm_backend, model=llm_model, timeout_ms=llm_timeout_ms
        )
        if action is not None:
            return action

    return clarify("Please repeat.")


def main():
    payload = read_stdin_json()
    transcript = str(payload.get("transcript", "")).strip()
    context = payload.get("context", {}) or {}
    action = parse(transcript, context) if transcript else clarify("Please repeat.")
    write_stdout_json(validate_action(action))


if __name__ == "__main__":
    main()
