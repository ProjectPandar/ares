#!/usr/bin/env bash
set -euo pipefail
export ARES_ROLE_ORACLE_MODE="$5"
export ARES_ROLE_ORACLE_INPUT="$1"
export ARES_ROLE_ORACLE_OUTPUT="$2"
exec "$4" -nx -q -batch -x "$3"
