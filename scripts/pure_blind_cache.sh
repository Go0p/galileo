#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"

DEFAULT_OUTPUT="$ROOT_DIR/cache/pure_markets.json"
DEFAULT_URL="${PURE_BLIND_MARKET_URL:-https://cache.jup.ag/markets?v=4}"

OUTPUT_PATH="${1:-$DEFAULT_OUTPUT}"
DOWNLOAD_URL="${2:-$DEFAULT_URL}"

mkdir -p "$(dirname "$OUTPUT_PATH")"

echo "[pure-blind-cache] downloading markets.json" >&2
echo "  url   : $DOWNLOAD_URL" >&2
echo "  output: $OUTPUT_PATH" >&2

if command -v curl >/dev/null 2>&1; then
  curl -sSfL "$DOWNLOAD_URL" -o "$OUTPUT_PATH"
elif command -v wget >/dev/null 2>&1; then
  wget -q -O "$OUTPUT_PATH" "$DOWNLOAD_URL"
else
  echo "error: curl 或 wget 未安装，无法下载 markets.json" >&2
  exit 1
fi

if command -v python3 >/dev/null 2>&1; then
  python3 - <<'PY' "$OUTPUT_PATH"
import json, sys
path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fp:
    data = json.load(fp)
print(f"[pure-blind-cache] cached {len(data)} markets @ {path}")
PY
else
  echo "[pure-blind-cache] 下载完成，可使用 python3/jq 查看详情" >&2
fi

echo "[pure-blind-cache] done" >&2
