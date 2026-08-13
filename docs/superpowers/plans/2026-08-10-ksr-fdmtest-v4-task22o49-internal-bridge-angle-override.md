# Task 22O.49 internal bridge angle override implementation plan

## Status

Complete. Every runtime/static gate passes, and the independent six-axis
repair/re-review loop ends in unconditional approval.

## Objective

Implement the exact private override at pinned `PrintObject.cpp:3253-3267`
using only typed embedded region options and Ares' already-resolved object
rotation.

## Plan

1. **Review the boundary**
   - Compare ADR/spec with `PrintObject.cpp`, `Geometry.hpp`,
     `libslic3r.h:71::PI`, typed region
     fields, and `perimeters/context.rs` transform rotation.
   - Record O43/perimeter records as reused upstream-aligned seams and the
     legacy `InfillOptions` angle field as a non-destination compatibility
     shell with no fallback or duplicate activation.
   - Keep automatic direction and bridge transaction activation deferred.

2. **RED focused tests**
   - Register `internal_bridge_angle.rs` and nested `tests.rs`.
   - Freeze pass-through bits, exact degree-to-radian operation order,
     relative/absolute/alignment ownership, no normalization, nonfinite
     arithmetic, exact non-NaN/pass-through repeatability, classification-only
     arithmetic-NaN repeatability, and complete option nonmutation.
   - Add a compiling `todo!()` operation stub, run
     `cargo nextest run -p ares-core -E 'test(/task22o49/)'`, and retain the
     behavioral RED.

3. **Minimal implementation**
   - Implement one crate-private borrowed function with the source branch
     structure and exact f64 operation order.
   - Add no validation, error type, helper abstraction, fallback, stage, or
     lifecycle successor.

4. **Real KSR composition**
   - Prepare O43 from `KsrArchive` and recover each candidate's region options
     and `PerimeterInputRecord::model_rotation_rad` from the typed predecessor.
   - Freeze default ordered output bits and complete input preservation.
   - Prepare two separately mutated archives before O43: both set a positive
     override, enable alignment, and replace the actual occurrence transform
     with first XY column `(0,1)`; one absolute case must add the retained pi/2
     rotation, while one relative case must ignore that nonzero rotation.
   - Use existing semantic archive replacement helpers and freeze option,
     transform, rotation, ordered-output, repeatability, and nonmutation bits,
     proving provenance rather than fixture identity.

5. **Document and verify**
   - Update ADR/spec/plan, roadmap, and option parity with actual evidence.
   - Run focused, O43-O49 dependency, workspace, rustfmt, warning-denying
     Clippy, wasm32, diff/LOC/static gates.
   - Run an independent read-only six-axis review; main-thread fixes and fresh
     re-review repeat until unconditional approval.

## Exit criteria

- Exact source arithmetic/branch behavior is source-discriminated.
- KSR inputs come only from embedded 3MF and existing typed stages.
- Public slicing remains terminal at O43.
- Every changed Rust file is below 400 LOC and uses no source-splitting include
  macro.
- All gates and independent review approve.
