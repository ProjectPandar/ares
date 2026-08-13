# Task 22O.71 implementation plan

1. Independently audit pinned `PrintObject.cpp:2725-2761,3114-3389`, the
   O43-O70 state graph, and the deep transaction seam.
2. Preserve a real-KSR behavioral RED at layer 15 through
   `bridge_over_infill::transaction::prepare`.
3. Implement private candidate expansion: O46 before O54, then ordered
   O55-O64 with per-cluster history and per-layer expansion state.
4. Implement private surface rewrite: O65 once per scheduled layer, O66-O69
   from one pre-commit region snapshot, O67/O68/O69 assembly, and one O70
   commit; dispose the predecessor on every error.
5. Activate the prepared lifecycle and add focused provenance, order,
   topology/metadata, ownership, unsupported-pattern, empty, and repeatability
   tests without hardcoded fixture geometry.
6. Run focused/dependency/workspace Nextest, strict Clippy/rustfmt, Tier-1
   portability builds, diff/LOC/static/include/no-staged gates, and the current
   KSR golden checkpoint.
7. Run independent read-only six-axis review; repair only in the main thread
   and repeat review until unconditional approval.

Lightning, adaptive/support-cubic infill, generic multi-region/other patterns,
parallel scheduling, the second/third bridge layers, combine-infill,
extrusion/motion/G-code/CLI, and complete golden parity remain deferred.

## Completion evidence

The final implementation passes focused O71 16/16, bridge dependency 240/240,
and workspace 6,473/6,473 with two configured skips. Strict Clippy/rustfmt,
diff/static/LOC/no-staged gates, WASM, both Windows targets, both macOS targets,
and independent review all pass. The prepared KSR result freezes 47 bridge
surfaces, 15,689 ordered points, 17 bridge layers, and SHA-256
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`.
The public API remains intentionally incomplete after O71; complete G-code
golden parity is not claimed.
