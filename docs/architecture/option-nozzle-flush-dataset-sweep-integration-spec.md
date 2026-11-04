# Spec: bounded nozzle_flush_dataset sweep integration

## Decision requested

Claim `nozzle_flush_dataset` value-bearing option-coverage integration only.
Existing `runner/x2d_flush_matrix.rs` coverage is retained, not declared missing.

## Source-owned boundary

- `libslic3r/PrintConfig.cpp:2602` and `PrintConfig.hpp:1350`: option definition
  and nullable printer-axis storage.
- `libslic3r/FlushVolPredictor.cpp:312-327`, `GenericFlushPredictor`: dataset
  selection for predictor resources.
- `libslic3r/FlushVolCalc.cpp:43-62,102-124`, `FlushVolCalculator`: selected
  predictor versus analytic calculation and dataset-specific colour behavior.
- `OrcaSlicer.cpp:3436-3463,3487`: upstream CLI admission/variant selection
  passes the selected physical-nozzle dataset into the libslic3r calculator.
  This adapter is evidence context, not a proposed independent slicing owner.

Rust destinations already exist: `ares-core/src/options/flush_matrix.rs` and
`options/project_variants/printer.rs`. They are compatibility shells around
these upstream concepts. This request adds CLI E2E/sweep evidence, not a new
Ares predictor, flush scheduler, tower pipeline, or production redesign.

## Included and deferred

Include a bounded existing X2D two-logical-filament input, fixed colours,
physical identities and selected printer variant. Keep the matrix absent to
exercise upstream rebuild. Compare coherent all-slot datasets 0, 1 and 2;
retain the existing mixed `[1,2,2,1]` case as an additional selection witness.
Add two controls with identical explicit valid matrix and datasets 1 versus 2;
prove whether upstream retains the matrix or rebuilds it before freezing the
control expectation. Do not assume total G-code equality from an inactive
calculator; config serialization still changes.

Defer malformed IDs, resource lookup errors, exhaustive RGB/RGBA grids,
support/long-retraction changes, arbitrary variant cardinalities, SEMM,
WipeTower2 production, and wider printer-family behavior. Existing tests for
those adjacent cases remain unchanged.

## Test-first admission and acceptance

1. Audit the exact existing fixture and reconstruction trigger before defining
   labels/inputs; publish member/settings input-diff proofs.
2. Run fresh live Native admission for the six bounded inputs. Require real
   output and source-attributable matrix values, not only serialized key text.
   Establish active dataset differences from Native; if a pair has no effect,
   report it and revise the spec before accepting a weak witness.
3. Write CLI E2Es before any implementation. For every admitted case compare
   the entire validated API output and real CLI output against live Native
   using the unchanged strict comparator and wall-clock exemptions.
4. Register authored value-bearing plans and bounded sweep dispatch for this
   key. Do not modify the existing `x2d_flush_matrix` assertions.
5. Run both modes with fresh caches, strict Clippy, fmt and wasm32.

Direct GREEN is valid. Any production mismatch first stops for an operation-
level source-cited report; this request does not repair the tower/flush
production. Preserve all failed Native admissions, setup errors and
interrupted/VOID runs.
