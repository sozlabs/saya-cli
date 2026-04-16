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

## Environment Overrides (DoD 2)

- `SAYA_BASE_URL`
- `SAYA_TIMEOUT_MS`
- `SAYA_OUTPUT_FORMAT` (`text` or `json`)
- `SAYA_NON_INTERACTIVE` (`true|false` / `1|0`)
- `SAYA_DEBUG` (`true|false` / `1|0`)
- `SAYA_CONFIG_DIR` (optional override for config directory)

Resolution order: CLI flags > environment > config file > defaults.

## Sensitive Data Storage (DoD 2)

Credentials and config are stored only under OS standard config directories (or `SAYA_CONFIG_DIR` override), never in current working directory.

- Linux/macOS: `~/.config/saya/`
- Windows: `%APPDATA%\\saya\\`

Sensitive values are never printed raw in debug logs.
