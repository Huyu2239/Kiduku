#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <query> [limit]" >&2
  exit 1
fi

query="$1"
limit="${2:-20}"

echo "[info] Searching similar issues: query='${query}', limit=${limit}" >&2
gh issue list --state all --limit "${limit}" --search "${query} in:title,body"
