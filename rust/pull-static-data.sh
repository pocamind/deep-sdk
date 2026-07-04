#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p assets
curl -fsSL "https://github.com/pocamind/data/releases/latest/download/all.json" -o assets/all.json
echo "wrote assets/all.json ($(wc -c < assets/all.json) bytes)"
