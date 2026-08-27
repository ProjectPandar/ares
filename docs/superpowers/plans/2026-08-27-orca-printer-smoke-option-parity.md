# Plan: OrcaSlicer printer smoke and option coverage parity

## Package P1 — loader compatibility with CLI-exported 3MF

1. Commit a captured OrcaSlicer CLI `--export-3mf` project under
   `tests/parity/` (no thumbnail parts, relationships reference
   `Metadata/plate_1.png`); add a red `slice_project` test asserting the load
   succeeds and emits G-code.
2. Relax `project/load.rs` `validate_root_previews`/`validate_plate_previews`
   to follow OrcaSlicer: a referenced preview part that is absent from the
   archive is skipped; present parts keep PNG content-type validation.
3. rustfmt + clippy + full nextest; commit and push.

## Package P2 — parity fixture suite and single-printer smoke

1. Add `tests/parity/cube10.stl` (binary STL cube) as the harness model.
2. Add `crates/ares-cli/tests/orca_parity.rs` (env-gated on `ARES_ORCA_BIN`
   and `OrcaSlicer/resources/profiles`): flatten a vendor preset chain in
   Rust, invoke the OrcaSlicer CLI to `--export-3mf` + `--slice 0`, slice the
   same 3MF through `ares_core::slice_project`, and run the existing semantic
   comparison. Split into `mod`s under 400 LOC each.
3. Land the Ender-3 case green (or with its first divergence recorded);
   commit and push.

## Package P3 — vendor-wide printer smoke sweep

1. Enumerate every FDM machine preset across vendors; map each to its default
   process/filament presets (falling back to vendor-generic presets when the
   machine names none).
2. Run the Package P2 seam for every printer with a bounded concurrency
   pool; cache Orca reference output under `target/parity-cache/`.
3. Commit a tracked inventory (`tests/parity/printer-smoke-summary.md`) of
   pass/fail per printer with the first semantic divergence for failures.
4. Fix divergences in source-cited slices, smallest common cause first;
   commit and push each.

## Package P4 — option coverage sweep

1. Drive coverage from `tests/ksr_fdmtest_v4/options-v242.json`: derive
   value domains (bool ×2, enum ×N, range min/max/seeded interior) from the
   option registry definitions and upstream PrintConfig min/max.
2. Apply each override to the owning flattened preset (process default,
   machine/printer-owned keys per `s_Preset_printer_options`, filament for
   `raw_scope: filament`), rebuild the 3MF through the OrcaSlicer CLI, and
   compare both slicer outputs with the semantic comparator.
3. Commit a tracked inventory (`tests/parity/option-coverage-summary.md`)
   listing each option, tested values, and pass/fail with first divergence.
4. Fix divergences in source-cited slices; commit and push each.

## Package P5 — cleanup and audit

1. Delete remaining OrcaSlicer source-level pinning tests whose observable
   behavior is covered by parity tests.
2. Audit all touched files below 400 LOC; split with `mod`s only.
3. Full `cargo nextest run --workspace`, clippy, fmt; commit and push.

## Package P6 — independent review loop

1. Launch a read-only reviewer subagent verifying requirement completeness,
   logic correctness, boundary conditions, code quality, test coverage, and
   actual run results.
2. Fix findings on the main thread; re-run the reviewer until it passes or
   the remaining blockers are stated explicitly.

## Divergence queue (Ender-3 smoke evidence, 2026-08-27)

Live tracker: `orca_parity_ender3_smoke` (fails while open; skips in CI).
Comparator: `semantic::compare_ignoring_time` — timing excluded until the
GCodeProcessor motion planner port.

1. Skirt/brim generation is not implemented (`footprint.rs` only uses skirt
   config for bounds). Orca emits `;TYPE:Skirt` for 2 loops
   (Print.cpp / GCode.cpp `_print_skirt`). ~18mm of E on the cube.
2. Lift merging: Ares emits standalone `G1 Z{..} F9000` after wipe;
   upstream merges lift into the next XY travel
   (`G1 X.. Y.. Z2`) — GCodeWriter travel semantics.
3. Object-comment ordering at object end differs between BBL golden
   (stop comment → retract/wipe, pinned by KSR X2D fixture) and non-BBL
   Orca reference (retract/wipe → stop comment). Needs the upstream
   GCodeWriter object_start_str/object_end_str flush mechanics.
4. Travel feedrate rounding `G1 F894` vs `F908` — likely upstream float
   flow/speed computation differences in feature entry feedrates.
5. `M204 S500/S0` pair emitted by Ares around Sparse infill but not by
   upstream (acceleration-marker gating per feature).
6. Model printing time difference (Ares 1540s vs Orca 535s) — blocked on
   full acceleration-planner parity; soft metric meanwhile.
7. Arachne wall generator unported; classic baseline pinned via overrides,
   plus detect_thin_wall=0.
