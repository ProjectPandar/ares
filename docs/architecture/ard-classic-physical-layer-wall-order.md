# ARD: Classic ordering uses upstream layer identity, not record position

Status: proposed with [SPEC](classic-physical-layer-wall-order-spec.md).
Base main:57fea94989ff362129d0e53da0c35c298822f689.

## Context and owning boundary

Pinned Orca8500fcdccaa10b5099ac20d252af3a7c560046f1,
`libslic3r/PerimeterGenerator.cpp:1463–1471`: Classic post-traversal first-layer
brim and later-layer InnerOuterInner ordering use PerimeterGenerator.layer_id.
Rust `classic/perimeter_append.rs::transform_object` instead passes flattened
region-major record enumeration as layer ID. Ares-only repeatable observations
retain complete public output and show arguments2/3 versus actual layers0/1.
The two Classic M later-owner Filled/None complete-stream REDs are independent
of nested Arachne counterbore behavior.

## Non-negotiable decisions

- Port the cited upstream layer-gated ordering boundary; no self-designed Ares
  pipeline, new sorting policy or compatibility fallback.
- Keep record index for slot alignment and region lookup; actual layer ID for
  upstream first/later-layer conditions. Planned layer index is a separate
  coordinate for physical-slice lookup; do not assume raft offset equality.
- Correct sandwich and first-layer OuterOnly brim operand handoffs within the
  exact existing `perimeter_append.rs` destination. Preserve emitted geometry,
  flow, wall-direction, inset permutation and current-source owner selection.
- Adjacent `lower_slices(index)` misuse is documented and deferred pending its
  own source-owned public RED/amendment; do not silently broaden this slice.
- Existing scaffold using record position as layer identity is replaced, not
  retained as a legacy route. No regions[0] or default-owner shortcut.
- Tests are written/frozen before production. Both existing M REDs compare
  complete ordered G-code with established generator-only normalization. The
  included brim consumer needs an accepted Native/public witness before coding.
- No Native runtime instrumentation, binary patch or comparator/assertion
  weakening. Temporary Ares diagnostics are archived and removed, not public API.
- No core filesystem/UI/terminal behavior; remain platform-neutral byte APIs
  for browser WASM, Windows, macOS and Linux. Coordinator approves exact hash
  and schedules full runs/push/merge; this docs-only proposal authorizes no fix.

## Included / deferred behavior

Included: actual-layer handoff for Classic sandwich gate and same first-layer
brim gate, plus exact two REDs and scoped ordering controls.
Deferred: lower-slice identity, raft policy changes, compatible-group ownership,
Native internal state, Arachne N-split, imports/classification/fill, loop nesting,
chaining and geometry/reorder algorithm changes. Evidence and exit gates are
in SPEC; any scope expansion requires a separate owner-approved amendment.
