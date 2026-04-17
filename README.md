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
| `saya chat --allow-restricted-tools` | Opt-in for soz-saya restricted tools without a prompt (see **Restricted tools**). |

Use `saya --help` and `saya chat --help` for flag details.

## Global flags

| Flag | Meaning |
|------|---------|
| `--base-url <url>` | soz-saya base URL. |
| `--timeout-ms <n>` | Connect / blocking HTTP timeout for non-stream calls; stream uses this as **connect** timeout. |
| `--output <text\|json>` | Text: stream tokens to stdout. JSON: buffer tokens and print one JSON object at the end. |
| `--non-interactive` | No interactive prompts; restricted tools stay denied unless you set env/flag (see **Restricted tools**). |
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
| `SAYA_ALLOW_RESTRICTED_TOOLS` | `1` / `true`: send `allow_restricted_tools: true` on chat requests (automation opt-in; same risk as `--allow-restricted-tools`). |

Optional in `config.json` (same merge order as other file fields): `allow_restricted_tools`: boolean — when `true`, equivalent to the env flag above.

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

## Restricted tools

soz-saya may call **restricted** (write-like) tools only if the message body includes `allow_restricted_tools: true` (see `MessageRequestSchema` in soz-saya). By default the CLI does **not** send that.

| Situation | Behavior |
|-----------|----------|
| `--non-interactive` or stdin is not a TTY | Field omitted (server treats as deny). |
| Interactive TTY | Prompt once per `chat`: allow restricted tools or not. |
| `--allow-restricted-tools` on `saya chat` | Sends `true` without prompting (use only if you accept the risk). |
| `SAYA_ALLOW_RESTRICTED_TOOLS=true` or `allow_restricted_tools: true` in config | Same as the flag (for scripts/CI). |

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

The CLI accumulates `token` payloads into the final reply (stdout in text mode, or the `text` field in JSON output).

### Emotion and status (stderr)

`emotion` and `status` events are **not** appended to the assistant text. They are shown on a separate UX layer: short lines on **stderr** with prefixes `[saya:emotion]` and `[saya:status]` (muted gray). Semantic emotions carry `priority` and `ttl_ms` from the server: a new emotion replaces the active one if its **priority is strictly higher**, or if the previous emotion’s TTL has **expired**. When soz-saya adds a `local` emotion source to the schema, this crate should follow that spec and be updated together with [`src/stream_contract.rs`](src/stream_contract.rs).

`tool` events still appear in `--debug` diagnostics only (no second stdout stream).

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

Do not invent stream fields in this crate: when **soz-saya** changes `StreamEventSchema`, update `src/stream_contract.rs`, SSE tests/fixtures, and the golden schema under `schemas/stream_events.schema.json`.

### CI and schema drift

GitHub Actions runs `cargo fmt`, `clippy` (`-D warnings`), tests, [`.ci/check-boundaries.sh`](.ci/check-boundaries.sh) (forbidden deps/patterns), and [`.ci/check-stream-contract.sh`](.ci/check-stream-contract.sh).

The contract script always validates that `schemas/stream_events.schema.json` is valid JSON. If you set **`SOZ_SAYA_ROOT`** to a checkout of **soz-saya** (same machine as the CLI repo), it also diffs the golden file against `buildStreamJsonSchema()` from that tree and fails on mismatch.

To refresh the golden file after a server-side schema change:

```bash
# from soz-saya (Node 20+)
npx tsx scripts/export-stream-schema.ts > ../saya-cli/schemas/stream_events.schema.json
```

(or the equivalent `npx tsx -e "import { buildStreamJsonSchema } from './src/contracts.ts'; ..."`).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
