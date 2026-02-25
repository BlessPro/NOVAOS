# Decisions Log

This file records project/process decisions with timestamps.

## DEC-001: Branching Strategy by Work Scope
- Timestamp: 2026-02-25
- Status: Accepted
- Context:
  - Work spans distinct domains (`adapter`, `plugin`, `security`, `ui`) plus occasional cross-cutting tasks.
- Decision:
  - Use `main` as stable branch.
  - Use scoped feature branches:
    - `feature/adapter/*`
    - `feature/plugin/*`
    - `feature/security/*`
    - `feature/ui/*`
    - `feature/full-change/*` for cross-cutting work.
  - Push completed work to the appropriate scoped branch.
- Consequences:
  - Clear ownership per change area.
  - Lower merge risk for focused tasks.
  - Cross-cutting work remains explicit and auditable.

## DEC-002: Intent Parsing Architecture (Rules + Local LLM)
- Timestamp: 2026-02-25
- Status: Accepted
- Context:
  - Strict rule-only parsing was too rigid for natural speech.
  - Privacy requirement favors local inference.
- Decision:
  - Keep deterministic Action execution contract in Rust.
  - Add local LLM parsing via Ollama with `parser_mode`:
    - `rule`
    - `llm`
    - `hybrid` (default; rules first, LLM fallback)
  - Continue schema validation before execution.
- Consequences:
  - Better natural-language understanding.
  - Deterministic runtime safety remains enforced by action schema and dispatcher.
  - Local model dependency required for full hybrid behavior.

## DEC-003: Session-Based Correction Memory (In-Engine Context)
- Timestamp: 2026-02-25
- Status: Planned
- Context:
  - Users need conversational corrections like:
    - "search this"
    - "the spelling is ..."
    - "no, use this instead"
  - User requested this be captured as a decision, not a separate spec file.
- Decision:
  - Add in-memory session context in runtime (not persisted by default in V0.1) with:
    - `last_action`
    - `last_entities` (app name/query/text)
    - `pending_slot` (field waiting for clarification)
    - short transcript/action history window
  - Extend parser to consume context for correction intents and apply deterministic updates.
- Consequences:
  - More natural conversational repair without unsafe guessing.
  - Requires careful precedence rules to avoid accidental carry-over.
  - Must include clear reset/timeout behavior for stale context.

