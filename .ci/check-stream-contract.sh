#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

GOLDEN="schemas/stream_events.schema.json"
if [[ ! -f "$GOLDEN" ]]; then
  echo "Missing $GOLDEN"
  exit 1
fi

echo "[contract] parsing $GOLDEN"
python3 -c "import json; json.load(open('$GOLDEN'))"

SOZ_SAYA_ROOT="${SOZ_SAYA_ROOT:-}"
if [[ -n "$SOZ_SAYA_ROOT" && -d "$SOZ_SAYA_ROOT" ]]; then
  echo "[contract] comparing live export from SOZ_SAYA_ROOT=$SOZ_SAYA_ROOT"
  live="$(mktemp)"
  (
    cd "$SOZ_SAYA_ROOT"
    npx --yes tsx -e "import { buildStreamJsonSchema } from './src/contracts.ts'; process.stdout.write(JSON.stringify(buildStreamJsonSchema(), null, 2))"
  ) >"$live"
  if ! diff -u "$GOLDEN" "$live"; then
    echo "Stream JSON schema drift. Regenerate: (cd soz-saya && npx tsx -e \"import { buildStreamJsonSchema } from './src/contracts.ts'; process.stdout.write(JSON.stringify(buildStreamJsonSchema(), null, 2))\") > saya-cli/schemas/stream_events.schema.json"
    rm -f "$live"
    exit 1
  fi
  rm -f "$live"
else
  echo "[contract] SOZ_SAYA_ROOT not set; skipped live diff (golden file is still enforced in-repo)"
fi

echo "[contract] stream schema check passed"
