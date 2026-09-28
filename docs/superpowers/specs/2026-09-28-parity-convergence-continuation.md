# Spec: parity convergence continuation — option coverage + printer smoke residuals

## Status: ACTIVE (authored 2026-09-28)

This spec continues the bounded convergence wave defined by
`2026-09-08-parity-convergence.md` and `2026-09-26-parity-residuals-milestone.md`
on the same fixed upstream baseline: OrcaSlicer `v2.4.2`,
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, oracle executable
`/tmp/orca-official-242/squashfs-root/bin/orca-slicer` driven through
`scripts/orca-parity.sh` (`ORCA_APPDIR=/tmp/orca-official-242/squashfs-root`).

## Scope

1. **Option coverage completion.** Every bounded domain in
   `tests/ksr_fdmtest_v4/options-v242.json` (650 entries, `Metadata`
   excluded) must produce compared output. Bool/enum domains enumerate every
   declared value; numeric ranges probe `min`, `max`, and a seeded interior;
   `coFloatOrPercent` adds the absolute-literal branch; schema-unbounded
   strings/points receive authored legal probes; id-valued/machine-internal
   keys that the oracle rejects stay baseline-only and are reported as
   `INCOMPLETE` with an honest reason — never suppressed.

2. **Printer smoke residual closure.** The strict ordered-byte sweep over
   all 1,001 vendor machine presets must reach every reachable `PASS`.
   `ORCA_ERROR` (the upstream binary itself fails) and `VENDOR_INCOMPLETE`
   (the vendor tree ships no usable default process) are reported, not
   counted as Ares failures. Remaining `DIVERGENT`/`ARES_ERROR` rows are
   each fixed as a source-cited slice.

3. **Harness hygiene.** No source-line pinning tests; test-only modules
   compare observable output only. Source files stay under 400 LOC; tests
   live in `tests.rs`-style sibling modules; no `include!`/`include_bytes!`
   for source splitting.

## Current non-green facts

- Option sweep (`ARES_PARITY_OPTIONS=1` merge run 2026-09-28): 608/650
  bounded domains exercised; residual INCOMPLETEs are authored-interior
  samples (upstream slice-time validation rejects the schema bound) or
  id-valued/unprovable domains, all with `compared ≥ 1` or documented
  oracle rejection.
- Printer sweep last full run: 958/1002 PASS; 15 DIVERGENT, 18 ORCA_ERROR,
  10 VENDOR_INCOMPLETE, 1 ARES_ERROR (vendor-load probe). After the
  pending-lift raised-Z fix and the wipe-before-external `next`-vertex fix,
  Artillery M1 Pro, Qidi Q2/Q2C, Eryone ER20 (0.4) and Kingroon KP3S verify
  PASS under `ARES_RACE_TARGET` — a refreshed sweep regenerates the
  summary's denominators (the OrcaFilamentLibrary row is gone now that
  vendor discovery requires a `machine/` dir).
- Divergence clusters observed: M73 `R` minute-boundary rounding and
  cache-entry mapping (Sermoon V1, ER20, Ginger G1, SK1, Volumic x3,
  TinyBotV1); last-digit f32 coordinate/E rounding (Kobra 3 0.2, E3NG,
  Rolohaun, SV06); travel-point ordering (PhoenixProV1, TiQ8).

## Independent-review fix loop (round 1)

- KSR semantic model now keeps `M73` progress lines and the
  `CONFIG_BLOCK`; the config block compares exactly while the progress
  lines remain diagnostic (the estimate-machine residual is the M73
  cluster below).
- Every option case verifies requested-vs-effective exported values
  (width domains previously were the only ones checked —
  `printer_technology/SLA` had passed while exporting `FFF`).
- Upstream `this->add` keys reconcile against the inventory through
  `tests/parity/upstream-key-exclusions.txt`.
- Upper-only numeric bounds probe the declared ceiling (`nozzle_diameter`
  max=100); bool values rejected by oracle-side fixture conflicts now
  co-inject the dependent option (`use_firmware_retraction`→`wipe=0`,
  `use_relative_e_distances`→G92-free layer gcodes).
- Excused oracle-nondeterministic cases report as evidenced INCOMPLETE;
  sweep acceptance gates only genuine failures; a failed oracle retry no
  longer overwrites an established divergence; `OrcaFilamentLibrary` no
  longer manufactures a printer row; `m73_profile` diagnostics reproduce
  the upstream f32 arithmetic and (P, desc R) merge order.

## Acceptance

- `ARES_PARITY_OPTIONS=1` full run: zero unexempted non-PASS domains.
- `ARES_PARITY_SWEEP=1` full run: every DIVERGENT row resolved to PASS or
  reclassified ORCA_ERROR/VENDOR_INCOMPLETE with recorded evidence.
- `cargo nextest run --workspace`, `cargo fmt --check`, warning-denying
  `cargo clippy`, WASM checks green; no file over 400 LOC.
