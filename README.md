# saya-cli

Thin Rust CLI client for the **soz-saya** HTTP API. It performs transport only: HTTP requests, SSE framing, and terminal output. It does not run agent logic, RAG, or call LLM providers directly.

## Scope

- Talks to **soz-saya** over HTTP (`/health`, conversations, message streaming).
- Parses **Server-Sent Events** from `POST /v1/conversations/{id}/messages/stream` as defined by the server contract (see **Streaming** below).
- Stores credentials and session defaults under OS config directories (see **Configuration**).

## Requirements

- Rust toolchain (edition 2021) with `cargo`.
- A running **soz-saya** instance (default base URL `http://127.0.0.1:3010`).

## Build

```bash
cargo build --release
```

The default release binary is `target/release/saya_cli` (crate package name `saya_cli`). The clap program name in `--help` is `saya`; you can rename or symlink the binary to `saya` if you prefer.

## Commands

| Command | Description |
|--------|-------------|
| `saya version` | Print CLI version. |
| `saya health` | `GET {base-url}/health`. |
| `saya chat --message "…"` | Create or resume a conversation and send a user message; response arrives over **SSE** and is printed as it streams (text mode). |

Use `saya --help` and `saya chat --help` for flag details.

## Global flags

| Flag | Meaning |
|------|---------|
| `--base-url <url>` | soz-saya base URL. |
| `--timeout-ms <n>` | Connect / blocking HTTP timeout for non-stream calls; stream uses this as **connect** timeout. |
| `--output <text\|json>` | Text: stream tokens to stdout. JSON: buffer tokens and print one JSON object at the end. |
| `--non-interactive` | Reserved for future prompts; prefer explicit flags in automation. |
| `--debug` | Transport logs with **redacted** secrets (no raw `Authorization` values). |
| `--conversation-id <id>` | Resume an existing conversation. |
| `--session-id`, `--tenant-id`, `--actor-id`, `--channel-id` | Conversation context (defaults match local “terminal” presets). |
| `--stream-max-retries <n>` | After a transport failure or incomplete stream, how many **additional** full `POST` retries to attempt (default `3`). |
| `--stream-retry-initial-ms <n>` | First backoff delay between retries (default `500`). |
| `--stream-retry-max-ms <n>` | Cap for exponential backoff (default `8000`). |

## Environment variables

Overrides follow: **CLI flags → environment → `config.json` → defaults**.

| Variable | Purpose |
|----------|---------|
| `SAYA_BASE_URL` | Base URL. |
| `SAYA_TIMEOUT_MS` | Timeout in milliseconds. |
| `SAYA_OUTPUT_FORMAT` | `text` or `json`. |
| `SAYA_NON_INTERACTIVE` | `1` / `true` or `0` / `false`. |
| `SAYA_DEBUG` | `1` / `true` enables debug logging. |
| `SAYA_CONFIG_DIR` | Override config directory. |
| `SAYA_CONVERSATION_ID` | Default conversation id. |
| `SAYA_SESSION_ID`, `SAYA_TENANT_ID`, `SAYA_ACTOR_ID`, `SAYA_CHANNEL_ID` | Context fields. |
| `SAYA_STREAM_MAX_RETRIES` | Extra stream reopen attempts. |
| `SAYA_STREAM_RETRY_INITIAL_MS` | Initial backoff (ms). |
| `SAYA_STREAM_RETRY_MAX_MS` | Backoff cap (ms). |

## Configuration directory

Config and credentials live only under standard OS paths (or `SAYA_CONFIG_DIR`), never implicitly in the current working directory.

- Linux / macOS: `~/.config/saya/`
- Windows: `%APPDATA%\saya\`

Sensitive values are not printed in debug logs.

## Auth and session

- `chat` expects a **Bearer** access token stored via the credentials file (see project tooling / docs for how you set the token in your setup).
- Without a token, `chat` exits with an error.
- If `conversation_id` is omitted, the client creates a conversation (`POST /v1/conversations`) and persists the new id for the next run.
- `channel_id` defaults to `terminal` unless overridden.

## Streaming

The `chat` command calls:

`POST /v1/conversations/{conversation_id}/messages/stream`

with the same JSON body shape as the non-stream message endpoint. The response is `text/event-stream`.

### Event types (wire format)

Each SSE event uses an `event:` line plus one or more `data:` lines (JSON payload). Event names and JSON fields match **soz-saya** (`src/schemas.ts`, `StreamEventSchema`, and `buildStreamJsonSchema` in `src/contracts.ts`). Supported event names:

- `token` — assistant text chunks (`text`, `seq`).
- `emotion` — avatar / UX state (must not be mixed into assistant text).
- `status` — lifecycle signals (e.g. stalls, cancellation).
- `tool` — tool execution metadata.
- `error` — terminal error for the stream (`code`, `message`, `seq`).
- `done` — normal completion (`seq`).

The CLI accumulates `token` payloads into the final reply. Other events are parsed for correctness; in `--debug` mode, summaries are logged to the debug sink.

### Retries and idempotency

On network I/O errors or an incomplete stream (no `done`), the client may repeat the **same** `POST` up to `--stream-max-retries` times with exponential backoff. Each retry is a **new** request to the server. If the server does not deduplicate messages, you may see duplicated assistant output or side effects—prefer stable networking or server-side idempotency keys when available.

### Interrupts and terminal state

- **Ctrl+C** stops reading the stream, clears the progress spinner when used, prints a trailing newline when text was already written, and exits without leaving the cursor hidden.
- A **panic hook** and a drop guard restore the cursor and SGR state to reduce “broken” terminals after failures.

### JSON output

With `--output json`, token text is **not** streamed to stdout during the request. After the stream finishes (success, interrupt, or retry exhaustion), the client prints a single JSON object, for example:

- `command`: `"chat"`
- `ok`: `true` if the stream completed without a client-side warning
- `conversation_id`: string
- `text`: full concatenation of `token` events
- `warning`: optional human-readable message (interrupt, incomplete stream, retries exhausted, etc.)

## Troubleshooting

| Symptom | Check |
|---------|--------|
| `401` on chat | Token missing or expired; refresh credentials. |
| `404` on stream | Wrong `conversation_id` or context mismatch vs. conversation. |
| `429` | Server concurrency limits; retry later per server `Retry-After` if present. |
| Truncated reply + `[saya] stream closed before done` | Network drop or proxy idle timeout; adjust retries or server timeouts. |
| Garbled terminal after a crash | Rare; the panic hook should restore the cursor—if not, run `reset` in the shell. |

## Contract source of truth

Do not invent stream fields in this crate: when **soz-saya** changes `StreamEventSchema`, update `src/stream_contract.rs` and SSE tests/fixtures accordingly.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
