#!/usr/bin/env bash
set -euo pipefail
export ARES_ROLE_ORACLE_MODE=process
export ARES_ROLE_ORACLE_INPUT="$1"
export ARES_ROLE_ORACLE_OUTPUT="$2"
exec nix shell nixpkgs#gdb -c gdb -q -batch -x "$3"
