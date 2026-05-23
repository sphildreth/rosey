#!/usr/bin/env bash
set -euo pipefail

title="${1:-}"
if [[ -z "$title" ]]; then
  echo "Usage: $0 \"ADR title\"" >&2
  exit 1
fi

dir="design/adr"
mkdir -p "$dir"

next="$(
  find "$dir" -maxdepth 1 -type f -name '[0-9][0-9][0-9][0-9]-*.md' \
    | sed -E 's#.*/([0-9]{4})-.*#\1#' \
    | sort -n \
    | tail -1
)"

if [[ -z "$next" ]]; then
  num="0001"
else
  num="$(printf '%04d' "$((10#$next + 1))")"
fi

slug="$(echo "$title" \
  | tr '[:upper:]' '[:lower:]' \
  | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g')"

file="$dir/${num}-${slug}.md"

cat > "$file" <<EOF
# ADR-${num}: ${title}

Status: Proposed  
Date: $(date +%F)

## Context

## Decision

## Consequences

## Alternatives considered

EOF

echo "$file"
