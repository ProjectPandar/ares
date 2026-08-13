# Task 22O.51 automatic bridge-angle implementation plan

## Status

Implemented and complete. Independent source/specification review preceded RED; final independent implementation re-review approved unconditionally.

## Objective

Implement the exact automatic angle vote in pinned
`PrintObject.cpp:2849-2932::determine_bridging_angle` on O50, without yet
constructing its transaction inputs or changing O43 lifecycle output.

## Plan

1. **Freeze the source boundary**
   - Independently review ADR/spec against pinned `PrintObject.cpp`,
     `AABBTreeLines.hpp`, `Line.cpp`/`Line.hpp`, `Point.hpp::scaled`, pinned
     Eigen `Core/Dot.h`, `libslic3r.h`, and `PrintConfig.hpp`.
   - Build a temporary standalone C++ oracle from the actual pinned source
     dependencies. Freeze sampled points, exact angle keys/counts, reduction
     results, pattern outputs, per-polygon reset, integer Normal/LargeBed
     thresholds, normalization division order, boundary/wrap/tie cases, and
     finite synthetic upper-wrap reducer inputs. Restore
     the source checkout and retain no runtime pinning.

2. **Behavioral RED**
   - Register `automatic_bridge_angle.rs` and an ordinary nested test module.
   - Add the specified compiling API with a `todo!()` body plus test-only
     read-only sample/reduction seams.
   - Add literal source-shaped tests covering sampling arithmetic, ordered vote
     reduction, pattern adjustments, repeatability, and nonmutation.
   - Run focused Nextest and retain the expected RED.

3. **Minimal implementation**
   - Reuse O50 and existing `Line::orientation`.
   - Implement ordered exact-key insertion without host sort/hash maps, the
     source sampling cast order, closed windows, wrap adjustments, strict score
     replacement, fallback, and exhaustive typed pattern adjustment.
   - Keep the operation crate-private, borrowed, unwired, and below 400 LOC per
     source file.

4. **Verify and review**
   - Run focused, O43-O51/geometry dependency, workspace, rustfmt,
     warning-denying Clippy, core/browser wasm32, diff/LOC/static audits,
     explicitly banning `include!`, `include_bytes!`, and `include_str!`
     source splitting.
   - Record exact evidence in ADR/spec/plan, roadmap, and option parity.
   - Run independent read-only six-axis review; main-thread repair and re-review
     until unconditional approval.

## Exit criteria

- Every committed sample/bucket/result literal matches the actual pinned C++
  oracle.
- Sampling and ordered voting retain exact source arithmetic and tie ownership.
- Inputs remain unchanged; production remains lifecycle-neutral and portable.
- No file reaches 400 LOC or uses source-splitting include macros.
- All runtime/static gates and independent review approve.

## Execution evidence

Independent source/specification repair and re-review approved before the
compiling 0/9 RED. Implementation gates pass focused 9/9, dependency 622/622,
workspace 6,282/6,282, warning-denying workspace Clippy, rustfmt, core/browser
wasm32, diff/LOC/static audits. The first independent six-axis implementation review rejected a
non-discriminating synthetic upper-wrap assertion. The main thread added a
production per-candidate accumulator seam/literal and killed the branch-removal
mutation, then reran every gate. Final read-only re-review approved
unconditionally with no remaining finding.
