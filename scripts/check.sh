#!/usr/bin/env bash

set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

command -v jq >/dev/null || {
  echo "check: jq is required" >&2
  exit 1
}

while IFS= read -r schema; do
  jq -e . "$schema" >/dev/null
done < <(rg --files "$project_root/spec" -g '*.json' | sort)

while IFS= read -r message; do
  [[ -z "$message" ]] || jq -e . >/dev/null <<<"$message"
done < "$project_root/examples/agent-session.jsonl"

link_errors=0
while IFS=: read -r source line match; do
  target="${match#']('}"
  [[ "$target" =~ ^[a-z][a-z0-9+.-]*: ]] && continue

  resolved="$(dirname "$source")/$target"
  if [[ ! -e "$resolved" ]]; then
    echo "check: broken link at $source:$line -> $target" >&2
    link_errors=1
  fi
done < <(rg --no-heading --line-number --only-matching '\]\([^)#]+' "$project_root" --glob '*.md')

if [[ "$link_errors" -ne 0 ]]; then
  exit 1
fi

echo "check: JSON, JSONL, and local Markdown links are valid"

