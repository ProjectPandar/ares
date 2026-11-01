# ARD 0109: NP beta vgcode viewer-time and path-split kernels at 345efec64

Assignment #290, Stage 2, after the Stage 1 extractor re-pin (`f416e1718`).
All ports are source-cited rewrite slices of OrcaSlicer
`345efec64c7b95a05ce5cf96df0dde66a158835c` (pinned at
`/home/indexyz/ares-coord/orca-source-250b`); no Ares-owned pipeline design.

## Slice 1 — marker-time layer-seeded cache (approved 2a)

- Upstream: `src/libvgcode/src/ViewerImpl.cpp:1189-1213` (build),
  `:990-992` (reset), `:1717-1732` (`get_estimated_time_at` layer-seeded
  partial sum with the `id >= size → 0.0f` guard); members
  `ViewerImpl.hpp:270-282`, `get_estimated_time` `ViewerImpl.hpp:179`.
- Destination: `crates/ares-vgcode/src/viewer.rs` — replace the full-prefix
  `elapsed_times` table with `total_time` plus `layer_first_vertex` /
  `layer_start_times`; query becomes guard + layer seed + partial loop with
  identical f32 addition order.
- Included: the exact f32 accumulation order (seed sums in vertex order, so
  results match a full accumulation bit-for-bit, as the upstream comment at
  `ViewerImpl.cpp:1204-1206` states), the OOB guard, the non-decreasing
  back-to-front first-vertex index build.
- Deferred: per-vertex `layer_duration` population (`ViewerImpl.cpp:1185-1188`)
  — no current case consumes it; `m_layers_in_vertex_order`
  (`ViewerImpl.hpp:282-284`) — only consumed by behavior no current case
  exercises.
- RED: the marker-time family NP cases at `f416e1718` (ares-wasm
  `marker_time_transport_preserves_native_bits_after_buffer_transfer`,
  `tagged_normal_time_matches_every_stable_native_vertex`); the ares-vgcode
  `marker_time` pair is the regression guard (passes before and after, by the
  upstream bit-equality design).
- Harness: `tools/chain-probe/marker-time-oracle.hpp` rebuilds the load-time
  caches locally (citing `ViewerImpl.cpp:1189-1213`) and calls the regenerated
  5-parameter kernel; future probe rebuilds stay consistent with the beta
  semantics. The merged-oracle binaries used by the parity env are prebuilt
  and unaffected.

## Slice 2 — starts_path acceleration/jerk path splits (approved 2a-prereq)

- Upstream: `src/slic3r/GUI/LibVGCode/LibVGCodeWrapper.cpp:227-240` —
  `starts_path` additionally splits when `prev.acceleration != curr.acceleration`
  or `prev.jerk != curr.jerk` ("ORCA: Split the path when a preview value
  changes"), which adds phantom vertices and therefore changes the vertex
  count (`convert()` sizes its array by the same test, `:255-263`).
- Destination: `crates/ares-vgcode/src/convert.rs:37-44` — add the two
  comparisons to the existing split predicate (the `mm3_per_mm` flow split is
  already ported).
- RED: ares-wasm `marker_time_transport_preserves_native_bits_after_buffer_transfer`
  at `f416e1718` fails the count gate 71 (Ares) vs 72 (Native).
- Included: predicate change only; no vertex layout change.

## Slice 3 — marker-time-bounds.mjs beta re-anchor (approved)

- Upstream effect: beta Native raw/stable dumps changed shape around the
  phantom-vertex boundary the tool documents.
- Change: `tools/chain-probe/marker-time-bounds.mjs` — ids and cardinality
  constants only; comparator semantics, the bounded-impact checks and the
  `stop, no waiver` policy stay unchanged. Before/after check list recorded in
  `/home/indexyz/ares-evidence/np-processor-vgcode-stage2/`.
- RED: `tagged_normal_time_matches_every_stable_native_vertex` crashes in the
  tool at `pairs.find` (propagation ids hard-coded to the 8500fcdc shape).

## Deferred — belt-printer get_layers_zs (per coordinator decision)

- `src/slic3r/GUI/GCodeViewer.cpp` `get_layers_zs()` gained a belt branch:
  if `m_belt_layer_zs` is non-empty, the slider/colour-change Zs come from the
  layers' print Z (populated on belt load paths) instead of toolpath heights.
- No current NP case exercises a belt printer: the custom-gcode-overview
  fixtures are `overview.3mf`, `two-plates.3mf`, `orca_cli_ender3.3mf`
  (Cartesian). Not ported; the extractor captures the branch verbatim in
  `tools/chain-probe/custom-gcode-overview/kernel*.hpp` and this ARD records
  the deferral.

## Non-negotiables

- Comparators and test semantics unchanged; each kernel is its own
  RED→GREEN commit with the RED run recorded before the port.
- No `#[allow]`/`#[expect]`; clippy/rustfmt clean; no probes or Native
  instrumentation beyond the approved harnesses above.
