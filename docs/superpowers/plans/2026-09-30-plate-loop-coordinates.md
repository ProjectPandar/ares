# Plate-loop coordinate implementation plan

The [spec](../specs/2026-09-30-plate-loop-coordinates.md) identifies the
upstream origin-zero loop emission and the Rust destination.

1. Reproduce the real Sovol output difference and inspect pre-round values.
2. Add an emitter regression for plate translation and extruder offsets.
3. Use the clipped integer endpoints for plate-loop G-code positions,
   retaining the current local-state representation for other consumers.
4. Run complete-output Sovol and related printer comparisons, Rust checks,
   and independent review. Commit code, evidence summary, spec and plan
   together; push after review.

## Verification evidence

- Real Sovol red: `/tmp/ares-sovol-round-red.log`, three reference attempts
  show the same Y98.188/Y98.189 difference. Emitter red:
  `/tmp/ares-plate-origin-red.log`.
- Sovol green: `/tmp/ares-sovol-round-green/case-tyvTcQ`; RH3D E3NG
  0.2 nozzle green: `/tmp/ares-plate-origin-rh3d/case-gH510D`.
  Each directory retains the input, complete paired G-code and manifest.
  Only generator identity/timestamp normalization is permitted.
- Rolohaun and Anycubic remain divergent; evidence is retained under
  `/tmp/ares-plate-origin-rolohaun` and `/tmp/ares-plate-origin-kobra`.
  The historical modified-profile matrix now has 973/1001 passing rows;
  this targeted update is not a new all-printer/default-profile sweep.
- Independent initial review found no logic repair items and requested
  first-travel and local-wipe regression assertions. Both were added,
  covering Skirt/Brim with zero and nonzero extruder offsets.
- The pre-existing uncommitted geometry refactor is present during
  validation but is not part of this slice or commit.
- `cargo fmt --check`, `git diff --check`, and `cargo clippy` passed;
  Clippy retains 227 pre-existing core warnings.
- WASM checks for core/bindings and core checks for Windows GNU and macOS
  passed. These establish compilation, not browser/desktop runtime parity;
  the real-Orca runs above are Linux evidence.
- `cargo nextest run --workspace`: 7357 passed, 9 skipped, recorded in
  `/tmp/ares-plate-origin-workspace.log`. The subsequently strengthened
  regression also passed in `/tmp/ares-plate-origin-focused-final.log`;
  production behavior was unchanged between those runs.
- Final independent six-aspect review passed with no remaining repair
  items; the initial coverage suggestions are closed.
