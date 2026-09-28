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
  10 VENDOR_INCOMPLETE, 1 ARES_ERROR (vendor-load probe).
- Divergence clusters observed: M73 `R` minute-boundary rounding and
  cache-entry mapping (Sermoon V1, ER20, Ginger G1, KP3S, SK1, Volumic x3,
  TinyBotV1); last-digit f32 coordinate/E rounding (Kobra 3 0.2, E3NG,
  Rolohaun, SV06); travel-point ordering (PhoenixProV1, TiQ8).

## Acceptance

- `ARES_PARITY_OPTIONS=1` full run: zero unexempted non-PASS domains.
- `ARES_PARITY_SWEEP=1` full run: every DIVERGENT row resolved to PASS or
  reclassified ORCA_ERROR/VENDOR_INCOMPLETE with recorded evidence.
- `cargo nextest run --workspace`, `cargo fmt --check`, warning-denying
  `cargo clippy`, WASM checks green; no file over 400 LOC.
