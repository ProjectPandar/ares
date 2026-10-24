# ARD: P1S ByLayer brim contour materialization

Status: coordinator-authorized docs + tests-only RED; GREEN not authorized.
Owner: W5. Base: frozen rib3 `a96cf4d033b44ed9418975f1f68ac8118710a97e`.

## Upstream and destination

Pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Brim.cpp:813–856`, `tryExPolygonOffset` and
`makeBrimInfillImpl`, owns this rewrite slice. Included behavior: stepped
-0.5/-1.3/+0.3 spacing offsets, Douglas–Peucker passes, contour topology,
`union_pt_chained_outside_in`, and conversion to ordered polylines.
The adjacent reversal/connection consumer remains source-owned; it is not
permission to redesign an Ares brim pipeline.

Rust destination: `crates/ares-core/src/brims/ears/paths.rs::from_area`.
Existing approximation in this module is a temporary compatibility shell
around those upstream functions, not the specification. The existing
`project_slice/gcode_emit/brim.rs` plan and G-code emitter are consumers.
No production edits are part of this RED checkpoint. Any later change to
shared geometry requires evidence and separate boundary review.
Core behavior must remain platform-neutral on browser WASM, Windows, macOS,
and Linux; no filesystem or terminal dependencies enter ares-core.

## Witness and uncertainty

Immutable P1S ByLayer fixture SHA256:
`dcae275709dc16016c60308ae42d8523385dbabc18e5d31448f0baceb6e2d36a`.
Origin: worker1 sequential-next discovery, copied byte-for-byte, not re-exported.
Settings: outer_only width5/gap0.1, combine_brims0, perobject skirt loops2.
Discovery Native line1127 enters the contour at X112.995 Y133.257 and
steps through X115.766 before returning to X112.995 Y112.6; Ares instead
moves directly from Y150.992 to Y107.288. This establishes a physical
residual, not yet the precise faulty intermediate geometry operation.
Timing/M73 differences precede it. They stay in the complete comparison;
RED must not be described as an isolated geometry-only comparator failure.

## Non-negotiable decisions and deferred behavior

- Public seam: actual Native slice → public API → CLI, full retained streams,
  unchanged generator-only strict comparator and raw API/CLI equality.
- No masks, trimming, rounding fixes, lint allowances or implementation here.
- Freeze Type1 rib3 conversion and all other production code.
- Defer timing/M73 correction, brim-area construction, grouping/registration,
  shared Clipper rewrites and unrelated skirt/tower behavior. If localization
  exits this boundary, request a SPEC amendment rather than expanding scope.
- No GREEN, FULL, merge, rebase or push without coordinator authorization.

## RED and review gate

Run P1S target and the existing immutable OuterOnly width5 fixture as control
through the same Native/API/CLI seam. Require real Native brim activation and
hash identity; target additionally requires the witnessed Native contour.
Each mode uses CPU11/taskset, threads1, fresh caches; official then seeded.
Expected result is target RED with successful producers, control PASS in both
modes. Preserve commands, statuses, full outputs, raw diffs, hashes and
provenance; distinguish failure to launch from semantic RED.
Then send REVIEW-REQUEST plus inbox line and HOLD. GREEN exit criteria, if
later authorized, require full-stream parity; a remaining compound residual
must be reported, never masked or declared accepted.
