#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[DoD0] Checking forbidden dependency names in Cargo.toml"
forbidden_deps_regex='openai|anthropic|langchain|llm|tiktoken|qdrant|weaviate|pinecone|milvus|lancedb|candle|ollama|vllm'
if rg -n -i "${forbidden_deps_regex}" Cargo.toml >/dev/null; then
  echo "Found forbidden dependency token in Cargo.toml"
  rg -n -i "${forbidden_deps_regex}" Cargo.toml
  exit 1
fi

echo "[DoD0] Checking forbidden runtime/provider patterns in src"
forbidden_code_regex='api\.openai\.com|api\.anthropic\.com|router|rag|vector|embedding|langchain|tool_orchestration|agent_runtime'
if rg -n -i "${forbidden_code_regex}" src >/dev/null; then
  echo "Found forbidden code pattern in src/"
  rg -n -i "${forbidden_code_regex}" src
  exit 1
fi

echo "[DoD0] Ensuring code references soz-saya endpoint semantics"
if ! rg -n "soz-saya|/health|/messages|/stream" src >/dev/null; then
  echo "No saya transport endpoint references found in src/"
  exit 1
fi

echo "[DoD0] Boundary checks passed"
