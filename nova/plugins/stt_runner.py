import json
import os
import sys
from typing import Dict, Optional, Tuple


def transcribe(audio_path: str) -> Tuple[str, Optional[str]]:
    fallback = os.getenv("NOVA_STT_FALLBACK_TEXT")
    if fallback:
        return fallback.strip(), None

    model_size = os.getenv("NOVA_WHISPER_MODEL", "tiny")
    try:
        from faster_whisper import WhisperModel
        model = WhisperModel(model_size, device="cpu", compute_type="int8")
        segments, _ = model.transcribe(audio_path, beam_size=1, vad_filter=True)
        text = " ".join(seg.text.strip() for seg in segments).strip()
        return text, None
    except Exception as e_fast:
        try:
            import whisper

            model = whisper.load_model(model_size)
            result = model.transcribe(audio_path, fp16=False)
            text = str(result.get("text", "")).strip()
            return text, None
        except Exception as e_openai:
            error = (
                "No STT backend available. Install faster-whisper or whisper. "
                f"faster-whisper error: {e_fast}; whisper error: {e_openai}"
            )
            return "", error


def main() -> None:
    raw = sys.stdin.read().strip()
    payload: Dict = json.loads(raw) if raw else {}
    audio_path = str(payload.get("audio_path", ""))
    text, error = transcribe(audio_path) if audio_path else ("", "Missing audio_path")
    print(json.dumps({"text": text, "error": error}, ensure_ascii=True))


if __name__ == "__main__":
    main()
