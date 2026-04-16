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
- `saya health --base-url <url>`
- `saya chat --message "..." --base-url <url>`
