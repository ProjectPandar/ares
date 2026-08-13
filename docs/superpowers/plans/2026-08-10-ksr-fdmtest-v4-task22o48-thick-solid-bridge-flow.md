# Task 22O.48 thick solid-infill bridge Flow implementation plan

## Status

Complete. All runtime/static gates pass, and the independent six-axis
repair/re-review loop ends in unconditional approval.

## Objective

Implement the exact typed `LayerRegion::bridging_flow(frSolidInfill, true)`
dependency needed by the pinned bridge-over-infill transaction, including
`Config.hpp:624-628,1284-1286` vector fallback and float-or-percent evaluation.

## Plan

1. **RED focused tests**
   - Register `project_slice/tests/perimeters/thick_bridge_flow.rs`.
   - Freeze exact default, width form, selector/fallback, ratio cast, spacing,
     volume, invalid-input, repeatability, and nonmutation behavior.
   - Run `cargo nextest run -p ares-core -E 'test(/task22o48/)'` and retain the
     unresolved resolver RED.

2. **Implement by deepening the existing Flow module**
   - Add `resolve_thick_solid_infill_bridge_flow` to
     `project_slice/perimeters/flow.rs`.
   - Factor the existing thick overhang branch through one private helper that
     accepts the source role selector.
   - Reuse `selected_nozzle`, `absolute_f64`, `bridge_volume`, and
     `require_positive_volume`; add no new Flow type or fallback.
   - Keep `flow.rs` below 400 physical lines.

3. **Compose the real KSR dependency**
   - Replace O47's integration-test manual bridge-height reconstruction with
     the production O48 resolver.
   - Assert exact Flow fields and preserve the existing 18-layer flat geometry
     totals and ordered digest.

4. **Document and verify**
   - Update ADR/spec/plan, roadmap, and option parity with actual evidence.
   - Run focused, dependency, workspace, rustfmt, warning-denying Clippy,
     wasm32, diff/LOC/static gates.
   - Start an independent read-only six-axis review. Main-thread fixes are
     re-run and re-reviewed until literal approval.

## Exit criteria

- The production resolver matches the cited source specialization and consumes
  only typed embedded options.
- Existing overhang Flow behavior remains green.
- O47's KSR geometry is unchanged when driven by O48.
- No lifecycle activation or G-code claim is introduced.
- All gates and independent review approve.
