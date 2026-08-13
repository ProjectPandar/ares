# Task 22O.71 — first internal bridge transaction

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`PrintObject.cpp:2725-2761`, `3114-3310`, and `3315-3389`, closing the
already implemented O43-O70 dependencies at the
`PrintObject::bridge_over_infill` owner called by `PrintObject.cpp:673-680`.

Destination seam:

```rust
pub(in crate::project_slice) fn prepare(
    predecessor: PreparedPostBridgeCandidates,
) -> Result<PreparedPostBridgeOverInfill, SliceError>;
```

The deep `bridge_over_infill::transaction` successor must hide two private
ordinary modules: candidate expansion and region surface rewrite. It consumes
the raw O43 predecessor, preserves lower-anchor generation before clustering,
processes each cluster/layer/candidate in source-defined order, retains O64
completed history including present-empty keys, and only then runs O65-O70.
Every region must read one pre-commit surface collection, assemble O67 then O68
then O69, and commit once. O54-O64 use thick bridge Flow; O65 upper rings and
O66 use the matching normal solid-infill Flow. Errors dispose the whole owned
predecessor and expose no partial successor.

The real KSR RED must prove that layer 15 lacks `InternalBridge` before the
transaction and has source-derived `InternalBridge` output after it. Tests must
also cover lifecycle invocation/disposal, completed-map scheduling and
candidate order, full contour/ordered-hole/metadata snapshots, repeatability,
empty/no-candidate behavior, active unsupported sparse-pattern rejection, and
ownership cleanup. Do not use fixture branches, hardcoded KSR geometry,
fallbacks, source-splitting macros, or files of 400 LOC or more.

Included: active single-region non-Lightning CrossHatch behavior, O46-O70
composition, first internal bridge commit, and lifecycle activation.

Deferred: Lightning `2593-2723`; adaptive/support-cubic octrees `2728-2735`;
generic multi-region/other infill generation; TBB/logging/timeout/debug
adapters; second/third bridge layers `3393+`; combine-infill, extrusion,
motion, G-code, CLI, and complete golden parity.

Final gates: focused/dependency/workspace Nextest, strict Clippy/rustfmt, WASM
and x86_64/aarch64 Windows/macOS checks, diff/LOC/static/include/no-staged
checks, and independent read-only six-axis review followed by main-thread
repair and unconditional re-review.
