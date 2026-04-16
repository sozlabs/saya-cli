# saya-cli

Thin Rust CLI client for `soz-saya`.

## Scope

`saya-cli` is a transport client only. It talks to `soz-saya` over HTTP/SSE and does not contain server-side agent logic.

## DoD 0 Guardrails

- No direct LLM providers in CLI code (OpenAI/Anthropic/Groq/etc).
- No RAG/indexing/vector store implementation in CLI.
- No agent planner/runtime/tool orchestration in CLI.
- Only client transport to `soz-saya`.

Reference plan: `plans.md` (DoD 0).

## Current Commands

- `saya version`
- `saya health`
- `saya chat --message "..."`

## Global Flags (DoD 1)

- `--base-url <url>`: override `soz-saya` base URL.
- `--timeout-ms <n>`: transport timeout.
- `--output <text|json>`: output format.
- `--non-interactive`: disable interactive behavior.
- `--debug`: enable transport diagnostics with secret redaction.
- `--conversation-id <id>`: continue existing conversation.
- `--session-id <id>`: client session scope for conversation/message context.
- `--tenant-id <id>`: tenant scope for conversation/message context.
- `--actor-id <id>`: actor scope for conversation/message context.
- `--channel-id <id>`: channel scope (default `terminal`).

## Environment Overrides (DoD 2)

- `SAYA_BASE_URL`
- `SAYA_TIMEOUT_MS`
- `SAYA_OUTPUT_FORMAT` (`text` or `json`)
- `SAYA_NON_INTERACTIVE` (`true|false` / `1|0`)
- `SAYA_DEBUG` (`true|false` / `1|0`)
- `SAYA_CONFIG_DIR` (optional override for config directory)
- `SAYA_CONVERSATION_ID`
- `SAYA_SESSION_ID`
- `SAYA_TENANT_ID`
- `SAYA_ACTOR_ID`
- `SAYA_CHANNEL_ID`

Resolution order: CLI flags > environment > config file > defaults.

## Sensitive Data Storage (DoD 2)

Credentials and config are stored only under OS standard config directories (or `SAYA_CONFIG_DIR` override), never in current working directory.

- Linux/macOS: `~/.config/saya/`
- Windows: `%APPDATA%\\saya\\`

Sensitive values are never printed raw in debug logs.

## Auth and Session (DoD 3–4)

- `chat` requires a token in credentials storage; the client sends `Authorization: Bearer ...` only when token is present.
- If `conversation_id` is not provided, the client creates a new conversation and then sends the message.
- If `conversation_id` is provided (flag, env, or stored credentials), the client resumes that conversation.
- `channel_id` defaults to `terminal` and is sent in message context.
