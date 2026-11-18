#!/usr/bin/env bash
# 檢查教學內容 → 匯出 Codelabs → 套用主題 → 產生首頁。
set -euo pipefail
cd "$(dirname "$0")/.."

CLAAT=${CLAAT:-$(command -v claat || echo "$HOME/go/bin/claat")}

python3 codelabs/lint_callouts.py codelabs/tutorials/*.md
python3 scripts/check_snippets.py codelabs/tutorials/*.md

rm -rf codelabs/generated
mkdir -p codelabs/generated
# -prefix ".." 讓頁面引用本機 claat-public，避免 Google CDN 403 並讓主題生效。
(cd codelabs && "$CLAAT" export -prefix ".." -o generated tutorials/*.md)
cp -r codelabs/claat-public codelabs/generated/claat-public
python3 codelabs/apply_devsite_theme.py
cp codelabs/index.html codelabs/generated/index.html
echo "完成：codelabs/generated/（執行 scripts/serve.sh 預覽）"
