#!/usr/bin/env bash
# 本機預覽：scripts/serve.sh [port]
set -euo pipefail
PORT=${1:-8088}
cd "$(dirname "$0")/../codelabs/generated"
echo "http://localhost:$PORT"
python3 -m http.server "$PORT"
