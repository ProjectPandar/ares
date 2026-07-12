#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -eq 0 ]; then
  exec cargo nextest run --workspace
fi

exec cargo nextest run "$@"
