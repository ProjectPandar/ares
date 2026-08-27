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
