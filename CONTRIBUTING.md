# Contributing to saya-cli

## Architecture Contract (DoD 0)

Before opening a PR, ensure all changes keep `saya-cli` a thin client:

- Allowed: CLI UX, config, HTTP/SSE transport, parsing server events.
- Not allowed: direct LLM API calls, embedded agent runtime, RAG/vector retrieval logic.

## Mandatory Checks

- Run `cargo fmt -- --check`
- Run `cargo clippy -- -D warnings`
- Run `cargo test`
- Run boundary checks:
  - `bash .ci/check-boundaries.sh`

## Design Notes

- New commands must route through transport modules.
- Any contract fields must come from `soz-saya` schemas/OpenAPI, not invented in CLI.
- Keep secrets out of logs.
- Respect DoD 1/2 config contract:
  - flags/env/file/default precedence,
  - OS-safe config/credentials paths only,
  - non-interactive behavior must remain predictable.
- Debug output must always pass through redaction; never log full Authorization/refresh/cookie values.
- Respect DoD 3/4 auth-session contract:
  - `chat` must pass Authorization only via secure token source,
  - no token value in logs/errors,
  - resume flow must honor `conversation_id` and `channel_id=terminal` default.
- Runtime code must avoid `unwrap`/`expect` and keep error handling explicit.
