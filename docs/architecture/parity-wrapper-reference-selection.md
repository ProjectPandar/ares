# Parity wrapper reference selection

## Boundary

This repair is limited to the native parity adapter in `scripts/orca-parity.sh`
and its subprocess tests in `ares-cli`. It does not change any slicer behavior.

## Decision

The wrapper must reject a missing explicit reference before accessing its library
cache or discovering libraries. An explicitly supplied `ARES_PARITY_ENV` may
provide `ORCA_APPDIR`; no implicit reference discovery is permitted.

The rejection diagnostic must retain the version-independent phrase
`ORCA_APPDIR must name an explicitly selected reference`. The no-reference test
must remove both reference environment inputs from its child process. A selected
reference must continue to receive the exact argument vector, including spaces.

Library resolution, job-local interpreter patching, and execution remain unchanged.
