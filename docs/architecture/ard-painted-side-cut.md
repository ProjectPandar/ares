# ARD — one consumed upstream painted side-cut operation

Status: accepted SPEC boundary; docs then public RED only. No GREEN
or change to the frozen 1045b1775 branch is part of this slice.

`MultiMaterialSegmentation.cpp1144–1168` at8500fcdcc ports one side-cut operation,
with its2169–2181 gate/order and2197–2207 wrapper. The Rust destination is private
`ares-core/.../region_slices/painted/cut.rs`, directly consumed by existing
`apply_project` before caps/merge and actual `PrintObjectSlice874–886/1206–1224`
ownership apply. Existing omission/guard is a temporary upstream compatibility
shell, not a new Ares pipeline; no independent API, second registry or engine.

Non-negotiable decisions:

- Port executable source, not the misleading tooltip: width0/depth>0 and
  depth>width remain operative on even object indices. Do not activate the unused
  `max(width-depth,0)` temporary. Discovery Native output confirms both edges.
- Preserve double option→float mm→double scale division→float scaled radius,
  zero-based object layer indexing, state0/empty slots, zero-radius untouched
  vectors and side-minus-inward-whole-layer-offset ordering. Reuse existing kernels,
  including upstream Miter/3.0 offset and no-safety-offset difference defaults.
- Keep the same carriers, repaired graph, cap merge, shared registry, ordered
  actual apply and single compensation implementation. Do not cut caps or trim
  post-merge, offset individual colors, reorder coordinates/rings or copy a backend.
- Keep painted+beam deferred (separate generator atPrintObjectSlice1243), as well
  as fuzzy painting; preserve ordinary fuzzy and unpainted interlocking.
- Test the public seam with unchanged full ordered NativeCase/API/actual CLI and
  raw API==CLI. Only three target3MFs are new; reuse the existing painted-split
  control rather than copying it. No expected-error assertions or inverted RED.
- Both-mode Native-first execution is mandatory. Activity and filtered raw XYE
  traces are auxiliary, not private-stage/whole-output proof or performance data;
  final odd-layer changes do not justify an even-only G-code assertion.

Rejected: painted+beam first (different owner); tooltip clamping or subtracting
width-depth (wrong source); width-only partial helper (artificially splits one
operation); post-merge trimming/new registry (wrong seam). See
[SPEC](painted-side-cut-spec.md) for included/deferred behavior, selected matrix,
verification, artifact requirements and separate later gates.
